//! Local `LanceDB` backend implementation.

use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use arrow_array::{
    Array, BooleanArray, RecordBatch, RecordBatchIterator, StringArray, UInt32Array, UInt64Array,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lance_index::scalar::FullTextSearchQuery;
use lancedb::{
    Table, connect,
    connection::Connection,
    index::{Index, IndexType, scalar::FtsIndexBuilder},
    query::{ExecutableQuery, QueryBase, Select},
};
use temporal_plane_core::{
    CoreError, MemoryId, RecordedAt,
    checkpoints::{Checkpoint, CheckpointRequest, CheckpointSummary, VersionNumber, VersionRecord},
    memory::{MemoryKind, MemoryRecord},
    query::{HistoryQuery, RecallQuery, SearchQuery, StatsQuery, StatsSnapshot},
    traits::{
        BackendCapabilities, BackendCapability, CheckpointBackend, HistoryBackend,
        MemoryRepository, RecallBackend, StatsBackend, StorageBackend,
    },
};
use thiserror::Error;
use tokio::runtime::{Builder, Handle, Runtime};

const MEMORIES_TABLE: &str = "memories";
const CHECKPOINTS_TABLE: &str = "checkpoints";
const SCHEMA_METADATA_TABLE: &str = "schema_metadata";
const PAYLOAD_COLUMN: &str = "payload_json";
const SCHEMA_VERSION: u64 = 1;

/// Backend-local error type for the `LanceDB` adapter crate.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LanceDbError {
    /// Wraps storage-agnostic domain validation failures.
    #[error(transparent)]
    Core(#[from] CoreError),

    /// Wraps `LanceDB` SDK failures.
    #[error(transparent)]
    LanceDb(#[from] lancedb::Error),

    /// Wraps Arrow batch construction failures.
    #[error(transparent)]
    Arrow(#[from] arrow_schema::ArrowError),

    /// Wraps filesystem failures.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Wraps JSON serialization and deserialization failures.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// Indicates a required table is missing.
    #[error("required table `{table}` is missing")]
    MissingTable {
        /// Missing table name.
        table: &'static str,
    },

    /// Indicates an existing memory identifier would be duplicated.
    #[error("memory `{id}` already exists")]
    DuplicateMemoryId {
        /// Duplicate identifier value.
        id: String,
    },

    /// Indicates the requested store path is invalid for the selected operation.
    #[error("invalid store path `{path}`: {details}")]
    InvalidStorePath {
        /// Store path.
        path: PathBuf,
        /// Human-readable failure details.
        details: &'static str,
    },

    /// Indicates a checkpoint name already exists.
    #[error("checkpoint `{name}` already exists")]
    DuplicateCheckpointName {
        /// Duplicate checkpoint name.
        name: String,
    },

    /// Indicates a checkpoint already exists for the resolved version.
    #[error("version `{version}` already has a checkpoint")]
    DuplicateCheckpointVersion {
        /// Duplicate version number.
        version: u64,
    },

    /// Indicates stored data could not be decoded.
    #[error("invalid persisted data for `{field}`: {details}")]
    InvalidData {
        /// Logical field name.
        field: &'static str,
        /// Human-readable failure details.
        details: String,
    },

    /// Indicates a timestamp is outside the supported range.
    #[error("timestamp for `{field}` is outside the supported range")]
    InvalidTimestamp {
        /// Logical field name.
        field: &'static str,
    },

    /// Placeholder until import/export is implemented.
    #[error("{feature} is not implemented yet")]
    NotImplemented {
        /// Unimplemented feature name.
        feature: &'static str,
    },

    /// Indicates a sync wrapper was called from within an async runtime.
    #[error("sync LanceDB backend APIs cannot be called from an async runtime")]
    UnsupportedCallerContext,
}

/// A persistent local backend backed by `LanceDB`.
pub struct LanceDbBackend {
    path: PathBuf,
    runtime: Runtime,
    memories: Table,
    checkpoints: Table,
    schema_metadata: Table,
}

impl std::fmt::Debug for LanceDbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanceDbBackend")
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}

impl LanceDbBackend {
    /// Initializes a local store, creating required tables when missing.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store or its required tables cannot be created.
    pub fn init(path: impl AsRef<Path>) -> Result<Self, LanceDbError> {
        Self::open_internal(path.as_ref(), true)
    }

    /// Opens an existing local store.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store cannot be opened or required tables are missing.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LanceDbError> {
        Self::open_internal(path.as_ref(), false)
    }

    /// Opens a local store, creating it if it does not exist yet.
    ///
    /// In the MVP this is equivalent to [`Self::init`]. The separate entry
    /// point preserves room for future open-vs-create policy differences.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store cannot be opened or initialized.
    pub fn connect_or_init(path: impl AsRef<Path>) -> Result<Self, LanceDbError> {
        Self::open_internal(path.as_ref(), true)
    }

    /// Returns the on-disk store path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the persisted schema version.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the metadata table cannot be read.
    pub fn schema_version(&self) -> Result<u64, LanceDbError> {
        let batches = self.block_on(async {
            self.schema_metadata
                .query()
                .select(Select::Columns(vec!["schema_version".to_owned()]))
                .limit(1)
                .execute()
                .await?
                .try_collect::<Vec<_>>()
                .await
        })?;

        let Some(batch) = batches.first() else {
            return Err(LanceDbError::InvalidData {
                field: "schema_version",
                details: "missing schema metadata row".to_owned(),
            });
        };
        let Some(column) = batch
            .column_by_name("schema_version")
            .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
        else {
            return Err(LanceDbError::InvalidData {
                field: "schema_version",
                details: "expected UInt64 column".to_owned(),
            });
        };

        if column.is_empty() {
            return Err(LanceDbError::InvalidData {
                field: "schema_version",
                details: "missing schema_version value".to_owned(),
            });
        }

        if column.is_null(0) {
            return Err(LanceDbError::InvalidData {
                field: "schema_version",
                details: "null schema_version value".to_owned(),
            });
        }

        Ok(column.value(0))
    }

    /// Deletes a memory by identifier.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the delete operation fails.
    pub fn delete_memory(&mut self, id: &MemoryId) -> Result<bool, LanceDbError> {
        let filter = string_filter("id", id.as_str());
        if self.count_memories(Some(filter.clone()))? == 0 {
            return Ok(false);
        }

        self.block_on(self.memories.delete(&filter))?;
        Ok(true)
    }

    /// Placeholder export skeleton for Milestone 2.
    ///
    /// # Errors
    ///
    /// Always returns [`LanceDbError::NotImplemented`].
    pub fn export_store(&self, _destination: impl AsRef<Path>) -> Result<(), LanceDbError> {
        Err(LanceDbError::NotImplemented { feature: "export" })
    }

    /// Placeholder import skeleton for Milestone 2.
    ///
    /// # Errors
    ///
    /// Always returns [`LanceDbError::NotImplemented`].
    pub fn import_store(&mut self, _source: impl AsRef<Path>) -> Result<(), LanceDbError> {
        Err(LanceDbError::NotImplemented { feature: "import" })
    }

    fn open_internal(path: &Path, create_missing: bool) -> Result<Self, LanceDbError> {
        if Handle::try_current().is_ok() {
            return Err(LanceDbError::UnsupportedCallerContext);
        }

        if create_missing {
            std::fs::create_dir_all(path)?;
        } else {
            let metadata = std::fs::metadata(path).map_err(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    LanceDbError::InvalidStorePath {
                        path: path.to_path_buf(),
                        details: "store path does not exist",
                    }
                } else {
                    LanceDbError::Io(error)
                }
            })?;

            if !metadata.is_dir() {
                return Err(LanceDbError::InvalidStorePath {
                    path: path.to_path_buf(),
                    details: "store path is not a directory",
                });
            }
        }

        let runtime = Builder::new_multi_thread().enable_all().build()?;
        let uri = path.to_string_lossy().to_string();
        let connection = runtime.block_on(connect(&uri).execute())?;
        let existing_tables = runtime.block_on(connection.table_names().execute())?;

        let memories = Self::open_or_create_memories_table(
            &runtime,
            &connection,
            &existing_tables,
            create_missing,
        )?;
        let checkpoints = Self::open_or_create_checkpoints_table(
            &runtime,
            &connection,
            &existing_tables,
            create_missing,
        )?;
        let schema_metadata = Self::open_or_create_schema_metadata_table(
            &runtime,
            &connection,
            &existing_tables,
            create_missing,
        )?;

        let backend = Self {
            path: path.to_path_buf(),
            runtime,
            memories,
            checkpoints,
            schema_metadata,
        };

        backend.ensure_fts_index()?;
        backend.ensure_schema_version_row()?;
        Ok(backend)
    }

    fn open_or_create_memories_table(
        runtime: &Runtime,
        connection: &Connection,
        existing_tables: &[String],
        create_missing: bool,
    ) -> Result<Table, LanceDbError> {
        if existing_tables.iter().any(|name| name == MEMORIES_TABLE) {
            return Ok(runtime.block_on(connection.open_table(MEMORIES_TABLE).execute())?);
        }

        if !create_missing {
            return Err(LanceDbError::MissingTable {
                table: MEMORIES_TABLE,
            });
        }

        Ok(runtime.block_on(
            connection
                .create_empty_table(MEMORIES_TABLE, memories_schema())
                .execute(),
        )?)
    }

    fn open_or_create_checkpoints_table(
        runtime: &Runtime,
        connection: &Connection,
        existing_tables: &[String],
        create_missing: bool,
    ) -> Result<Table, LanceDbError> {
        if existing_tables.iter().any(|name| name == CHECKPOINTS_TABLE) {
            return Ok(runtime.block_on(connection.open_table(CHECKPOINTS_TABLE).execute())?);
        }

        if !create_missing {
            return Err(LanceDbError::MissingTable {
                table: CHECKPOINTS_TABLE,
            });
        }

        Ok(runtime.block_on(
            connection
                .create_empty_table(CHECKPOINTS_TABLE, checkpoints_schema())
                .execute(),
        )?)
    }

    fn open_or_create_schema_metadata_table(
        runtime: &Runtime,
        connection: &Connection,
        existing_tables: &[String],
        create_missing: bool,
    ) -> Result<Table, LanceDbError> {
        if existing_tables
            .iter()
            .any(|name| name == SCHEMA_METADATA_TABLE)
        {
            return Ok(runtime.block_on(connection.open_table(SCHEMA_METADATA_TABLE).execute())?);
        }

        if !create_missing {
            return Err(LanceDbError::MissingTable {
                table: SCHEMA_METADATA_TABLE,
            });
        }

        Ok(runtime.block_on(
            connection
                .create_empty_table(SCHEMA_METADATA_TABLE, schema_metadata_schema())
                .execute(),
        )?)
    }

    fn ensure_fts_index(&self) -> Result<(), LanceDbError> {
        let indices = self.block_on(self.memories.list_indices())?;
        let has_fts = indices.iter().any(|index| {
            index.index_type == IndexType::FTS && index.columns == ["fts_text".to_owned()]
        });

        if !has_fts {
            self.block_on(
                self.memories
                    .create_index(&["fts_text"], Index::FTS(FtsIndexBuilder::default()))
                    .execute(),
            )?;
        }

        Ok(())
    }

    fn ensure_schema_version_row(&self) -> Result<(), LanceDbError> {
        if self.block_on(self.schema_metadata.count_rows(None))? > 0 {
            return Ok(());
        }

        let schema = schema_metadata_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(UInt64Array::from(vec![SCHEMA_VERSION]))],
        )?;
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        self.block_on(self.schema_metadata.add(reader).execute())?;
        Ok(())
    }

    fn count_memories(&self, filter: Option<String>) -> Result<usize, LanceDbError> {
        self.block_on(self.memories.count_rows(filter))
    }

    fn query_payloads(
        &self,
        table: &Table,
        filter: Option<String>,
        limit: Option<usize>,
        full_text: Option<String>,
    ) -> Result<Vec<String>, LanceDbError> {
        let batches = self.block_on(async {
            let mut query = table
                .query()
                .select(Select::Columns(vec![PAYLOAD_COLUMN.to_owned()]));

            if let Some(filter) = filter {
                query = query.only_if(filter);
            }
            if let Some(full_text) = full_text {
                query = query.full_text_search(FullTextSearchQuery::new(full_text));
            }
            if let Some(limit) = limit {
                query = query.limit(limit);
            }

            query.execute().await?.try_collect::<Vec<_>>().await
        })?;

        let mut payloads = Vec::new();
        for batch in batches {
            let Some(array) = batch.column(0).as_any().downcast_ref::<StringArray>() else {
                return Err(LanceDbError::InvalidData {
                    field: PAYLOAD_COLUMN,
                    details: "expected Utf8 payload column".to_owned(),
                });
            };

            for index in 0..array.len() {
                if array.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: PAYLOAD_COLUMN,
                        details: "payload column contained null".to_owned(),
                    });
                }
                payloads.push(array.value(index).to_owned());
            }
        }

        Ok(payloads)
    }

    fn block_on<F, T>(&self, future: F) -> Result<T, LanceDbError>
    where
        F: Future<Output = Result<T, lancedb::Error>>,
    {
        if Handle::try_current().is_ok() {
            return Err(LanceDbError::UnsupportedCallerContext);
        }

        Ok(self.runtime.block_on(future)?)
    }
}

impl StorageBackend for LanceDbBackend {
    type Error = LanceDbError;

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::new([
            BackendCapability::Remember,
            BackendCapability::Search,
            BackendCapability::History,
            BackendCapability::Checkpoints,
        ])
    }
}

impl MemoryRepository for LanceDbBackend {
    fn remember(&mut self, record: MemoryRecord) -> Result<MemoryRecord, Self::Error> {
        if self.count_memories(Some(string_filter("id", record.id().as_str())))? > 0 {
            return Err(LanceDbError::DuplicateMemoryId {
                id: record.id().as_str().to_owned(),
            });
        }

        let payload_json = serde_json::to_string(&record)?;
        let (created_secs, created_nanos) =
            system_time_to_parts(record.created_at().value(), "created_at")?;
        let (updated_secs, updated_nanos) =
            system_time_to_parts(record.updated_at().value(), "updated_at")?;
        let schema = memories_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![Some(record.id().as_str())])),
                Arc::new(StringArray::from(vec![Some(record.scope_id().as_str())])),
                Arc::new(StringArray::from(vec![Some(memory_kind_name(
                    record.kind(),
                ))])),
                Arc::new(StringArray::from(vec![Some(record.title())])),
                Arc::new(StringArray::from(vec![Some(record.summary())])),
                Arc::new(StringArray::from(vec![Some(record.detail())])),
                Arc::new(StringArray::from(vec![Some(record.fts_text())])),
                Arc::new(UInt32Array::from(vec![u32::from(
                    record.importance().value(),
                )])),
                Arc::new(UInt32Array::from(vec![u32::from(
                    record.confidence().value(),
                )])),
                Arc::new(UInt64Array::from(vec![created_secs])),
                Arc::new(UInt32Array::from(vec![created_nanos])),
                Arc::new(UInt64Array::from(vec![updated_secs])),
                Arc::new(UInt32Array::from(vec![updated_nanos])),
                Arc::new(BooleanArray::from(vec![record.pin_state().is_pinned()])),
                Arc::new(StringArray::from(vec![Some(payload_json.as_str())])),
            ],
        )?;
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        self.block_on(self.memories.add(reader).execute())?;
        Ok(record)
    }

    fn get(&self, id: &MemoryId) -> Result<Option<MemoryRecord>, Self::Error> {
        let payloads = self.query_payloads(
            &self.memories,
            Some(string_filter("id", id.as_str())),
            Some(1),
            None,
        )?;

        payloads
            .into_iter()
            .next()
            .map(|payload| serde_json::from_str::<MemoryRecord>(&payload).map_err(Into::into))
            .transpose()
    }
}

impl RecallBackend for LanceDbBackend {
    fn recall(&self, query: &RecallQuery) -> Result<Vec<MemoryRecord>, Self::Error> {
        // TODO(m3uo4te8-followup): apply `DisclosureDepth` during ranking once
        // layered recall semantics land in the backend.
        let payloads = self.query_payloads(
            &self.memories,
            query
                .scope()
                .map(|scope| string_filter("scope_id", scope.as_str())),
            Some(usize::from(query.limit().value())),
            query.text().map(ToOwned::to_owned),
        )?;

        payloads
            .into_iter()
            .map(|payload| serde_json::from_str::<MemoryRecord>(&payload).map_err(Into::into))
            .collect()
    }

    fn search(&self, query: &SearchQuery) -> Result<Vec<MemoryRecord>, Self::Error> {
        let payloads = self.query_payloads(
            &self.memories,
            query
                .scope()
                .map(|scope| string_filter("scope_id", scope.as_str())),
            Some(usize::from(query.limit().value())),
            Some(query.text().to_owned()),
        )?;

        payloads
            .into_iter()
            .map(|payload| serde_json::from_str::<MemoryRecord>(&payload).map_err(Into::into))
            .collect()
    }
}

impl HistoryBackend for LanceDbBackend {
    fn history(&self, query: &HistoryQuery) -> Result<Vec<VersionRecord>, Self::Error> {
        if query.scope().is_some() {
            return Err(LanceDbError::NotImplemented {
                feature: "scoped history",
            });
        }

        let versions = self.block_on(self.memories.list_versions())?;
        let checkpoint_map: HashMap<u64, Vec<Checkpoint>> = self
            .list_checkpoints()?
            .into_iter()
            .fold(HashMap::new(), |mut checkpoints, checkpoint| {
                checkpoints
                    .entry(checkpoint.version().value())
                    .or_default()
                    .push(checkpoint);
                checkpoints
            });

        let mut records: Vec<_> = versions
            .into_iter()
            .map(|version| {
                let recorded_at = RecordedAt::new(version.timestamp.into());
                let checkpoint = checkpoint_map
                    .get(&version.version)
                    .and_then(|checkpoints| checkpoints.first())
                    .map(|checkpoint| {
                        CheckpointSummary::new(checkpoint.name().clone(), checkpoint.version())
                    });
                let summary = checkpoint_map
                    .get(&version.version)
                    .and_then(|checkpoints| checkpoints.first())
                    .and_then(|checkpoint| checkpoint.description().map(ToOwned::to_owned));

                VersionRecord::new(
                    VersionNumber::new(version.version),
                    recorded_at,
                    checkpoint,
                    summary,
                )
            })
            .collect();

        records.sort_by_key(|record| std::cmp::Reverse(record.version().value()));
        records.truncate(usize::from(query.limit().value()));
        Ok(records)
    }
}

impl CheckpointBackend for LanceDbBackend {
    fn checkpoint(&mut self, request: &CheckpointRequest) -> Result<Checkpoint, Self::Error> {
        let existing_checkpoints = self.list_checkpoints()?;
        if existing_checkpoints
            .iter()
            .any(|checkpoint| checkpoint.name() == request.name())
        {
            return Err(LanceDbError::DuplicateCheckpointName {
                name: request.name().as_str().to_owned(),
            });
        }

        let version = self.block_on(self.memories.version())?;
        if existing_checkpoints
            .iter()
            .any(|checkpoint| checkpoint.version().value() == version)
        {
            return Err(LanceDbError::DuplicateCheckpointVersion { version });
        }

        let created_at = RecordedAt::now();
        let checkpoint = Checkpoint::new_at(
            request.name().clone(),
            VersionNumber::new(version),
            created_at,
            request.description().map(ToOwned::to_owned),
        );

        let (created_secs, created_nanos) =
            system_time_to_parts(created_at.value(), "checkpoint_created_at")?;
        let payload_json = serde_json::to_string(&checkpoint)?;
        let schema = checkpoints_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![Some(request.name().as_str())])),
                Arc::new(UInt64Array::from(vec![version])),
                Arc::new(UInt64Array::from(vec![created_secs])),
                Arc::new(UInt32Array::from(vec![created_nanos])),
                Arc::new(StringArray::from(vec![Some(payload_json.as_str())])),
            ],
        )?;
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));

        let mut tags = self.block_on(self.memories.tags())?;
        self.block_on(tags.create(request.name().as_str(), version))?;

        if let Err(error) = self.block_on(self.checkpoints.add(reader).execute()) {
            let _ = self.block_on(tags.delete(request.name().as_str()));
            return Err(error);
        }

        Ok(checkpoint)
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, Self::Error> {
        let payloads = self.query_payloads(&self.checkpoints, None, None, None)?;

        payloads
            .into_iter()
            .map(|payload| serde_json::from_str::<Checkpoint>(&payload).map_err(Into::into))
            .collect()
    }
}

impl StatsBackend for LanceDbBackend {
    fn stats(&self, query: &StatsQuery) -> Result<StatsSnapshot, Self::Error> {
        let scope_filter = query
            .scope()
            .map(|scope| string_filter("scope_id", scope.as_str()));
        let total_memories = self.count_memories(scope_filter.clone())? as u64;
        let pinned_filter = match scope_filter {
            Some(filter) => Some(format!("{filter} AND pinned = true")),
            None => Some("pinned = true".to_owned()),
        };
        let pinned_memories = self.count_memories(pinned_filter)? as u64;
        let version_count = self.block_on(self.memories.list_versions())?.len() as u64;
        let latest_checkpoint = self
            .list_checkpoints()?
            .into_iter()
            .max_by_key(|checkpoint| checkpoint.version().value())
            .map(|checkpoint| checkpoint.name().clone());

        Ok(StatsSnapshot::new(
            total_memories,
            pinned_memories,
            version_count,
            query.scope().cloned(),
            latest_checkpoint,
        ))
    }
}

fn memories_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Utf8, false),
        Field::new("scope_id", DataType::Utf8, false),
        Field::new("kind", DataType::Utf8, false),
        Field::new("title", DataType::Utf8, false),
        Field::new("summary", DataType::Utf8, false),
        Field::new("detail", DataType::Utf8, false),
        Field::new("fts_text", DataType::Utf8, false),
        Field::new("importance", DataType::UInt32, false),
        Field::new("confidence", DataType::UInt32, false),
        Field::new("created_at_secs", DataType::UInt64, false),
        Field::new("created_at_nanos", DataType::UInt32, false),
        Field::new("updated_at_secs", DataType::UInt64, false),
        Field::new("updated_at_nanos", DataType::UInt32, false),
        Field::new("pinned", DataType::Boolean, false),
        Field::new(PAYLOAD_COLUMN, DataType::Utf8, false),
    ]))
}

fn checkpoints_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("name", DataType::Utf8, false),
        Field::new("version", DataType::UInt64, false),
        Field::new("created_at_secs", DataType::UInt64, false),
        Field::new("created_at_nanos", DataType::UInt32, false),
        Field::new(PAYLOAD_COLUMN, DataType::Utf8, false),
    ]))
}

fn schema_metadata_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![Field::new(
        "schema_version",
        DataType::UInt64,
        false,
    )]))
}

fn string_filter(column: &str, value: &str) -> String {
    format!("{column} = '{}'", value.replace('\'', "''"))
}

fn system_time_to_parts(
    value: SystemTime,
    field: &'static str,
) -> Result<(u64, u32), LanceDbError> {
    let duration = value
        .duration_since(UNIX_EPOCH)
        .map_err(|_| LanceDbError::InvalidTimestamp { field })?;
    Ok((duration.as_secs(), duration.subsec_nanos()))
}

fn memory_kind_name(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Observation => "observation",
        MemoryKind::Decision => "decision",
        MemoryKind::Preference => "preference",
        MemoryKind::Summary => "summary",
        MemoryKind::Fact => "fact",
        MemoryKind::Procedure => "procedure",
        MemoryKind::Warning => "warning",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use tempfile::TempDir;
    use temporal_plane_core::{
        CheckpointName, Confidence, Importance, QueryLimit, ScopeId, SearchQuery, StatsQuery,
        TagName,
        traits::{
            CheckpointBackend, HistoryBackend, MemoryRepository, RecallBackend, StatsBackend,
        },
    };

    use super::*;

    fn build_memory(id: &str, scope: &str, title: &str, detail: &str) -> MemoryRecord {
        MemoryRecord::builder(
            MemoryId::try_from(id).expect("valid id"),
            ScopeId::try_from(scope).expect("valid scope"),
            MemoryKind::Decision,
        )
        .title(title)
        .expect("valid title")
        .summary("summary")
        .expect("valid summary")
        .detail(detail)
        .expect("valid detail")
        .importance(Importance::new(90).expect("valid importance"))
        .confidence(Confidence::new(95).expect("valid confidence"))
        .add_tag(TagName::try_from("milestone-2").expect("valid tag"))
        .metadata(BTreeMap::from([(
            "owner".to_string(),
            "backend".to_string(),
        )]))
        .build()
        .expect("memory should build")
    }

    fn new_backend() -> (TempDir, LanceDbBackend) {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        (temp_dir, backend)
    }

    #[test]
    fn init_sets_schema_version() {
        let (_temp_dir, backend) = new_backend();

        assert_eq!(
            backend.schema_version().expect("schema version"),
            SCHEMA_VERSION
        );
    }

    #[test]
    fn remember_get_search_and_delete_memory() {
        let (_temp_dir, mut backend) = new_backend();
        let memory = build_memory(
            "memory:one",
            "repo:temporal-plane",
            "Store milestone",
            "Use LanceDB for local persistence.",
        );

        let stored = backend
            .remember(memory.clone())
            .expect("memory should store");
        let fetched = backend
            .get(&MemoryId::try_from("memory:one").expect("valid id"))
            .expect("lookup should succeed")
            .expect("memory should exist");
        let results = backend
            .search(
                &SearchQuery::new(
                    "LanceDB",
                    Some(ScopeId::try_from("repo:temporal-plane").expect("valid scope")),
                    QueryLimit::new(10).expect("valid limit"),
                )
                .expect("search query should build"),
            )
            .expect("search should succeed");
        let deleted = backend
            .delete_memory(&MemoryId::try_from("memory:one").expect("valid id"))
            .expect("delete should succeed");

        assert_eq!(stored, memory);
        assert_eq!(fetched, memory);
        assert_eq!(results, vec![memory]);
        assert!(deleted);
        assert!(
            backend
                .get(&MemoryId::try_from("memory:one").expect("valid id"))
                .expect("lookup should succeed")
                .is_none()
        );
    }

    #[test]
    fn open_existing_store_recovers_memories_and_checkpoints() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        {
            let mut backend =
                LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
            let memory = build_memory(
                "memory:checkpoint",
                "repo:temporal-plane",
                "Checkpoint store",
                "Persist checkpoints with tags.",
            );
            backend.remember(memory).expect("memory should store");
            backend
                .checkpoint(&CheckpointRequest::new(
                    CheckpointName::try_from("m2-freeze").expect("valid checkpoint name"),
                    Some("before reopening".to_string()),
                ))
                .expect("checkpoint should be created");
        }

        let reopened = LanceDbBackend::open(temp_dir.path()).expect("backend should reopen");
        let memory = reopened
            .get(&MemoryId::try_from("memory:checkpoint").expect("valid id"))
            .expect("lookup should succeed");
        let checkpoints = reopened
            .list_checkpoints()
            .expect("checkpoints should list");

        assert!(memory.is_some());
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].name().as_str(), "m2-freeze");
    }

    #[test]
    fn open_missing_store_path_fails_without_creating_directory() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let missing_path = temp_dir.path().join("missing-store");

        let error = LanceDbBackend::open(&missing_path).expect_err("open should fail");

        assert!(matches!(
            error,
            LanceDbError::InvalidStorePath {
                details: "store path does not exist",
                ..
            }
        ));
        assert!(!missing_path.exists());
    }

    #[test]
    fn stats_and_history_reflect_persisted_versions() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:history",
                "repo:temporal-plane",
                "History entry",
                "Version listing should work.",
            ))
            .expect("memory should store");
        backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("history-tag").expect("valid checkpoint name"),
                Some("checkpoint summary".to_string()),
            ))
            .expect("checkpoint should be created");

        let stats = backend
            .stats(&StatsQuery::new(None))
            .expect("stats should load");
        let history = backend
            .history(&HistoryQuery::new(
                None,
                QueryLimit::new(20).expect("valid limit"),
            ))
            .expect("history should load");

        assert_eq!(stats.total_memories(), 1);
        assert_eq!(
            stats.latest_checkpoint().map(CheckpointName::as_str),
            Some("history-tag")
        );
        assert!(!history.is_empty());
        assert!(history.iter().any(|entry| {
            entry
                .checkpoint()
                .map(|checkpoint| checkpoint.name().as_str())
                == Some("history-tag")
        }));
    }

    #[test]
    fn history_rejects_scope_filters() {
        let (_temp_dir, backend) = new_backend();

        let error = backend
            .history(&HistoryQuery::new(
                Some(ScopeId::try_from("repo:temporal-plane").expect("valid scope")),
                QueryLimit::new(5).expect("valid limit"),
            ))
            .expect_err("scoped history should be rejected explicitly");

        assert!(matches!(
            error,
            LanceDbError::NotImplemented {
                feature: "scoped history"
            }
        ));
    }

    #[test]
    fn checkpoint_rejects_duplicate_version_tags() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:duplicate-checkpoint",
                "repo:temporal-plane",
                "Duplicate checkpoint",
                "Only one checkpoint may target a version.",
            ))
            .expect("memory should store");
        backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("v1-tag").expect("valid checkpoint name"),
                Some("first checkpoint".to_string()),
            ))
            .expect("checkpoint should be created");

        let error = backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("v1-tag-second").expect("valid checkpoint name"),
                Some("duplicate version checkpoint".to_string()),
            ))
            .expect_err("duplicate checkpoint version should fail");

        assert!(matches!(
            error,
            LanceDbError::DuplicateCheckpointVersion { .. }
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn sync_backend_calls_fail_inside_async_runtime() {
        let temp_dir = TempDir::new().expect("tempdir should be created");

        let error = LanceDbBackend::init(temp_dir.path())
            .expect_err("sync backend initialization should be rejected");

        assert!(matches!(error, LanceDbError::UnsupportedCallerContext));
    }
}
