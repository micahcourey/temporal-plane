//! Local `LanceDB` backend implementation.

use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use arrow_array::{
    Array, BooleanArray, RecordBatch, RecordBatchIterator, StringArray, UInt32Array, UInt64Array,
};
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use lance::dataset::{builder::DatasetBuilder as LanceDatasetBuilder, refs::BranchContents};
use lance_index::scalar::FullTextSearchQuery;
use lancedb::{
    Table,
    arrow::SendableRecordBatchStream,
    connect,
    connection::Connection,
    index::{Index, IndexType, scalar::FtsIndexBuilder},
    query::{ExecutableQuery, QueryBase, Select},
    table::{BaseTable, CompactionOptions, NativeTable, OptimizeAction, OptimizeOptions},
};
use mnemix_core::{
    BranchListResult, BranchName, BranchRecord, BranchRequest, BranchStatus, CheckpointName,
    CloneInfo, CloneKind, CoreError, ImportStageRequest, ImportStageResult, Importance, MemoryId,
    OptimizeRequest, OptimizeResult, RecordedAt, RestoreRequest, RestoreResult, ScopeId,
    checkpoints::{
        Checkpoint, CheckpointRequest, CheckpointSelector, CheckpointSummary, VersionNumber,
        VersionRecord,
    },
    memory::{MemoryKind, MemoryRecord, PinState},
    query::{
        DisclosureDepth, HistoryQuery, QueryLimit, RecallEntry, RecallExplanation, RecallLayer,
        RecallQuery, RecallReason, RecallResult, SearchQuery, StatsQuery, StatsSnapshot,
    },
    retention::{CleanupMode, PreOperationCheckpointPolicy},
    traits::{
        AdvancedStorageBackend, BackendCapabilities, BackendCapability, CheckpointBackend,
        HistoryBackend, MemoryRepository, OptimizeBackend, PinningBackend, RecallBackend,
        RestoreBackend, StatsBackend, StorageBackend,
    },
};
use thiserror::Error;
use tokio::runtime::{Builder, Handle, Runtime};

const MEMORIES_TABLE: &str = "memories";
const CHECKPOINTS_TABLE: &str = "checkpoints";
const SCHEMA_METADATA_TABLE: &str = "schema_metadata";
const PAYLOAD_COLUMN: &str = "payload_json";
const SCHEMA_VERSION: u64 = 1;
/// Maximum pinned-context entries surfaced for any recall request.
const PINNED_CONTEXT_HARD_CAP: usize = 3;
/// Bounded over-fetch window used so product-side recall ranking can bucket
/// results without materializing every matching row.
const RECALL_FETCH_MULTIPLIER: usize = 4;

/// Backend-local error type for the `LanceDB` adapter crate.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum LanceDbError {
    /// Wraps lower-level Lance dataset failures.
    #[error(transparent)]
    Lance(#[from] lance::Error),

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

    /// Indicates a branch could not be resolved by name.
    #[error("branch `{name}` was not found")]
    BranchNotFound {
        /// Missing branch name.
        name: String,
    },

    /// Indicates a branch still contains staged changes.
    #[error("branch `{name}` has staged changes and cannot be deleted")]
    BranchHasChanges {
        /// Branch name that still diverges from its parent version.
        name: String,
    },

    /// Indicates a checkpoint could not be resolved by name.
    #[error("checkpoint `{name}` was not found")]
    CheckpointNotFound {
        /// Missing checkpoint name.
        name: String,
    },

    /// Indicates a version could not be resolved from history.
    #[error("version `{version}` was not found")]
    VersionNotFound {
        /// Missing version number.
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

    /// Indicates the current policy requires a caller-provided checkpoint.
    #[error("{operation} requires a caller-provided checkpoint before continuing")]
    CallerCheckpointRequired {
        /// The operation that requires a checkpoint.
        operation: &'static str,
    },

    /// Indicates pruning was requested while cleanup remains disabled.
    #[error("retention policy does not allow pruning old versions")]
    CleanupNotAllowed,

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
        let batches: Vec<RecordBatch> = self.block_on(async {
            let stream: SendableRecordBatchStream = self
                .schema_metadata
                .query()
                .select(Select::Columns(vec!["schema_version".to_owned()]))
                .limit(1)
                .execute()
                .await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
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

    /// Lists stored memories without requiring a text search.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the backing table cannot be queried.
    pub fn list_memories(
        &self,
        scope: Option<&ScopeId>,
        limit: QueryLimit,
    ) -> Result<Vec<MemoryRecord>, LanceDbError> {
        let payloads = self.query_payloads(
            &self.memories,
            scope.map(|value| string_filter("scope_id", value.as_str())),
            Some(usize::from(limit.value())),
            None,
        )?;

        let mut records = decode_memory_records(payloads)?;
        sort_memory_records(&mut records);
        Ok(records)
    }

    /// Lists pinned memories.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the backing table cannot be queried.
    pub fn list_pinned_memories(
        &self,
        scope: Option<&ScopeId>,
        limit: QueryLimit,
    ) -> Result<Vec<MemoryRecord>, LanceDbError> {
        let filter = match scope {
            Some(value) => Some(format!(
                "{} AND pinned = true",
                string_filter("scope_id", value.as_str())
            )),
            None => Some("pinned = true".to_owned()),
        };
        let payloads = self.query_payloads(
            &self.memories,
            filter,
            Some(usize::from(limit.value())),
            None,
        )?;

        let mut records = decode_memory_records(payloads)?;
        sort_memory_records(&mut records);
        Ok(records)
    }

    /// Placeholder export skeleton for Milestone 2.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the clone cannot be created.
    pub fn export_store(&self, destination: impl AsRef<Path>) -> Result<(), LanceDbError> {
        self.deep_clone(destination.as_ref())?;
        Ok(())
    }

    /// Placeholder import skeleton for Milestone 2.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the import cannot be staged.
    pub fn import_store(
        &mut self,
        source: impl AsRef<Path>,
    ) -> Result<ImportStageResult, LanceDbError> {
        self.stage_import(&ImportStageRequest::new(source.as_ref().to_path_buf()))
    }

    /// Restores the memories table from a historical version or checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the restore target cannot be resolved or
    /// the underlying restore operation fails.
    pub(crate) fn restore_store(
        &mut self,
        request: &RestoreRequest,
    ) -> Result<RestoreResult, LanceDbError> {
        let previous_version = self.block_on(self.memories.version())?;
        let restored_version = self.resolve_version_selector(request.target())?;
        let pre_restore_checkpoint = self.maybe_create_operation_checkpoint(
            request.retention_policy().pre_restore_checkpoint(),
            "restore",
            Some(format!(
                "automatic checkpoint before restore to {}",
                checkpoint_selector_label(request.target())
            )),
        )?;

        match request.target() {
            CheckpointSelector::Named(name) => {
                self.block_on(self.memories.checkout_tag(name.as_str()))?;
            }
            CheckpointSelector::Version(version) => {
                self.block_on(self.memories.checkout(version.value()))?;
            }
        }

        self.block_on(self.memories.restore())?;
        self.refresh_memories_table()?;

        let current_version = self.block_on(self.memories.version())?;
        Ok(RestoreResult::new(
            request.target().clone(),
            VersionNumber::new(previous_version),
            VersionNumber::new(restored_version),
            VersionNumber::new(current_version),
            pre_restore_checkpoint,
        ))
    }

    /// Runs explicit store maintenance with conservative defaults.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when optimization or requested pruning fails.
    pub(crate) fn optimize_store(
        &mut self,
        request: &OptimizeRequest,
    ) -> Result<OptimizeResult, LanceDbError> {
        let previous_version = self.block_on(self.memories.version())?;
        let pre_optimize_checkpoint = self.maybe_create_operation_checkpoint(
            request.retention_policy().pre_optimize_checkpoint(),
            "optimize",
            Some("automatic checkpoint before optimize".to_string()),
        )?;

        let compact_stats = self.block_on(self.memories.optimize(OptimizeAction::Compact {
            options: CompactionOptions::default(),
            remap_options: None,
        }))?;
        match self.runtime.block_on(
            self.memories
                .optimize(OptimizeAction::Index(OptimizeOptions::default())),
        ) {
            Ok(_)
            | Err(lancedb::Error::NotSupported { .. } | lancedb::Error::IndexNotFound { .. }) => {}
            Err(error) => return Err(LanceDbError::from(error)),
        }

        let prune_stats = if request.prune_old_versions() {
            if request.retention_policy().cleanup_mode() != CleanupMode::AllowPrune {
                return Err(LanceDbError::CleanupNotAllowed);
            }

            Some(self.block_on(self.memories.optimize(OptimizeAction::Prune {
                older_than: Some(lancedb::table::Duration::days(i64::from(
                    request.retention_policy().minimum_age_days(),
                ))),
                delete_unverified: Some(request.retention_policy().delete_unverified()),
                error_if_tagged_old_versions: Some(
                    request.retention_policy().error_if_tagged_old_versions(),
                ),
            }))?)
        } else {
            None
        };

        let current_version = self.block_on(self.memories.version())?;
        let (pruned_versions, bytes_removed) = prune_stats
            .and_then(|stats| stats.prune)
            .map_or((0, 0), |stats| (stats.old_versions, stats.bytes_removed));

        Ok(OptimizeResult::new(
            VersionNumber::new(previous_version),
            VersionNumber::new(current_version),
            pre_optimize_checkpoint,
            compact_stats.compaction.is_some(),
            pruned_versions,
            bytes_removed,
        ))
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
        let indices: Vec<_> = self.block_on(self.memories.list_indices())?;
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

    fn table_uri(&self, table: &Table) -> Result<String, LanceDbError> {
        self.block_on(table.uri())
    }

    fn load_lance_dataset(&self, uri: &str) -> Result<lance::dataset::Dataset, LanceDbError> {
        self.block_on_lance(LanceDatasetBuilder::from_uri(uri).load())
    }

    fn load_branch_dataset(
        &self,
        uri: &str,
        branch: &BranchName,
    ) -> Result<lance::dataset::Dataset, LanceDbError> {
        self.block_on_lance(
            LanceDatasetBuilder::from_uri(uri)
                .with_branch(branch.as_str(), None)
                .load(),
        )
    }

    fn open_branch_memories_table(&self, branch: &BranchName) -> Result<Table, LanceDbError> {
        let uri = self.table_uri(&self.memories)?;
        let dataset = self.load_branch_dataset(&uri, branch)?;
        // `with_branch()` resolves the branch to its branch-specific dataset
        // location, so opening the table at that resolved URI preserves branch
        // isolation for subsequent reads and writes.
        let native = self.block_on(NativeTable::open(dataset.uri()))?;
        let inner: Arc<dyn BaseTable> = Arc::new(native);
        Ok(Table::from(inner))
    }

    fn force_delete_branch_internal(&self, name: &BranchName) -> Result<(), LanceDbError> {
        let memories_uri = self.table_uri(&self.memories)?;
        let mut dataset = self.load_lance_dataset(&memories_uri)?;
        self.block_on_lance(dataset.force_delete_branch(name.as_str()))?;
        Ok(())
    }

    fn insert_memory_record(
        &self,
        table: &Table,
        record: &MemoryRecord,
    ) -> Result<(), LanceDbError> {
        self.block_on_backend(insert_memory_record_async(table, record))
    }

    fn branch_record_from_contents(
        name: String,
        contents: &BranchContents,
    ) -> Result<BranchRecord, LanceDbError> {
        let created_at = UNIX_EPOCH
            .checked_add(Duration::from_secs(contents.create_at))
            .ok_or(LanceDbError::InvalidTimestamp {
                field: "branch_created_at",
            })?;

        Ok(BranchRecord::new(
            BranchName::new(name)?,
            VersionNumber::new(contents.parent_version),
            RecordedAt::new(created_at),
            // Lance exposes branch lineage metadata but not a richer lifecycle
            // state, so visible branches are surfaced as active here.
            BranchStatus::Active,
        ))
    }

    fn next_import_stage_branch_name(&self) -> Result<BranchName, LanceDbError> {
        let current_version = self.block_on(self.memories.version())?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| LanceDbError::InvalidTimestamp {
                field: "import_stage_branch_name",
            })?
            .as_secs();

        Ok(BranchName::new(format!(
            "import-stage-v{current_version}-{timestamp}"
        ))?)
    }

    fn clone_destination_uri(destination: &Path, table_name: &str) -> String {
        destination
            .join(format!("{table_name}.lance"))
            .to_string_lossy()
            .into_owned()
    }

    fn deep_copy_dataset(source_uri: &str, destination_uri: &str) -> Result<(), LanceDbError> {
        fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), LanceDbError> {
            std::fs::create_dir_all(destination)?;

            for entry in std::fs::read_dir(source)? {
                let entry = entry?;
                let source_path = entry.path();
                let destination_path = destination.join(entry.file_name());
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    copy_dir_recursive(&source_path, &destination_path)?;
                } else {
                    std::fs::copy(&source_path, &destination_path)?;
                }
            }

            Ok(())
        }

        copy_dir_recursive(Path::new(source_uri), Path::new(destination_uri))
    }

    fn clone_table_dataset(
        &self,
        source_uri: &str,
        destination_uri: &str,
        kind: CloneKind,
    ) -> Result<(), LanceDbError> {
        let mut dataset = self.load_lance_dataset(source_uri)?;
        match kind {
            CloneKind::Shallow => {
                self.block_on_lance(dataset.shallow_clone(
                    destination_uri,
                    dataset.manifest.version,
                    None,
                ))?;
            }
            CloneKind::Deep => {
                Self::deep_copy_dataset(source_uri, destination_uri)?;
            }
        }
        Ok(())
    }

    fn clone_store_internal(
        &self,
        destination: &Path,
        kind: CloneKind,
    ) -> Result<CloneInfo, LanceDbError> {
        if destination == self.path() {
            return Err(LanceDbError::InvalidStorePath {
                path: destination.to_path_buf(),
                details: "clone destination must differ from the source store",
            });
        }

        std::fs::create_dir_all(destination)?;

        let memories_uri = self.table_uri(&self.memories)?;
        let checkpoints_uri = self.table_uri(&self.checkpoints)?;
        let schema_metadata_uri = self.table_uri(&self.schema_metadata)?;

        let cloned_memories_uri = Self::clone_destination_uri(destination, MEMORIES_TABLE);

        self.clone_table_dataset(&memories_uri, &cloned_memories_uri, kind)?;
        self.clone_table_dataset(
            &checkpoints_uri,
            &Self::clone_destination_uri(destination, CHECKPOINTS_TABLE),
            kind,
        )?;
        self.clone_table_dataset(
            &schema_metadata_uri,
            &Self::clone_destination_uri(destination, SCHEMA_METADATA_TABLE),
            kind,
        )?;

        let cloned_backend = Self::open(destination)?;
        let versions: Vec<_> = cloned_backend.block_on(cloned_backend.memories.list_versions())?;
        let version_count = versions.len() as u64;
        Ok(CloneInfo::new(
            destination.to_path_buf(),
            version_count,
            kind,
        ))
    }

    fn stage_import_records(
        &self,
        source_table: &Table,
        branch_table: &Table,
    ) -> Result<u64, LanceDbError> {
        self.block_on_backend(async {
            let mut staged_records = 0_u64;
            let stream: SendableRecordBatchStream = source_table
                .query()
                .select(Select::Columns(vec![PAYLOAD_COLUMN.to_owned()]))
                .execute()
                .await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;

            for batch in batches {
                let Some(array): Option<&StringArray> = batch
                    .column_by_name(PAYLOAD_COLUMN)
                    .and_then(|column: &Arc<dyn Array>| {
                        column.as_any().downcast_ref::<StringArray>()
                    })
                else {
                    return Err(LanceDbError::InvalidData {
                        field: PAYLOAD_COLUMN,
                        details: "expected Utf8 payload column".to_owned(),
                    });
                };

                for index in 0..array.len() {
                    if array.is_null(index) {
                        return Err(LanceDbError::InvalidData {
                            field: PAYLOAD_COLUMN,
                            details: "null payload value".to_owned(),
                        });
                    }

                    let record = serde_json::from_str::<MemoryRecord>(array.value(index))?;
                    if table_contains_memory_id_async(branch_table, record.id()).await? {
                        continue;
                    }

                    insert_memory_record_async(branch_table, &record).await?;
                    staged_records += 1;
                }
            }

            Ok(staged_records)
        })
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

            let stream: SendableRecordBatchStream = query.execute().await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut payloads = Vec::new();
        for batch in batches {
            let batch: RecordBatch = batch;
            let column: &Arc<dyn Array> = batch.column(0);
            let Some(array): Option<&StringArray> = column.as_any().downcast_ref::<StringArray>()
            else {
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

    fn block_on_backend<F, T>(&self, future: F) -> Result<T, LanceDbError>
    where
        F: Future<Output = Result<T, LanceDbError>>,
    {
        if Handle::try_current().is_ok() {
            return Err(LanceDbError::UnsupportedCallerContext);
        }

        self.runtime.block_on(future)
    }

    fn block_on_lance<F, T>(&self, future: F) -> Result<T, LanceDbError>
    where
        F: Future<Output = Result<T, lance::Error>>,
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
            BackendCapability::Pinning,
            BackendCapability::Search,
            BackendCapability::History,
            BackendCapability::Restore,
            BackendCapability::Checkpoints,
            BackendCapability::Optimize,
            BackendCapability::BranchCreate,
            BackendCapability::BranchList,
            BackendCapability::ImportStaging,
            BackendCapability::ShallowClone,
            BackendCapability::DeepClone,
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

        self.insert_memory_record(&self.memories, &record)?;
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

impl PinningBackend for LanceDbBackend {
    fn pin(&mut self, id: &MemoryId, reason: &str) -> Result<Option<MemoryRecord>, Self::Error> {
        let Some(existing) = self.get(id)? else {
            return Ok(None);
        };

        let updated = rebuild_memory_record(&existing)?
            .pin_state(PinState::pinned(reason)?)
            .updated_at(RecordedAt::now())
            .build()?;
        self.update_memory_record(&updated)?;
        Ok(Some(updated))
    }

    fn unpin(&mut self, id: &MemoryId) -> Result<Option<MemoryRecord>, Self::Error> {
        let Some(existing) = self.get(id)? else {
            return Ok(None);
        };

        let updated = rebuild_memory_record(&existing)?
            .pin_state(PinState::NotPinned)
            .updated_at(RecordedAt::now())
            .build()?;
        self.update_memory_record(&updated)?;
        Ok(Some(updated))
    }
}

impl RecallBackend for LanceDbBackend {
    fn recall(&self, query: &RecallQuery) -> Result<RecallResult, Self::Error> {
        let limit = usize::from(query.limit().value());
        let pinned_limit = limit.min(PINNED_CONTEXT_HARD_CAP);
        let mut remaining = limit;

        let pinned_context = if query.disclosure_depth() == DisclosureDepth::SummaryOnly {
            Vec::new()
        } else {
            let layer_limit = pinned_limit.min(remaining);
            let entries = self.recall_layer_entries(
                query,
                combine_filters([
                    query
                        .scope()
                        .map(|scope| string_filter("scope_id", scope.as_str())),
                    Some("pinned = true".to_owned()),
                ]),
                layer_limit,
                RecallLayer::PinnedContext,
            )?;
            remaining = remaining.saturating_sub(entries.len());
            entries
        };

        let summaries = if remaining == 0 {
            Vec::new()
        } else {
            let entries = self.recall_layer_entries(
                query,
                combine_filters([
                    query
                        .scope()
                        .map(|scope| string_filter("scope_id", scope.as_str())),
                    Some(string_filter("kind", memory_kind_name(MemoryKind::Summary))),
                    (query.disclosure_depth() != DisclosureDepth::SummaryOnly)
                        .then_some("pinned = false".to_owned()),
                ]),
                remaining,
                RecallLayer::Summary,
            )?;
            remaining = remaining.saturating_sub(entries.len());
            entries
        };

        let archival = if query.disclosure_depth() == DisclosureDepth::Full && remaining > 0 {
            self.recall_layer_entries(
                query,
                combine_filters([
                    query
                        .scope()
                        .map(|scope| string_filter("scope_id", scope.as_str())),
                    Some("pinned = false".to_owned()),
                    Some(format!(
                        "kind != {}",
                        sql_string_literal(memory_kind_name(MemoryKind::Summary))
                    )),
                ]),
                remaining,
                RecallLayer::Archival,
            )?
        } else {
            Vec::new()
        };

        Ok(RecallResult::new(
            query.disclosure_depth(),
            pinned_context,
            summaries,
            archival,
        ))
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

        let versions: Vec<_> = self.block_on(self.memories.list_versions())?;
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

        records.sort_by_key(|record: &VersionRecord| std::cmp::Reverse(record.version().value()));
        records.truncate(usize::from(query.limit().value()));
        Ok(records)
    }
}

impl RestoreBackend for LanceDbBackend {
    fn restore(&mut self, request: &RestoreRequest) -> Result<RestoreResult, Self::Error> {
        self.restore_store(request)
    }
}

impl CheckpointBackend for LanceDbBackend {
    fn checkpoint(&mut self, request: &CheckpointRequest) -> Result<Checkpoint, Self::Error> {
        let version: u64 = self.block_on(self.memories.version())?;
        self.create_checkpoint_at_version(
            request.name(),
            version,
            request.description().map(ToOwned::to_owned),
        )
    }

    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, Self::Error> {
        let payloads = self.query_payloads(&self.checkpoints, None, None, None)?;

        payloads
            .into_iter()
            .map(|payload| serde_json::from_str::<Checkpoint>(&payload).map_err(Into::into))
            .collect()
    }
}

impl OptimizeBackend for LanceDbBackend {
    fn optimize(&mut self, request: &OptimizeRequest) -> Result<OptimizeResult, Self::Error> {
        self.optimize_store(request)
    }
}

impl AdvancedStorageBackend for LanceDbBackend {
    fn create_branch(&mut self, request: &BranchRequest) -> Result<BranchRecord, Self::Error> {
        let memories_uri = self.table_uri(&self.memories)?;
        let mut dataset = self.load_lance_dataset(&memories_uri)?;
        let base_version = request
            .base_version()
            .map_or(dataset.manifest.version, VersionNumber::value);

        self.block_on_lance(dataset.create_branch(request.name().as_str(), base_version, None))?;
        let mut all_branches: HashMap<String, BranchContents> =
            self.block_on_lance(dataset.list_branches())?;
        let contents = all_branches
            .remove(request.name().as_str())
            .ok_or_else(|| LanceDbError::BranchNotFound {
                name: request.name().as_str().to_owned(),
            })?;

        Self::branch_record_from_contents(request.name().as_str().to_owned(), &contents)
    }

    fn list_branches(&self) -> Result<BranchListResult, Self::Error> {
        let memories_uri = self.table_uri(&self.memories)?;
        let dataset = self.load_lance_dataset(&memories_uri)?;
        let all_branches: HashMap<String, BranchContents> =
            self.block_on_lance(dataset.list_branches())?;
        let mut branches = all_branches
            .into_iter()
            .map(|(name, contents)| Self::branch_record_from_contents(name, &contents))
            .collect::<Result<Vec<_>, _>>()?;

        branches.sort_by(|left: &BranchRecord, right: &BranchRecord| left.name().cmp(right.name()));
        Ok(BranchListResult::new(branches))
    }

    fn delete_branch(&mut self, name: &BranchName) -> Result<(), Self::Error> {
        let memories_uri = self.table_uri(&self.memories)?;
        let mut dataset = self.load_lance_dataset(&memories_uri)?;
        let branches: HashMap<String, BranchContents> =
            self.block_on_lance(dataset.list_branches())?;
        let Some(contents) = branches.get(name.as_str()) else {
            return Err(LanceDbError::BranchNotFound {
                name: name.as_str().to_owned(),
            });
        };

        let branch_dataset = self.load_branch_dataset(&memories_uri, name)?;
        let latest_branch_version = self.block_on_lance(branch_dataset.latest_version_id())?;
        if latest_branch_version > contents.parent_version {
            return Err(LanceDbError::BranchHasChanges {
                name: name.as_str().to_owned(),
            });
        }

        self.block_on_lance(dataset.delete_branch(name.as_str()))?;
        Ok(())
    }

    fn stage_import(
        &mut self,
        request: &ImportStageRequest,
    ) -> Result<ImportStageResult, Self::Error> {
        let source_backend = Self::open(request.source_path())?;

        let branch_name = request
            .branch_name()
            .cloned()
            .unwrap_or(self.next_import_stage_branch_name()?);
        let branch_record = self.create_branch(&BranchRequest::new(branch_name.clone()))?;
        let staged_records = match (|| {
            let branch_table = self.open_branch_memories_table(branch_record.name())?;
            self.stage_import_records(&source_backend.memories, &branch_table)
        })() {
            Ok(staged_records) => staged_records,
            Err(error) => {
                let _ = self.force_delete_branch_internal(branch_record.name());
                return Err(error);
            }
        };

        Ok(ImportStageResult::new(
            branch_name,
            staged_records,
            staged_records > 0,
        ))
    }

    fn shallow_clone(&self, destination: &Path) -> Result<CloneInfo, Self::Error> {
        self.clone_store_internal(destination, CloneKind::Shallow)
    }

    fn deep_clone(&self, destination: &Path) -> Result<CloneInfo, Self::Error> {
        self.clone_store_internal(destination, CloneKind::Deep)
    }
}

impl LanceDbBackend {
    fn resolve_version_selector(&self, selector: &CheckpointSelector) -> Result<u64, LanceDbError> {
        match selector {
            CheckpointSelector::Named(name) => self
                .list_checkpoints()?
                .into_iter()
                .find(|checkpoint| checkpoint.name() == name)
                .map(|checkpoint| checkpoint.version().value())
                .ok_or_else(|| LanceDbError::CheckpointNotFound {
                    name: name.as_str().to_owned(),
                }),
            CheckpointSelector::Version(version) => self.resolve_raw_version(*version),
        }
    }

    fn resolve_raw_version(&self, version: VersionNumber) -> Result<u64, LanceDbError> {
        let requested = version.value();
        let table = self.open_latest_memories_table()?;

        match self.runtime.block_on(table.checkout(requested)) {
            Ok(()) => Ok(requested),
            Err(error) if lancedb_error_indicates_missing_version(&error) => {
                Err(LanceDbError::VersionNotFound { version: requested })
            }
            Err(error) => Err(LanceDbError::from(error)),
        }
    }

    fn open_latest_memories_table(&self) -> Result<Table, LanceDbError> {
        let uri = self.path.to_string_lossy().to_string();
        let connection: Connection = self.block_on(connect(&uri).execute())?;
        self.block_on(connection.open_table(MEMORIES_TABLE).execute())
    }

    fn refresh_memories_table(&mut self) -> Result<(), LanceDbError> {
        self.memories = self.open_latest_memories_table()?;
        Ok(())
    }

    fn maybe_create_operation_checkpoint(
        &mut self,
        policy: &PreOperationCheckpointPolicy,
        operation: &'static str,
        description: Option<String>,
    ) -> Result<Option<Checkpoint>, LanceDbError> {
        match policy {
            PreOperationCheckpointPolicy::Skip => Ok(None),
            PreOperationCheckpointPolicy::RequireCallerProvided => {
                Err(LanceDbError::CallerCheckpointRequired { operation })
            }
            PreOperationCheckpointPolicy::AutoCreate { prefix } => {
                let current_version = self.block_on(self.memories.version())?;
                if let Some(existing) = self.checkpoint_for_version(current_version)? {
                    return Ok(Some(existing));
                }

                let name = CheckpointName::new(format!("{prefix}-v{current_version}"))?;
                let checkpoint =
                    self.create_checkpoint_at_version(&name, current_version, description)?;
                Ok(Some(checkpoint))
            }
        }
    }

    fn checkpoint_for_version(&self, version: u64) -> Result<Option<Checkpoint>, LanceDbError> {
        Ok(self
            .list_checkpoints()?
            .into_iter()
            .find(|checkpoint| checkpoint.version().value() == version))
    }

    fn create_checkpoint_at_version(
        &mut self,
        name: &CheckpointName,
        version: u64,
        description: Option<String>,
    ) -> Result<Checkpoint, LanceDbError> {
        let existing_checkpoints = self.list_checkpoints()?;
        if existing_checkpoints
            .iter()
            .any(|checkpoint| checkpoint.name() == name)
        {
            return Err(LanceDbError::DuplicateCheckpointName {
                name: name.as_str().to_owned(),
            });
        }

        if existing_checkpoints
            .iter()
            .any(|checkpoint| checkpoint.version().value() == version)
        {
            return Err(LanceDbError::DuplicateCheckpointVersion { version });
        }

        let created_at = RecordedAt::now();
        let checkpoint = Checkpoint::new_at(
            name.clone(),
            VersionNumber::new(version),
            created_at,
            description,
        );

        let (created_secs, created_nanos) =
            system_time_to_parts(created_at.value(), "checkpoint_created_at")?;
        let payload_json = serde_json::to_string(&checkpoint)?;
        let schema = checkpoints_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(StringArray::from(vec![Some(name.as_str())])),
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

        let mut tags: Box<dyn lancedb::table::Tags> = self.block_on(self.memories.tags())?;
        self.block_on(tags.create(name.as_str(), version))?;

        if let Err(error) = self.block_on(self.checkpoints.add(reader).execute()) {
            let _ = self.block_on(tags.delete(name.as_str()));
            return Err(error);
        }

        Ok(checkpoint)
    }

    fn recall_layer_entries(
        &self,
        query: &RecallQuery,
        filter: Option<String>,
        limit: usize,
        layer: RecallLayer,
    ) -> Result<Vec<RecallEntry>, LanceDbError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let payloads = self.query_payloads(
            &self.memories,
            filter,
            Some(recall_fetch_limit(limit)),
            query.text().map(ToOwned::to_owned),
        )?;
        let mut records = decode_memory_records(payloads)?;
        sort_recall_records(&mut records);
        let explain_context = RecallExplainContext::from_records(&records);

        Ok(records
            .into_iter()
            .take(limit)
            .map(|record| recall_entry(record, layer, query, &explain_context))
            .collect())
    }

    fn update_memory_record(&mut self, record: &MemoryRecord) -> Result<(), LanceDbError> {
        let payload_json = serde_json::to_string(record)?;
        let (updated_secs, updated_nanos) =
            system_time_to_parts(record.updated_at().value(), "updated_at")?;

        self.block_on(
            self.memories
                .update()
                .only_if(string_filter("id", record.id().as_str()))
                .column(
                    "pinned",
                    if record.pin_state().is_pinned() {
                        "true"
                    } else {
                        "false"
                    },
                )
                .column("updated_at_secs", updated_secs.to_string())
                .column("updated_at_nanos", updated_nanos.to_string())
                .column(PAYLOAD_COLUMN, sql_string_literal(&payload_json))
                .execute(),
        )?;

        Ok(())
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
        let versions: Vec<_> = self.block_on(self.memories.list_versions())?;
        let version_count = versions.len() as u64;
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
    format!("{column} = '{}'", sql_escape(value))
}

fn combine_filters(filters: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    let filters = filters.into_iter().flatten().collect::<Vec<_>>();
    if filters.is_empty() {
        None
    } else {
        Some(filters.join(" AND "))
    }
}

fn decode_memory_records(payloads: Vec<String>) -> Result<Vec<MemoryRecord>, LanceDbError> {
    payloads
        .into_iter()
        .map(|payload| serde_json::from_str::<MemoryRecord>(&payload).map_err(Into::into))
        .collect()
}

async fn table_contains_memory_id_async(
    table: &Table,
    id: &MemoryId,
) -> Result<bool, LanceDbError> {
    let stream: SendableRecordBatchStream = table
        .query()
        .select(Select::Columns(vec![PAYLOAD_COLUMN.to_owned()]))
        .only_if(string_filter("id", id.as_str()))
        .limit(1)
        .execute()
        .await?;
    let batches: Vec<RecordBatch> = stream.try_collect().await?;

    Ok(!batches.is_empty())
}

fn memory_record_batch(record: &MemoryRecord) -> Result<(Arc<Schema>, RecordBatch), LanceDbError> {
    let payload_json = serde_json::to_string(record)?;
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

    Ok((schema, batch))
}

async fn insert_memory_record_async(
    table: &Table,
    record: &MemoryRecord,
) -> Result<(), LanceDbError> {
    let (schema, batch) = memory_record_batch(record)?;
    let reader = Box::new(RecordBatchIterator::new(
        vec![Ok(batch)].into_iter(),
        schema,
    ));
    table.add(reader).execute().await?;
    Ok(())
}

fn sort_memory_records(records: &mut [MemoryRecord]) {
    records.sort_by(|left, right| {
        right
            .updated_at()
            .value()
            .cmp(&left.updated_at().value())
            .then_with(|| left.id().as_str().cmp(right.id().as_str()))
    });
}

fn sort_recall_records(records: &mut [MemoryRecord]) {
    records.sort_by(|left, right| {
        recall_rank_key(right)
            .cmp(&recall_rank_key(left))
            .then_with(|| left.id().as_str().cmp(right.id().as_str()))
    });
}

fn recall_rank_key(record: &MemoryRecord) -> (u8, u8, u8, SystemTime) {
    (
        u8::from(record.pin_state().is_pinned()),
        u8::from(matches!(record.kind(), MemoryKind::Summary)),
        record.importance().value(),
        record.updated_at().value(),
    )
}

struct RecallExplainContext {
    newest_updated_at: Option<SystemTime>,
    importance_floor: u8,
}

impl RecallExplainContext {
    fn from_records(records: &[MemoryRecord]) -> Self {
        Self {
            newest_updated_at: records
                .iter()
                .map(|record| record.updated_at().value())
                .max(),
            importance_floor: Importance::default().value(),
        }
    }

    fn receives_importance_boost(&self, record: &MemoryRecord) -> bool {
        record.importance().value() > self.importance_floor
    }

    fn receives_recency_boost(&self, record: &MemoryRecord) -> bool {
        self.newest_updated_at
            .is_some_and(|newest| record.updated_at().value() == newest)
    }
}

fn recall_entry(
    record: MemoryRecord,
    layer: RecallLayer,
    query: &RecallQuery,
    explain_context: &RecallExplainContext,
) -> RecallEntry {
    let mut reasons = Vec::new();
    if record.pin_state().is_pinned() {
        reasons.push(RecallReason::Pinned);
    }
    if query.scope().is_some() {
        reasons.push(RecallReason::ScopeFilter);
    }
    if query.text().is_some() {
        reasons.push(RecallReason::TextMatch);
    }
    if matches!(record.kind(), MemoryKind::Summary) {
        reasons.push(RecallReason::SummaryKind);
    }
    if explain_context.receives_importance_boost(&record) {
        reasons.push(RecallReason::ImportanceBoost);
    }
    if explain_context.receives_recency_boost(&record) {
        reasons.push(RecallReason::RecencyBoost);
    }
    if layer == RecallLayer::Archival {
        reasons.push(RecallReason::ArchivalExpansion);
    }

    RecallEntry::new(record, RecallExplanation::new(layer, reasons))
}

fn rebuild_memory_record(
    record: &MemoryRecord,
) -> Result<mnemix_core::MemoryRecordBuilder, CoreError> {
    let builder = MemoryRecord::builder(
        record.id().clone(),
        record.scope_id().clone(),
        record.kind(),
    )
    .title(record.title())?
    .summary(record.summary())?
    .detail(record.detail())?
    .fts_text(record.fts_text())?
    .importance(record.importance())
    .confidence(record.confidence())
    .created_at(record.created_at())
    .updated_at(record.updated_at())
    .pin_state(record.pin_state().clone())
    .metadata(record.metadata().clone());

    let builder = record
        .tags()
        .iter()
        .cloned()
        .fold(builder, mnemix_core::MemoryRecordBuilder::add_tag);
    let builder = record
        .entities()
        .iter()
        .cloned()
        .fold(builder, mnemix_core::MemoryRecordBuilder::add_entity);
    let builder = if let Some(value) = record.source_session_id() {
        builder.source_session_id(value.clone())
    } else {
        builder
    };
    let builder = if let Some(value) = record.source_tool() {
        builder.source_tool(value.clone())
    } else {
        builder
    };
    let builder = if let Some(value) = record.source_ref() {
        builder.source_ref(value.clone())
    } else {
        builder
    };

    Ok(builder)
}

fn sql_string_literal(value: &str) -> String {
    format!("'{}'", sql_escape(value))
}

fn sql_escape(value: &str) -> String {
    value.replace('\'', "''")
}

fn lancedb_error_indicates_missing_version(error: &lancedb::Error) -> bool {
    match error {
        lancedb::Error::InvalidInput { message }
        | lancedb::Error::NotSupported { message }
        | lancedb::Error::Other { message, .. } => {
            let message = message.to_lowercase();
            message.contains("version")
                && (message.contains("not found")
                    || message.contains("does not exist")
                    || message.contains("no version"))
        }
        lancedb::Error::Lance { source } => {
            let message = source.to_string().to_lowercase();
            message.contains("version")
                && (message.contains("not found")
                    || message.contains("does not exist")
                    || message.contains("no version"))
        }
        _ => false,
    }
}

fn checkpoint_selector_label(selector: &CheckpointSelector) -> String {
    match selector {
        CheckpointSelector::Named(name) => format!("checkpoint {}", name.as_str()),
        CheckpointSelector::Version(version) => format!("version {}", version.value()),
    }
}

fn recall_fetch_limit(limit: usize) -> usize {
    limit.saturating_mul(RECALL_FETCH_MULTIPLIER)
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

    use mnemix_core::{
        BranchName, BranchRequest, CheckpointName, CheckpointSelector, CleanupMode, CloneKind,
        Confidence, DisclosureDepth, ImportStageRequest, Importance, OptimizeRequest,
        PreOperationCheckpointPolicy, QueryLimit, RecallQuery, RestoreRequest, RetentionPolicy,
        ScopeId, SearchQuery, StatsQuery, TagName,
        traits::{
            AdvancedStorageBackend, CheckpointBackend, HistoryBackend, MemoryRepository,
            OptimizeBackend, PinningBackend, RecallBackend, RestoreBackend, StatsBackend,
            StorageBackend,
        },
    };
    use tempfile::TempDir;

    use super::*;

    #[derive(Clone, Copy)]
    struct MemoryFixture<'a> {
        id: &'a str,
        scope: &'a str,
        kind: MemoryKind,
        title: &'a str,
        summary: &'a str,
        detail: &'a str,
        importance: u8,
        pin_reason: Option<&'a str>,
        updated_at_secs: u64,
    }

    fn build_memory(id: &str, scope: &str, title: &str, detail: &str) -> MemoryRecord {
        build_memory_with(MemoryFixture {
            id,
            scope,
            kind: MemoryKind::Decision,
            title,
            summary: "summary",
            detail,
            importance: 90,
            pin_reason: None,
            updated_at_secs: 0,
        })
    }

    fn build_memory_with(fixture: MemoryFixture<'_>) -> MemoryRecord {
        let created_at =
            RecordedAt::new(UNIX_EPOCH + std::time::Duration::from_secs(fixture.updated_at_secs));

        MemoryRecord::builder(
            MemoryId::try_from(fixture.id).expect("valid id"),
            ScopeId::try_from(fixture.scope).expect("valid scope"),
            fixture.kind,
        )
        .title(fixture.title)
        .expect("valid title")
        .summary(fixture.summary)
        .expect("valid summary")
        .detail(fixture.detail)
        .expect("valid detail")
        .importance(Importance::new(fixture.importance).expect("valid importance"))
        .confidence(Confidence::new(95).expect("valid confidence"))
        .created_at(created_at)
        .updated_at(created_at)
        .add_tag(TagName::try_from("milestone-2").expect("valid tag"))
        .metadata(BTreeMap::from([(
            "owner".to_string(),
            "backend".to_string(),
        )]))
        .pin_state(match fixture.pin_reason {
            Some(reason) => PinState::pinned(reason).expect("valid pin reason"),
            None => PinState::NotPinned,
        })
        .build()
        .expect("memory should build")
    }

    fn new_backend() -> (TempDir, LanceDbBackend) {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        (temp_dir, backend)
    }

    fn branch_table(backend: &LanceDbBackend, branch: &BranchName) -> Table {
        backend
            .open_branch_memories_table(branch)
            .expect("branch table should open")
    }

    fn branch_memory_ids(backend: &LanceDbBackend, branch: &BranchName) -> Vec<String> {
        let table = branch_table(backend, branch);
        let mut records = decode_memory_records(
            backend
                .query_payloads(&table, None, None, None)
                .expect("branch payloads should query"),
        )
        .expect("branch records should decode");
        sort_memory_records(&mut records);
        records
            .into_iter()
            .map(|record| record.id().as_str().to_owned())
            .collect()
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
    fn backend_advertises_pinning_capability() {
        let (_temp_dir, backend) = new_backend();

        assert!(backend.capabilities().supports_pinning());
    }

    #[test]
    fn backend_advertises_advanced_storage_capabilities() {
        let (_temp_dir, backend) = new_backend();

        assert!(backend.capabilities().supports_branch_create());
        assert!(backend.capabilities().supports_branch_list());
        assert!(backend.capabilities().supports_import_staging());
        assert!(backend.capabilities().supports_shallow_clone());
        assert!(backend.capabilities().supports_deep_clone());
    }

    #[test]
    fn remember_get_search_and_delete_memory() {
        let (_temp_dir, mut backend) = new_backend();
        let memory = build_memory(
            "memory:one",
            "repo:mnemix",
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
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
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
    fn recall_returns_layered_progressive_disclosure_results() {
        let (_temp_dir, mut backend) = new_backend();
        let scope = "repo:mnemix";
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:pinned",
                scope,
                kind: MemoryKind::Decision,
                title: "Pinned working agreement",
                summary: "Pinned context summary",
                detail: "Always load this before planning work.",
                importance: 95,
                pin_reason: Some("critical context"),
                updated_at_secs: 30,
            }))
            .expect("pinned memory should store");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:summary",
                scope,
                kind: MemoryKind::Summary,
                title: "Repository summary",
                summary: "Compact repo context summary",
                detail: "Summary-first recall should keep this context visible.",
                importance: 90,
                pin_reason: None,
                updated_at_secs: 20,
            }))
            .expect("summary memory should store");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:archival",
                scope,
                kind: MemoryKind::Decision,
                title: "Historical note",
                summary: "Older archival memory",
                detail: "Useful when expanding recall to deeper context.",
                importance: 70,
                pin_reason: None,
                updated_at_secs: 10,
            }))
            .expect("archival memory should store");

        let summary_then_pinned = backend
            .recall(
                &RecallQuery::builder()
                    .scope(ScopeId::try_from(scope).expect("valid scope"))
                    .disclosure_depth(DisclosureDepth::SummaryThenPinned)
                    .build()
                    .expect("query should build"),
            )
            .expect("recall should succeed");

        assert_eq!(summary_then_pinned.count(), 2);
        assert_eq!(summary_then_pinned.pinned_context().len(), 1);
        assert_eq!(summary_then_pinned.summaries().len(), 1);
        assert!(summary_then_pinned.archival().is_empty());
        assert_eq!(
            summary_then_pinned.pinned_context()[0]
                .memory()
                .id()
                .as_str(),
            "memory:pinned"
        );
        assert_eq!(
            summary_then_pinned.summaries()[0].explanation().layer(),
            RecallLayer::Summary
        );
        assert!(
            summary_then_pinned.summaries()[0]
                .explanation()
                .reasons()
                .contains(&RecallReason::ScopeFilter)
        );

        let full = backend
            .recall(
                &RecallQuery::builder()
                    .scope(ScopeId::try_from(scope).expect("valid scope"))
                    .disclosure_depth(DisclosureDepth::Full)
                    .build()
                    .expect("query should build"),
            )
            .expect("full recall should succeed");

        assert_eq!(full.archival().len(), 1);
        assert_eq!(full.archival()[0].memory().id().as_str(), "memory:archival");
        assert!(
            full.archival()[0]
                .explanation()
                .reasons()
                .contains(&RecallReason::ArchivalExpansion)
        );
    }

    #[test]
    fn recall_summary_only_keeps_pinned_summaries_visible() {
        let (_temp_dir, mut backend) = new_backend();
        let scope = "repo:mnemix";
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:summary-pinned",
                scope,
                kind: MemoryKind::Summary,
                title: "Pinned summary",
                summary: "Pinned summaries should survive summary-only recall.",
                detail: "Pinned summaries remain visible even when pinned context is suppressed.",
                importance: 90,
                pin_reason: Some("always show this summary"),
                updated_at_secs: 20,
            }))
            .expect("pinned summary should store");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:decision-pinned",
                scope,
                kind: MemoryKind::Decision,
                title: "Pinned decision",
                summary: "Should not appear in summary-only mode.",
                detail: "Pinned non-summaries belong to pinned context only.",
                importance: 95,
                pin_reason: Some("active working set"),
                updated_at_secs: 30,
            }))
            .expect("pinned decision should store");

        let result = backend
            .recall(
                &RecallQuery::builder()
                    .scope(ScopeId::try_from(scope).expect("valid scope"))
                    .disclosure_depth(DisclosureDepth::SummaryOnly)
                    .build()
                    .expect("query should build"),
            )
            .expect("summary-only recall should succeed");

        assert!(result.pinned_context().is_empty());
        assert_eq!(result.summaries().len(), 1);
        assert_eq!(result.count(), 1);
        assert_eq!(
            result.summaries()[0].memory().id().as_str(),
            "memory:summary-pinned"
        );
        assert!(
            result.summaries()[0]
                .explanation()
                .reasons()
                .contains(&RecallReason::Pinned)
        );
        assert!(
            result.summaries()[0]
                .explanation()
                .reasons()
                .contains(&RecallReason::SummaryKind)
        );
    }

    #[test]
    fn recall_explanations_only_report_active_ranking_boosts() {
        let (_temp_dir, mut backend) = new_backend();
        let scope = "repo:mnemix";
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:pinned-recent",
                scope,
                kind: MemoryKind::Decision,
                title: "Newest pinned memory",
                summary: "High-importance pinned memory.",
                detail: "Should receive both ranking boosts.",
                importance: 90,
                pin_reason: Some("critical context"),
                updated_at_secs: 30,
            }))
            .expect("recent pinned memory should store");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:summary-stable",
                scope,
                kind: MemoryKind::Summary,
                title: "Stable summary",
                summary: "Default-importance summary.",
                detail: "Should not claim extra ranking boosts.",
                importance: 50,
                pin_reason: None,
                updated_at_secs: 10,
            }))
            .expect("stable summary should store");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:summary-recent",
                scope,
                kind: MemoryKind::Summary,
                title: "Recent summary",
                summary: "Newer summary in the same layer.",
                detail: "Keeps recency boosting scoped to the newest summary.",
                importance: 50,
                pin_reason: None,
                updated_at_secs: 20,
            }))
            .expect("recent summary should store");

        let result = backend
            .recall(
                &RecallQuery::builder()
                    .scope(ScopeId::try_from(scope).expect("valid scope"))
                    .disclosure_depth(DisclosureDepth::SummaryThenPinned)
                    .build()
                    .expect("query should build"),
            )
            .expect("recall should succeed");

        let pinned_reasons = result.pinned_context()[0].explanation().reasons();
        assert!(pinned_reasons.contains(&RecallReason::ImportanceBoost));
        assert!(pinned_reasons.contains(&RecallReason::RecencyBoost));

        let stable_summary = result
            .summaries()
            .iter()
            .find(|entry| entry.memory().id().as_str() == "memory:summary-stable")
            .expect("stable summary should be present");
        let summary_reasons = stable_summary.explanation().reasons();
        assert!(!summary_reasons.contains(&RecallReason::ImportanceBoost));
        assert!(!summary_reasons.contains(&RecallReason::RecencyBoost));
    }

    #[test]
    fn pin_and_unpin_update_persisted_memory_state() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:pin-target",
                "repo:mnemix",
                "Pin target",
                "Used to validate explicit pinning semantics.",
            ))
            .expect("memory should store");

        let pinned = backend
            .pin(
                &MemoryId::try_from("memory:pin-target").expect("valid id"),
                "keep in active context",
            )
            .expect("pin should succeed")
            .expect("memory should exist");
        assert!(pinned.pin_state().is_pinned());
        assert_eq!(pinned.pin_state().reason(), Some("keep in active context"));

        let fetched = backend
            .get(&MemoryId::try_from("memory:pin-target").expect("valid id"))
            .expect("lookup should succeed")
            .expect("memory should exist");
        assert!(fetched.pin_state().is_pinned());

        let unpinned = backend
            .unpin(&MemoryId::try_from("memory:pin-target").expect("valid id"))
            .expect("unpin should succeed")
            .expect("memory should exist");
        assert!(!unpinned.pin_state().is_pinned());
        assert_eq!(unpinned.pin_state().reason(), None);
    }

    #[test]
    fn open_existing_store_recovers_memories_and_checkpoints() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        {
            let mut backend =
                LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
            let memory = build_memory(
                "memory:checkpoint",
                "repo:mnemix",
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
                "repo:mnemix",
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
                Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
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
                "repo:mnemix",
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

    #[test]
    fn branch_create_list_and_delete_round_trip() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:branch-base",
                "repo:mnemix",
                "Branch base",
                "Create a branch from this state.",
            ))
            .expect("memory should store");

        let branch = backend
            .create_branch(&BranchRequest::new(
                BranchName::try_from("experiments/imports").expect("valid branch name"),
            ))
            .expect("branch should be created");
        let branches = backend.list_branches().expect("branches should list");

        assert_eq!(branch.name().as_str(), "experiments/imports");
        assert_eq!(branches.len(), 1);
        assert_eq!(
            branches.branches()[0].name().as_str(),
            "experiments/imports"
        );

        backend
            .delete_branch(branch.name())
            .expect("unchanged branch should delete");
        assert!(
            backend
                .list_branches()
                .expect("branches should list")
                .is_empty()
        );
    }

    #[test]
    fn delete_branch_rejects_staged_changes() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:branch-delete-base",
                "repo:mnemix",
                "Branch delete base",
                "Used to validate branch delete protection.",
            ))
            .expect("memory should store");
        let branch = backend
            .create_branch(&BranchRequest::new(
                BranchName::try_from("experiments/dirty-branch").expect("valid branch name"),
            ))
            .expect("branch should be created");

        let branch_table = branch_table(&backend, branch.name());
        backend
            .insert_memory_record(
                &branch_table,
                &build_memory(
                    "memory:branch-only",
                    "repo:mnemix",
                    "Branch only",
                    "Exists only on the experimental branch.",
                ),
            )
            .expect("branch memory should insert");

        let error = backend
            .delete_branch(branch.name())
            .expect_err("dirty branch should not delete");
        assert!(matches!(error, LanceDbError::BranchHasChanges { .. }));
    }

    #[test]
    fn staged_import_does_not_affect_main_branch() {
        let (_source_dir, mut source) = new_backend();
        source
            .remember(build_memory(
                "memory:import-source",
                "repo:import-source",
                "Imported memory",
                "Should appear only on the staging branch.",
            ))
            .expect("source memory should store");

        let (target_dir, mut target) = new_backend();
        target
            .remember(build_memory(
                "memory:target-main",
                "repo:mnemix",
                "Target memory",
                "Should remain on main.",
            ))
            .expect("target memory should store");

        let staged = target
            .stage_import(&ImportStageRequest::new(source.path()).with_branch_name(
                BranchName::try_from("imports/source-a").expect("valid branch name"),
            ))
            .expect("import should stage");

        assert_eq!(staged.branch_name().as_str(), "imports/source-a");
        assert_eq!(staged.staged_records(), 1);
        assert!(staged.ready_to_merge());
        assert!(
            target
                .get(&MemoryId::try_from("memory:import-source").expect("valid id"))
                .expect("lookup should succeed")
                .is_none()
        );

        let staged_ids = branch_memory_ids(&target, staged.branch_name());
        assert!(staged_ids.iter().any(|id| id == "memory:import-source"));
        assert!(staged_ids.iter().any(|id| id == "memory:target-main"));
        assert!(target_dir.path().exists());
    }

    #[test]
    fn staged_import_rejects_missing_source_path() {
        let (_temp_dir, mut backend) = new_backend();
        let missing_path = std::env::temp_dir().join("mnemix-m7-missing-source-store");
        if missing_path.exists() {
            std::fs::remove_dir_all(&missing_path).expect("missing-path fixture should be removed");
        }

        let error = backend
            .stage_import(&ImportStageRequest::new(&missing_path))
            .expect_err("missing import source should fail");

        assert!(matches!(
            error,
            LanceDbError::InvalidStorePath {
                details: "store path does not exist",
                ..
            }
        ));
    }

    #[test]
    fn shallow_clone_produces_independent_workspace() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:clone-shallow",
                "repo:mnemix",
                "Shallow clone memory",
                "Used to validate shallow clone flow.",
            ))
            .expect("memory should store");

        let export_dir = TempDir::new().expect("tempdir should be created");
        let clone_path = export_dir.path().join("shallow-clone-store");
        let info = backend
            .shallow_clone(&clone_path)
            .expect("shallow clone should succeed");

        assert_eq!(info.kind(), CloneKind::Shallow);
        assert!(info.version_count() > 0);

        let mut cloned = LanceDbBackend::open(&clone_path).expect("cloned store should open");
        cloned
            .remember(build_memory(
                "memory:clone-only",
                "repo:mnemix",
                "Clone only",
                "This should not alter the source store.",
            ))
            .expect("clone memory should store");

        assert!(
            backend
                .get(&MemoryId::try_from("memory:clone-only").expect("valid id"))
                .expect("lookup should succeed")
                .is_none()
        );
    }

    #[test]
    fn deep_clone_preserves_version_count() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:clone-deep-a",
                "repo:mnemix",
                "Deep clone A",
                "Create one historical version.",
            ))
            .expect("memory should store");
        backend
            .remember(build_memory(
                "memory:clone-deep-b",
                "repo:mnemix",
                "Deep clone B",
                "Create a second historical version.",
            ))
            .expect("memory should store");

        let export_dir = TempDir::new().expect("tempdir should be created");
        let clone_path = export_dir.path().join("deep-clone-store");
        let info = backend
            .deep_clone(&clone_path)
            .expect("deep clone should succeed");

        assert_eq!(info.kind(), CloneKind::Deep);
        assert!(info.version_count() >= 2);

        let cloned = LanceDbBackend::open(&clone_path).expect("deep clone should open");
        let source_history = backend
            .history(&HistoryQuery::new(
                None,
                QueryLimit::new(20).expect("valid limit"),
            ))
            .expect("source history should load");
        let cloned_history = cloned
            .history(&HistoryQuery::new(
                None,
                QueryLimit::new(20).expect("valid limit"),
            ))
            .expect("cloned history should load");

        assert_eq!(cloned_history.len(), source_history.len());
    }

    #[test]
    fn restore_recreates_a_new_head_from_a_checkpoint() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:restore-baseline",
                "repo:mnemix",
                "Restore baseline",
                "This state should be restored later.",
            ))
            .expect("baseline memory should store");
        let checkpoint = backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("restore-baseline").expect("valid checkpoint name"),
                Some("checkpoint before experimental change".to_string()),
            ))
            .expect("checkpoint should be created");
        backend
            .remember(build_memory(
                "memory:temporary",
                "repo:mnemix",
                "Temporary state",
                "This memory should disappear after restore.",
            ))
            .expect("temporary memory should store");

        let result = backend
            .restore(&RestoreRequest::new(CheckpointSelector::Named(
                CheckpointName::try_from("restore-baseline").expect("valid checkpoint name"),
            )))
            .expect("restore should succeed");

        assert_eq!(result.restored_version(), checkpoint.version());
        assert!(result.current_version().value() > result.previous_version().value());
        assert!(result.pre_restore_checkpoint().is_some());
        assert!(
            backend
                .get(&MemoryId::try_from("memory:restore-baseline").expect("valid id"))
                .expect("lookup should succeed")
                .is_some()
        );
        assert!(
            backend
                .get(&MemoryId::try_from("memory:temporary").expect("valid id"))
                .expect("lookup should succeed")
                .is_none()
        );
    }

    #[test]
    fn optimize_can_create_a_pre_optimize_checkpoint() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:optimize",
                "repo:mnemix",
                "Optimize fixture",
                "Used to validate checkpointed optimize flows.",
            ))
            .expect("memory should store");

        let result = backend
            .optimize(&OptimizeRequest::conservative())
            .expect("optimize should succeed");

        assert!(result.current_version().value() >= result.previous_version().value());
        assert!(result.pre_optimize_checkpoint().is_some());
        assert_eq!(result.pruned_versions(), 0);
    }

    #[test]
    fn restore_by_raw_version_recreates_a_new_head() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:restore-by-version",
                "repo:mnemix",
                "Restore baseline",
                "This state should be restored by version.",
            ))
            .expect("baseline memory should store");
        let checkpoint = backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("restore-by-version").expect("valid checkpoint name"),
                Some("capture a recoverable version".to_string()),
            ))
            .expect("checkpoint should be created");
        backend
            .remember(build_memory(
                "memory:temporary-version-state",
                "repo:mnemix",
                "Temporary state",
                "This memory should disappear after version restore.",
            ))
            .expect("temporary memory should store");

        let result = backend
            .restore(&RestoreRequest::new(CheckpointSelector::Version(
                checkpoint.version(),
            )))
            .expect("restore by version should succeed");

        assert_eq!(result.restored_version(), checkpoint.version());
        assert!(
            backend
                .get(&MemoryId::try_from("memory:temporary-version-state").expect("valid id"))
                .expect("lookup should succeed")
                .is_none()
        );
    }

    #[test]
    fn restore_can_skip_pre_restore_checkpoint_when_policy_requests_it() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:restore-no-safety-tag",
                "repo:mnemix",
                "Restore baseline",
                "This state should restore without an automatic checkpoint.",
            ))
            .expect("baseline memory should store");
        let checkpoint = backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("restore-no-safety-tag").expect("valid checkpoint name"),
                Some("capture baseline state".to_string()),
            ))
            .expect("checkpoint should be created");
        backend
            .remember(build_memory(
                "memory:skip-restore-candidate",
                "repo:mnemix",
                "Temporary state",
                "This memory should disappear after restore.",
            ))
            .expect("temporary memory should store");

        let request = RestoreRequest::new(CheckpointSelector::Named(
            CheckpointName::try_from("restore-no-safety-tag").expect("valid checkpoint name"),
        ))
        .with_retention_policy(
            RetentionPolicy::conservative()
                .with_pre_restore_checkpoint(PreOperationCheckpointPolicy::Skip),
        );
        let result = backend.restore(&request).expect("restore should succeed");

        assert_eq!(result.restored_version(), checkpoint.version());
        assert!(result.pre_restore_checkpoint().is_none());
    }

    #[test]
    fn restore_rejects_unknown_targets() {
        let (_temp_dir, mut backend) = new_backend();

        let missing_checkpoint = backend
            .restore(&RestoreRequest::new(CheckpointSelector::Named(
                CheckpointName::try_from("missing-checkpoint").expect("valid checkpoint name"),
            )))
            .expect_err("missing checkpoint should fail");
        assert!(matches!(
            missing_checkpoint,
            LanceDbError::CheckpointNotFound { .. }
        ));

        let missing_version = backend
            .restore(&RestoreRequest::new(CheckpointSelector::Version(
                VersionNumber::new(999),
            )))
            .expect_err("missing version should fail");
        assert!(matches!(
            missing_version,
            LanceDbError::VersionNotFound { .. }
        ));
    }

    #[test]
    fn optimize_prune_rejects_tagged_old_versions() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:protected",
                "repo:mnemix",
                "Protected history",
                "Tagged versions should block destructive cleanup.",
            ))
            .expect("memory should store");
        backend
            .checkpoint(&CheckpointRequest::new(
                CheckpointName::try_from("protected-history").expect("valid checkpoint name"),
                Some("tagged version must remain recoverable".to_string()),
            ))
            .expect("checkpoint should be created");
        backend
            .remember(build_memory(
                "memory:newer",
                "repo:mnemix",
                "Newer history",
                "Creates a later head so an older tagged version exists.",
            ))
            .expect("newer memory should store");

        let request = OptimizeRequest::new(
            RetentionPolicy::conservative()
                .with_cleanup_mode(CleanupMode::AllowPrune)
                .with_minimum_age_days(0),
        )
        .with_prune_old_versions(true);
        let error = backend
            .optimize(&request)
            .expect_err("tagged old versions should block pruning");

        assert!(error.to_string().to_lowercase().contains("tag"));
    }

    #[test]
    fn optimize_prune_removes_old_untagged_versions() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:prune-baseline",
                "repo:mnemix",
                "Prunable baseline",
                "This version should become prunable.",
            ))
            .expect("baseline memory should store");
        backend
            .remember(build_memory(
                "memory:prune-current",
                "repo:mnemix",
                "Current state",
                "This keeps a newer head version available.",
            ))
            .expect("current memory should store");

        let request = OptimizeRequest::new(
            RetentionPolicy::conservative()
                .with_cleanup_mode(CleanupMode::AllowPrune)
                .with_minimum_age_days(0)
                .with_delete_unverified(true)
                .with_pre_optimize_checkpoint(PreOperationCheckpointPolicy::Skip),
        )
        .with_prune_old_versions(true);
        let result = backend
            .optimize(&request)
            .expect("untagged old versions should be prunable");

        assert!(result.pruned_versions() > 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn sync_backend_calls_fail_inside_async_runtime() {
        let temp_dir = TempDir::new().expect("tempdir should be created");

        let error = LanceDbBackend::init(temp_dir.path())
            .expect_err("sync backend initialization should be rejected");

        assert!(matches!(error, LanceDbError::UnsupportedCallerContext));
    }
}
