//! Local `LanceDB` backend implementation.

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use arrow_array::{
    Array, BooleanArray, FixedSizeListArray, Float32Array, ListArray, RecordBatch,
    RecordBatchIterator, StringArray, UInt32Array, UInt64Array, types::Float32Type,
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
        RecallQuery, RecallReason, RecallResult, RetrievalMode, SearchQuery, StatsQuery,
        StatsSnapshot,
    },
    retention::{CleanupMode, PreOperationCheckpointPolicy},
    traits::{
        AdvancedStorageBackend, BackendCapabilities, BackendCapability, BrowseBackend,
        CheckpointBackend, HistoryBackend, MemoryRepository, OptimizeBackend, PinningBackend,
        RecallBackend, RestoreBackend, StatsBackend, StorageBackend,
    },
};
use thiserror::Error;
use tokio::runtime::{Builder, Handle, Runtime};

const MEMORIES_TABLE: &str = "memories";
const CHECKPOINTS_TABLE: &str = "checkpoints";
const SCHEMA_METADATA_TABLE: &str = "schema_metadata";
const VECTOR_CONFIG_TABLE: &str = "vector_config";
const MEMORY_EMBEDDINGS_TABLE: &str = "memory_embeddings";
const PAYLOAD_COLUMN: &str = "payload_json";
const LEGACY_SCHEMA_VERSION: u64 = 1;
const VECTOR_CONFIG_SCHEMA_VERSION: u64 = 2;
const PERSISTED_EMBEDDING_SCHEMA_VERSION: u64 = 3;
const INDEXABLE_EMBEDDING_SCHEMA_VERSION: u64 = 4;
const SCHEMA_VERSION: u64 = INDEXABLE_EMBEDDING_SCHEMA_VERSION;
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

    /// Indicates vector settings are internally inconsistent or incompatible
    /// with the selected embedding provider.
    #[error("invalid vector settings: {details}")]
    InvalidVectorSettings {
        /// Human-readable settings validation failure details.
        details: String,
    },
}

/// Embedding-provider-specific failures returned by backend-owned providers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum EmbeddingProviderError {
    /// Indicates the provider could not generate an embedding for the input.
    #[error("{message}")]
    Message {
        /// Human-readable provider failure details.
        message: String,
    },
}

/// A backend-owned embedding provider used for vector-enabled stores.
pub trait EmbeddingProvider: Send + Sync {
    /// Returns the stable provider model identifier.
    fn model_id(&self) -> &str;

    /// Returns the embedding dimensions produced by the provider.
    fn dimensions(&self) -> u32;

    /// Embeds a single input string.
    ///
    /// # Errors
    ///
    /// Returns [`EmbeddingProviderError`] when the provider cannot embed the
    /// supplied input.
    fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingProviderError>;
}

/// Store-level vector configuration persisted by the `LanceDB` backend.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct VectorSettings {
    vectors_enabled: bool,
    auto_embed_on_write: bool,
    embedding_model: Option<String>,
    embedding_dimensions: Option<u32>,
}

/// Operational readiness for the current store's persisted vector shape.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorIndexStatus {
    available: bool,
    reason: Option<String>,
}

impl VectorIndexStatus {
    /// Returns `true` when a LanceDB-native vector index can be built and used.
    #[must_use]
    pub const fn available(&self) -> bool {
        self.available
    }

    /// Returns a human-readable explanation when the vector index is unavailable.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Store-level vector runtime and coverage snapshot.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorStatus {
    settings: VectorSettings,
    has_embedding_provider: bool,
    can_embed_on_write: bool,
    semantic_retrieval_available: bool,
    persisted_embedding_storage: bool,
    indexable_embedding_storage: bool,
    total_memories: u64,
    embedded_memories: u64,
    vector_index: VectorIndexStatus,
}

impl VectorStatus {
    /// Returns the store's persisted vector settings.
    #[must_use]
    pub const fn settings(&self) -> &VectorSettings {
        &self.settings
    }

    /// Returns `true` when a runtime embedding provider is attached.
    #[must_use]
    pub const fn has_embedding_provider(&self) -> bool {
        self.has_embedding_provider
    }

    /// Returns `true` when new writes can be embedded immediately.
    #[must_use]
    pub const fn can_embed_on_write(&self) -> bool {
        self.can_embed_on_write
    }

    /// Returns `true` when semantic retrieval can run for this store.
    #[must_use]
    pub const fn semantic_retrieval_available(&self) -> bool {
        self.semantic_retrieval_available
    }

    /// Returns `true` when the current schema can persist embedding values.
    #[must_use]
    pub const fn persisted_embedding_storage(&self) -> bool {
        self.persisted_embedding_storage
    }

    /// Returns `true` when the current store has a fixed-size embedding schema
    /// suitable for future LanceDB-native vector indexing.
    #[must_use]
    pub const fn indexable_embedding_storage(&self) -> bool {
        self.indexable_embedding_storage
    }

    /// Returns the total number of memories stored in the backend.
    #[must_use]
    pub const fn total_memories(&self) -> u64 {
        self.total_memories
    }

    /// Returns the number of memories that currently have persisted embeddings.
    #[must_use]
    pub const fn embedded_memories(&self) -> u64 {
        self.embedded_memories
    }

    /// Returns the current LanceDB-native vector-index readiness snapshot.
    #[must_use]
    pub const fn vector_index(&self) -> &VectorIndexStatus {
        &self.vector_index
    }

    /// Returns whole-percent persisted embedding coverage for the store.
    #[must_use]
    pub fn embedding_coverage_percent(&self) -> u8 {
        if self.total_memories == 0 {
            return 0;
        }

        let percent = (self.embedded_memories.saturating_mul(100)) / self.total_memories;
        percent.min(100) as u8
    }
}

impl VectorSettings {
    /// Returns `true` when vector retrieval is enabled for the store.
    #[must_use]
    pub const fn vectors_enabled(&self) -> bool {
        self.vectors_enabled
    }

    /// Returns `true` when new writes should auto-embed when a provider is
    /// available.
    #[must_use]
    pub const fn auto_embed_on_write(&self) -> bool {
        self.auto_embed_on_write
    }

    /// Returns the configured embedding model identifier, if any.
    #[must_use]
    pub fn embedding_model(&self) -> Option<&str> {
        self.embedding_model.as_deref()
    }

    /// Returns the configured embedding dimensions, if any.
    #[must_use]
    pub const fn embedding_dimensions(&self) -> Option<u32> {
        self.embedding_dimensions
    }
}

/// A request to enable vector settings for a store.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VectorEnableRequest {
    embedding_model: String,
    embedding_dimensions: u32,
    auto_embed_on_write: bool,
}

impl VectorEnableRequest {
    /// Creates a validated vector-enablement request.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the model is blank or the dimensions are out
    /// of range.
    pub fn new(
        embedding_model: impl Into<String>,
        embedding_dimensions: u32,
    ) -> Result<Self, CoreError> {
        let embedding_model = embedding_model.into();
        let embedding_model = embedding_model.trim();
        if embedding_model.is_empty() {
            return Err(CoreError::EmptyValue {
                field: "embedding_model",
            });
        }
        if embedding_dimensions == 0 {
            return Err(CoreError::OutOfRange {
                field: "embedding_dimensions",
                min: 1,
                max: u64::from(u32::MAX),
                actual: 0,
            });
        }

        Ok(Self {
            embedding_model: embedding_model.to_owned(),
            embedding_dimensions,
            auto_embed_on_write: false,
        })
    }

    /// Returns the configured embedding model identifier.
    #[must_use]
    pub fn embedding_model(&self) -> &str {
        &self.embedding_model
    }

    /// Returns the requested embedding dimensions.
    #[must_use]
    pub const fn embedding_dimensions(&self) -> u32 {
        self.embedding_dimensions
    }

    /// Returns whether auto-embed-on-write should be enabled.
    #[must_use]
    pub const fn auto_embed_on_write(&self) -> bool {
        self.auto_embed_on_write
    }

    /// Enables or disables auto-embed-on-write.
    #[must_use]
    pub fn with_auto_embed_on_write(mut self, auto_embed_on_write: bool) -> Self {
        self.auto_embed_on_write = auto_embed_on_write;
        self
    }
}

/// A request to plan or apply an embedding backfill.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmbeddingBackfillRequest {
    apply: bool,
}

impl EmbeddingBackfillRequest {
    /// Creates a dry-run backfill request.
    #[must_use]
    pub const fn plan() -> Self {
        Self { apply: false }
    }

    /// Creates an apply-mode backfill request.
    #[must_use]
    pub const fn apply() -> Self {
        Self { apply: true }
    }

    /// Returns whether the request should apply writes.
    #[must_use]
    pub const fn apply_writes(&self) -> bool {
        self.apply
    }
}

/// A result describing a backfill plan or execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EmbeddingBackfillResult {
    apply: bool,
    candidate_memories: u64,
    updated_memories: u64,
}

impl EmbeddingBackfillResult {
    /// Creates a backfill result.
    #[must_use]
    pub const fn new(apply: bool, candidate_memories: u64, updated_memories: u64) -> Self {
        Self {
            apply,
            candidate_memories,
            updated_memories,
        }
    }

    /// Returns whether the request ran in apply mode.
    #[must_use]
    pub const fn apply_writes(&self) -> bool {
        self.apply
    }

    /// Returns the number of candidate memories considered for backfill.
    #[must_use]
    pub const fn candidate_memories(&self) -> u64 {
        self.candidate_memories
    }

    /// Returns the number of memories updated by the backfill.
    #[must_use]
    pub const fn updated_memories(&self) -> u64 {
        self.updated_memories
    }
}

/// Runtime options for opening a LanceDB-backed store.
#[derive(Clone, Default)]
pub struct LanceDbOpenOptions {
    embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
}

impl LanceDbOpenOptions {
    /// Starts building backend open options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Attaches a runtime embedding provider to the backend.
    #[must_use]
    pub fn embedding_provider(mut self, provider: Arc<dyn EmbeddingProvider>) -> Self {
        self.embedding_provider = Some(provider);
        self
    }
}

/// A persistent local backend backed by `LanceDB`.
pub struct LanceDbBackend {
    path: PathBuf,
    runtime: Runtime,
    memories: Table,
    memory_embeddings: Option<Table>,
    checkpoints: Table,
    schema_metadata: Table,
    vector_config: Table,
    schema_version: u64,
    vector_settings: VectorSettings,
    embedding_provider: Option<Arc<dyn EmbeddingProvider>>,
}

#[derive(Clone, Debug)]
struct PersistedEmbedding {
    model: String,
    dimensions: u32,
    updated_at: SystemTime,
    values: Vec<f32>,
}

#[derive(Clone, Debug)]
struct StoredMemoryRecord {
    record: MemoryRecord,
    embedding: Option<PersistedEmbedding>,
}

#[derive(Clone, Debug)]
struct SemanticCandidate {
    record: MemoryRecord,
    score: f32,
}

#[derive(Clone, Debug)]
struct RankedMemoryRecord {
    record: MemoryRecord,
    lexical_rank: Option<usize>,
    semantic_rank: Option<usize>,
    semantic_score: Option<f32>,
}

#[derive(Clone, Copy, Debug, Default)]
struct MatchSignals {
    lexical: bool,
    semantic: bool,
}

/// A search result with lightweight retrieval provenance for operator-facing
/// surfaces such as the TUI.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchMatch {
    record: MemoryRecord,
    lexical_match: bool,
    semantic_match: bool,
    semantic_score: Option<f32>,
}

impl SearchMatch {
    fn from_ranked_candidate(candidate: RankedMemoryRecord, text_present: bool) -> Self {
        // Query-free search paths can still produce ranked candidates, but we only surface
        // lexical or semantic provenance when actual query text was present.
        Self {
            lexical_match: candidate.lexical_rank.is_some() && text_present,
            semantic_match: candidate.semantic_rank.is_some() && text_present,
            semantic_score: candidate.semantic_score,
            record: candidate.record,
        }
    }

    /// Returns the surfaced memory record.
    #[must_use]
    pub const fn record(&self) -> &MemoryRecord {
        &self.record
    }

    /// Returns the surfaced memory record by value.
    #[must_use]
    pub fn into_record(self) -> MemoryRecord {
        self.record
    }

    /// Returns `true` when lexical ranking contributed to this result.
    #[must_use]
    pub const fn lexical_match(&self) -> bool {
        self.lexical_match
    }

    /// Returns `true` when semantic ranking contributed to this result.
    #[must_use]
    pub const fn semantic_match(&self) -> bool {
        self.semantic_match
    }

    /// Returns the semantic score when one was available.
    #[must_use]
    pub const fn semantic_score(&self) -> Option<f32> {
        self.semantic_score
    }
}

impl std::fmt::Debug for LanceDbBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanceDbBackend")
            .field("path", &self.path)
            .field("schema_version", &self.schema_version)
            .field("vector_settings", &self.vector_settings)
            .field(
                "has_memory_embeddings_table",
                &self.memory_embeddings.is_some(),
            )
            .field(
                "has_embedding_provider",
                &self.embedding_provider.as_ref().is_some_and(|_| true),
            )
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
        Self::init_with_options(path, LanceDbOpenOptions::default())
    }

    /// Initializes a local store using explicit runtime options.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store or its required tables cannot be
    /// created or the runtime options are incompatible with the store
    /// configuration.
    pub fn init_with_options(
        path: impl AsRef<Path>,
        options: LanceDbOpenOptions,
    ) -> Result<Self, LanceDbError> {
        Self::open_internal(path.as_ref(), true, options)
    }

    /// Opens an existing local store.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store cannot be opened or required tables are missing.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LanceDbError> {
        Self::open_with_options(path, LanceDbOpenOptions::default())
    }

    /// Opens an existing local store using explicit runtime options.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store cannot be opened, required
    /// tables are missing, or the runtime options are incompatible with the
    /// store configuration.
    pub fn open_with_options(
        path: impl AsRef<Path>,
        options: LanceDbOpenOptions,
    ) -> Result<Self, LanceDbError> {
        Self::open_internal(path.as_ref(), false, options)
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
        Self::connect_or_init_with_options(path, LanceDbOpenOptions::default())
    }

    /// Opens or initializes a local store using explicit runtime options.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the store cannot be opened, initialized,
    /// or paired with the supplied runtime options.
    pub fn connect_or_init_with_options(
        path: impl AsRef<Path>,
        options: LanceDbOpenOptions,
    ) -> Result<Self, LanceDbError> {
        Self::open_internal(path.as_ref(), true, options)
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
        Ok(self.schema_version)
    }

    /// Returns the effective vector settings for the store.
    #[must_use]
    pub const fn vector_settings(&self) -> &VectorSettings {
        &self.vector_settings
    }

    /// Returns a typed operational vector snapshot for the store.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when persisted coverage cannot be counted.
    pub fn vector_status(&self) -> Result<VectorStatus, LanceDbError> {
        let total_memories = self.count_memories(None)? as u64;
        let embedded_memories = self.embedded_memory_count()? as u64;

        Ok(VectorStatus {
            settings: self.vector_settings.clone(),
            has_embedding_provider: self.has_embedding_provider(),
            can_embed_on_write: self.can_embed_on_write(),
            semantic_retrieval_available: self.supports_semantic_retrieval(),
            persisted_embedding_storage: self.supports_embedding_storage(),
            indexable_embedding_storage: self.supports_indexable_embedding_storage(),
            total_memories,
            embedded_memories,
            vector_index: self.vector_index_status(),
        })
    }

    /// Returns search results with lightweight retrieval provenance.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the search cannot be executed for the
    /// selected retrieval mode.
    pub fn search_matches(&self, query: &SearchQuery) -> Result<Vec<SearchMatch>, LanceDbError> {
        let text_present = !query.text().is_empty();
        let ranked = self.ranked_search_candidates(query)?;

        Ok(ranked
            .into_iter()
            .map(|candidate| SearchMatch::from_ranked_candidate(candidate, text_present))
            .collect())
    }

    /// Returns `true` when a runtime embedding provider is attached.
    #[must_use]
    pub fn has_embedding_provider(&self) -> bool {
        self.embedding_provider.is_some()
    }

    /// Returns `true` when the store can currently embed new writes.
    #[must_use]
    pub fn can_embed_on_write(&self) -> bool {
        self.supports_embedding_storage()
            && self.vector_settings.vectors_enabled
            && self.vector_settings.auto_embed_on_write
            && self.embedding_provider.is_some()
    }

    fn supports_semantic_retrieval(&self) -> bool {
        self.supports_embedding_storage()
            && self.vector_settings.vectors_enabled
            && self.embedding_provider.is_some()
    }

    /// Enables vector settings for the current store.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when the current store cannot support vectors
    /// yet or the requested settings are invalid.
    pub fn enable_vectors(
        &mut self,
        request: &VectorEnableRequest,
    ) -> Result<VectorSettings, LanceDbError> {
        if !self.supports_embedding_storage() {
            return Err(LanceDbError::InvalidVectorSettings {
                details: format!(
                    "vector enablement requires schema version {PERSISTED_EMBEDDING_SCHEMA_VERSION} stores"
                ),
            });
        }

        let settings = VectorSettings {
            vectors_enabled: true,
            auto_embed_on_write: request.auto_embed_on_write(),
            embedding_model: Some(request.embedding_model().to_owned()),
            embedding_dimensions: Some(request.embedding_dimensions()),
        };
        self.validate_vector_settings(&settings)?;
        if self.schema_version >= INDEXABLE_EMBEDDING_SCHEMA_VERSION {
            self.ensure_memory_embeddings_table(request.embedding_dimensions())?;
        }
        self.write_vector_settings(&settings)?;
        self.vector_settings = settings.clone();
        Ok(settings)
    }

    /// Plans or applies an embedding backfill.
    ///
    /// # Errors
    ///
    /// Returns [`LanceDbError`] when vector settings are disabled, runtime
    /// requirements are missing, or apply mode is requested before write-path
    /// embedding storage is implemented.
    pub fn backfill_embeddings(
        &mut self,
        request: &EmbeddingBackfillRequest,
    ) -> Result<EmbeddingBackfillResult, LanceDbError> {
        if !self.supports_embedding_storage() {
            return Err(LanceDbError::InvalidVectorSettings {
                details: format!(
                    "embedding backfill requires schema version {PERSISTED_EMBEDDING_SCHEMA_VERSION} stores"
                ),
            });
        }
        if !self.vector_settings.vectors_enabled {
            return Err(LanceDbError::InvalidVectorSettings {
                details: "vectors must be enabled before planning or applying backfill".to_owned(),
            });
        }

        let candidates = self.backfill_candidate_records()?;
        let candidate_memories = candidates.len() as u64;
        if !request.apply_writes() {
            return Ok(EmbeddingBackfillResult::new(false, candidate_memories, 0));
        }

        if self.embedding_provider.is_none() {
            return Err(LanceDbError::Core(CoreError::CapabilityUnavailable {
                capability: "embedding_provider",
            }));
        }

        let mut updated_memories = 0_u64;
        for record in &candidates {
            if let Some(embedding) = self.embedding_for_record(record)? {
                self.update_memory_embedding(record.id(), &embedding)?;
                self.upsert_fixed_size_memory_embedding(record.id(), &embedding)?;
                updated_memories += 1;
            }
        }

        Ok(EmbeddingBackfillResult::new(
            true,
            candidate_memories,
            updated_memories,
        ))
    }

    fn backfill_candidate_records(&self) -> Result<Vec<MemoryRecord>, LanceDbError> {
        if self.supports_indexable_embedding_storage() {
            let fixed_embedding_ids = self
                .fixed_size_embedding_ids()?
                .into_iter()
                .collect::<HashSet<_>>();
            let records =
                decode_memory_records(self.query_payloads(&self.memories, None, None, None)?)?;
            return Ok(records
                .into_iter()
                .filter(|record| !fixed_embedding_ids.contains(record.id().as_str()))
                .collect());
        }

        decode_memory_records(self.query_payloads(
            &self.memories,
            Some("embedding_model IS NULL".to_owned()),
            None,
            None,
        )?)
    }

    fn read_schema_version(&self) -> Result<u64, LanceDbError> {
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

    fn read_vector_settings(&self) -> Result<VectorSettings, LanceDbError> {
        let batches: Vec<RecordBatch> = self.block_on(async {
            let stream: SendableRecordBatchStream = self
                .vector_config
                .query()
                .select(Select::Columns(vec![
                    "vectors_enabled".to_owned(),
                    "auto_embed_on_write".to_owned(),
                    "embedding_model".to_owned(),
                    "embedding_dimensions".to_owned(),
                ]))
                .limit(1)
                .execute()
                .await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let Some(batch) = batches.first() else {
            return Err(LanceDbError::InvalidData {
                field: "vector_config",
                details: "missing vector config row".to_owned(),
            });
        };
        let Some(vectors_enabled) = batch
            .column_by_name("vectors_enabled")
            .and_then(|column| column.as_any().downcast_ref::<BooleanArray>())
        else {
            return Err(LanceDbError::InvalidData {
                field: "vectors_enabled",
                details: "expected Boolean column".to_owned(),
            });
        };
        let Some(auto_embed_on_write) = batch
            .column_by_name("auto_embed_on_write")
            .and_then(|column| column.as_any().downcast_ref::<BooleanArray>())
        else {
            return Err(LanceDbError::InvalidData {
                field: "auto_embed_on_write",
                details: "expected Boolean column".to_owned(),
            });
        };
        let Some(embedding_model) = batch
            .column_by_name("embedding_model")
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        else {
            return Err(LanceDbError::InvalidData {
                field: "embedding_model",
                details: "expected Utf8 column".to_owned(),
            });
        };
        let Some(embedding_dimensions) = batch
            .column_by_name("embedding_dimensions")
            .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
        else {
            return Err(LanceDbError::InvalidData {
                field: "embedding_dimensions",
                details: "expected UInt32 column".to_owned(),
            });
        };

        if vectors_enabled.is_empty() || auto_embed_on_write.is_empty() {
            return Err(LanceDbError::InvalidData {
                field: "vector_config",
                details: "missing vector config value".to_owned(),
            });
        }
        if vectors_enabled.is_null(0) || auto_embed_on_write.is_null(0) {
            return Err(LanceDbError::InvalidData {
                field: "vector_config",
                details: "null vector config boolean".to_owned(),
            });
        }

        Ok(VectorSettings {
            vectors_enabled: vectors_enabled.value(0),
            auto_embed_on_write: auto_embed_on_write.value(0),
            embedding_model: (!embedding_model.is_null(0))
                .then(|| embedding_model.value(0).to_owned()),
            embedding_dimensions: (!embedding_dimensions.is_null(0))
                .then(|| embedding_dimensions.value(0)),
        })
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
        if let Some(table) = self.memory_embeddings.as_ref() {
            self.block_on(table.delete(&string_filter("memory_id", id.as_str())))?;
        }
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

    fn open_internal(
        path: &Path,
        create_missing: bool,
        options: LanceDbOpenOptions,
    ) -> Result<Self, LanceDbError> {
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
        let vector_config =
            Self::open_or_create_vector_config_table(&runtime, &connection, &existing_tables)?;
        let memory_embeddings =
            Self::open_memory_embeddings_table(&runtime, &connection, &existing_tables)?;

        let mut backend = Self {
            path: path.to_path_buf(),
            runtime,
            memories,
            memory_embeddings,
            checkpoints,
            schema_metadata,
            vector_config,
            schema_version: 0,
            vector_settings: VectorSettings::default(),
            embedding_provider: options.embedding_provider,
        };

        backend.ensure_fts_index()?;
        backend.ensure_schema_version_row()?;
        backend.ensure_vector_config_row()?;
        backend.schema_version = backend.read_schema_version()?;
        backend.vector_settings = backend.read_vector_settings()?;
        backend.validate_vector_runtime()?;
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
                .create_empty_table(MEMORIES_TABLE, memories_schema(SCHEMA_VERSION))
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

    fn open_or_create_vector_config_table(
        runtime: &Runtime,
        connection: &Connection,
        existing_tables: &[String],
    ) -> Result<Table, LanceDbError> {
        if existing_tables
            .iter()
            .any(|name| name == VECTOR_CONFIG_TABLE)
        {
            return Ok(runtime.block_on(connection.open_table(VECTOR_CONFIG_TABLE).execute())?);
        }

        Ok(runtime.block_on(
            connection
                .create_empty_table(VECTOR_CONFIG_TABLE, vector_config_schema())
                .execute(),
        )?)
    }

    fn open_memory_embeddings_table(
        runtime: &Runtime,
        connection: &Connection,
        existing_tables: &[String],
    ) -> Result<Option<Table>, LanceDbError> {
        if existing_tables
            .iter()
            .any(|name| name == MEMORY_EMBEDDINGS_TABLE)
        {
            return Ok(Some(runtime.block_on(
                connection.open_table(MEMORY_EMBEDDINGS_TABLE).execute(),
            )?));
        }

        Ok(None)
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

    fn ensure_vector_config_row(&self) -> Result<(), LanceDbError> {
        if self.block_on(self.vector_config.count_rows(None))? > 0 {
            return Ok(());
        }

        let config = VectorSettings::default();
        let schema = vector_config_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(BooleanArray::from(vec![config.vectors_enabled])),
                Arc::new(BooleanArray::from(vec![config.auto_embed_on_write])),
                Arc::new(StringArray::from(vec![config.embedding_model.as_deref()])),
                Arc::new(UInt32Array::from(vec![config.embedding_dimensions])),
            ],
        )?;
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        self.block_on(self.vector_config.add(reader).execute())?;
        Ok(())
    }

    fn validate_vector_runtime(&self) -> Result<(), LanceDbError> {
        self.validate_vector_settings(&self.vector_settings)
    }

    fn validate_vector_settings(&self, settings: &VectorSettings) -> Result<(), LanceDbError> {
        if settings.auto_embed_on_write && !settings.vectors_enabled {
            return Err(LanceDbError::InvalidVectorSettings {
                details: "auto-embed-on-write requires vectors_enabled=true".to_owned(),
            });
        }

        let Some(provider) = self.embedding_provider.as_ref() else {
            return Ok(());
        };

        if let Some(model) = settings.embedding_model()
            && model != provider.model_id()
        {
            return Err(LanceDbError::InvalidVectorSettings {
                details: format!(
                    "configured embedding model `{model}` does not match provider `{}`",
                    provider.model_id()
                ),
            });
        }

        if let Some(dimensions) = settings.embedding_dimensions()
            && dimensions != provider.dimensions()
        {
            return Err(LanceDbError::InvalidVectorSettings {
                details: format!(
                    "configured embedding dimensions `{dimensions}` do not match provider `{}`",
                    provider.dimensions()
                ),
            });
        }

        Ok(())
    }

    fn write_vector_settings(&self, settings: &VectorSettings) -> Result<(), LanceDbError> {
        self.block_on(self.vector_config.delete("true"))?;

        let schema = vector_config_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(BooleanArray::from(vec![settings.vectors_enabled()])),
                Arc::new(BooleanArray::from(vec![settings.auto_embed_on_write()])),
                Arc::new(StringArray::from(vec![settings.embedding_model()])),
                Arc::new(UInt32Array::from(vec![settings.embedding_dimensions()])),
            ],
        )?;
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        self.block_on(self.vector_config.add(reader).execute())?;
        Ok(())
    }

    fn supports_embedding_storage(&self) -> bool {
        self.schema_version >= PERSISTED_EMBEDDING_SCHEMA_VERSION
    }

    fn supports_indexable_embedding_storage(&self) -> bool {
        self.schema_version >= INDEXABLE_EMBEDDING_SCHEMA_VERSION
            && self.memory_embeddings.is_some()
    }

    fn ensure_memory_embeddings_table(&mut self, dimensions: u32) -> Result<(), LanceDbError> {
        if self.memory_embeddings.is_some() {
            return Ok(());
        }

        let uri = self.path.to_string_lossy().to_string();
        let connection = self.block_on(connect(&uri).execute())?;
        let existing_tables = self.block_on(connection.table_names().execute())?;
        if existing_tables
            .iter()
            .any(|name| name == MEMORY_EMBEDDINGS_TABLE)
        {
            self.memory_embeddings =
                Some(self.block_on(connection.open_table(MEMORY_EMBEDDINGS_TABLE).execute())?);
            return Ok(());
        }

        let table = self.block_on(
            connection
                .create_empty_table(
                    MEMORY_EMBEDDINGS_TABLE,
                    memory_embeddings_schema(dimensions)?,
                )
                .execute(),
        )?;
        self.memory_embeddings = Some(table);
        Ok(())
    }

    fn vector_index_status(&self) -> VectorIndexStatus {
        if !self.supports_embedding_storage() {
            return VectorIndexStatus {
                available: false,
                reason: Some(format!(
                    "schema version {PERSISTED_EMBEDDING_SCHEMA_VERSION} embedding storage is required first"
                )),
            };
        }

        if self.schema_version < INDEXABLE_EMBEDDING_SCHEMA_VERSION {
            return VectorIndexStatus {
                available: false,
                reason: Some(
                    "store uses schema-v3 List<Float32> embeddings; schema-v4 fixed-size embedding storage is required for LanceDB-native vector indexing"
                        .to_owned(),
                ),
            };
        }

        if self.memory_embeddings.is_none() {
            return VectorIndexStatus {
                available: false,
                reason: Some(
                    "enable vectors on this schema-v4 store to create the fixed-size embedding table"
                        .to_owned(),
                ),
            };
        }

        if self.embedded_memory_count().ok() == Some(0) {
            return VectorIndexStatus {
                available: false,
                reason: Some(
                    "fixed-size embedding storage exists but contains no persisted embeddings yet"
                        .to_owned(),
                ),
            };
        }

        VectorIndexStatus {
            available: false,
            reason: Some(
                "fixed-size embedding storage is populated; LanceDB-native vector index creation is not implemented yet"
                    .to_owned(),
            ),
        }
    }

    fn embedded_memory_count(&self) -> Result<usize, LanceDbError> {
        if self.schema_version <= LEGACY_SCHEMA_VERSION {
            return Ok(0);
        }

        if self.supports_indexable_embedding_storage() {
            let ids = self.fixed_size_embedding_ids()?;
            if ids.is_empty() {
                return Ok(0);
            }

            return Ok(self.query_memory_records_by_ids(&ids, None)?.len());
        }

        self.count_memories(Some("embedding_model IS NOT NULL".to_owned()))
    }

    fn embedding_for_record(
        &self,
        record: &MemoryRecord,
    ) -> Result<Option<PersistedEmbedding>, LanceDbError> {
        if !self.vector_settings.vectors_enabled {
            return Ok(None);
        }

        let Some(provider) = self.embedding_provider.as_ref() else {
            return Ok(None);
        };

        let values = provider.embed(record.fts_text()).map_err(|error| {
            LanceDbError::InvalidVectorSettings {
                details: error.to_string(),
            }
        })?;
        let model =
            self.vector_settings
                .embedding_model()
                .ok_or(LanceDbError::InvalidVectorSettings {
                    details: "vectors_enabled stores must define embedding_model".to_owned(),
                })?;
        let expected_dimensions = self.vector_settings.embedding_dimensions().ok_or(
            LanceDbError::InvalidVectorSettings {
                details: "vectors_enabled stores must define embedding_dimensions".to_owned(),
            },
        )?;
        if values.len() != expected_dimensions as usize {
            return Err(LanceDbError::InvalidVectorSettings {
                details: format!(
                    "provider returned {} dimensions but store expects {expected_dimensions}",
                    values.len()
                ),
            });
        }

        Ok(Some(PersistedEmbedding {
            model: model.to_owned(),
            dimensions: expected_dimensions,
            updated_at: SystemTime::now(),
            values,
        }))
    }

    fn update_memory_embedding(
        &self,
        id: &MemoryId,
        embedding: &PersistedEmbedding,
    ) -> Result<(), LanceDbError> {
        let (updated_secs, updated_nanos) =
            system_time_to_parts(embedding.updated_at, "embedding_updated_at")?;

        self.block_on(
            self.memories
                .update()
                .only_if(string_filter("id", id.as_str()))
                .column("embedding_model", sql_string_literal(&embedding.model))
                .column("embedding_dimensions", embedding.dimensions.to_string())
                .column("embedding_updated_at_secs", updated_secs.to_string())
                .column("embedding_updated_at_nanos", updated_nanos.to_string())
                .column("embedding", sql_float32_list_literal(&embedding.values))
                .execute(),
        )?;
        Ok(())
    }

    fn upsert_fixed_size_memory_embedding(
        &self,
        id: &MemoryId,
        embedding: &PersistedEmbedding,
    ) -> Result<(), LanceDbError> {
        let Some(table) = self.memory_embeddings.as_ref() else {
            return Ok(());
        };

        self.block_on(table.delete(&string_filter("memory_id", id.as_str())))?;
        let (schema, batch) = memory_embedding_batch(id, embedding)?;
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        self.block_on(table.add(reader).execute())?;
        Ok(())
    }

    fn semantic_query_embedding(
        &self,
        text: Option<&str>,
    ) -> Result<Option<Vec<f32>>, LanceDbError> {
        let Some(text) = text else {
            return Ok(None);
        };
        let Some(provider) = self.embedding_provider.as_ref() else {
            return Ok(None);
        };

        let values = provider
            .embed(text)
            .map_err(|error| LanceDbError::InvalidVectorSettings {
                details: error.to_string(),
            })?;
        let expected_dimensions = self.vector_settings.embedding_dimensions().ok_or(
            LanceDbError::InvalidVectorSettings {
                details: "vectors_enabled stores must define embedding_dimensions".to_owned(),
            },
        )?;
        if values.len() != expected_dimensions as usize {
            return Err(LanceDbError::InvalidVectorSettings {
                details: format!(
                    "provider returned {} dimensions but store expects {expected_dimensions}",
                    values.len()
                ),
            });
        }

        Ok(Some(values))
    }

    fn query_memory_records_by_ids(
        &self,
        ids: &[String],
        filter: Option<String>,
    ) -> Result<HashMap<String, MemoryRecord>, LanceDbError> {
        if ids.is_empty() {
            return Ok(HashMap::new());
        }

        let batches = self.block_on(async {
            let mut query = self.memories.query().select(Select::Columns(vec![
                "id".to_owned(),
                PAYLOAD_COLUMN.to_owned(),
            ]));

            let id_filter = string_in_filter("id", ids);
            let combined_filter = combine_filters([Some(id_filter), filter]);
            if let Some(filter) = combined_filter {
                query = query.only_if(filter);
            }

            let stream: SendableRecordBatchStream = query.execute().await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut records = HashMap::new();
        for batch in batches {
            let Some(ids) = batch
                .column_by_name("id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: "id",
                    details: "expected Utf8 id column".to_owned(),
                });
            };
            let Some(payloads) = batch
                .column_by_name(PAYLOAD_COLUMN)
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: PAYLOAD_COLUMN,
                    details: "expected Utf8 payload column".to_owned(),
                });
            };

            for index in 0..payloads.len() {
                if ids.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: "id",
                        details: "id column contained null".to_owned(),
                    });
                }
                if payloads.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: PAYLOAD_COLUMN,
                        details: "payload column contained null".to_owned(),
                    });
                }

                let id = ids.value(index).to_owned();
                let record = serde_json::from_str::<MemoryRecord>(payloads.value(index))?;
                records.insert(id, record);
            }
        }

        Ok(records)
    }

    fn read_stored_memory_records(
        &self,
        table: &Table,
        schema_version: u64,
        filter: Option<String>,
    ) -> Result<Vec<StoredMemoryRecord>, LanceDbError> {
        let mut columns = vec![PAYLOAD_COLUMN.to_owned()];
        if schema_version > LEGACY_SCHEMA_VERSION {
            columns.extend([
                "embedding_model".to_owned(),
                "embedding_dimensions".to_owned(),
                "embedding_updated_at_secs".to_owned(),
                "embedding_updated_at_nanos".to_owned(),
                "embedding".to_owned(),
            ]);
        }

        let batches = self.block_on(async {
            let mut query = table.query().select(Select::Columns(columns));
            if let Some(filter) = filter {
                query = query.only_if(filter);
            }

            let stream: SendableRecordBatchStream = query.execute().await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut records = Vec::new();
        for batch in batches {
            let Some(payloads) = batch
                .column_by_name(PAYLOAD_COLUMN)
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: PAYLOAD_COLUMN,
                    details: "expected Utf8 payload column".to_owned(),
                });
            };

            for index in 0..payloads.len() {
                if payloads.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: PAYLOAD_COLUMN,
                        details: "payload column contained null".to_owned(),
                    });
                }

                records.push(StoredMemoryRecord {
                    record: serde_json::from_str::<MemoryRecord>(payloads.value(index))?,
                    embedding: persisted_embedding_from_memory_batch(
                        &batch,
                        index,
                        schema_version,
                    )?,
                });
            }
        }

        Ok(records)
    }

    fn fixed_size_embedding_ids(&self) -> Result<Vec<String>, LanceDbError> {
        let Some(table) = self.memory_embeddings.as_ref() else {
            return Ok(Vec::new());
        };

        let batches = self.block_on(async {
            let stream: SendableRecordBatchStream = table
                .query()
                .select(Select::Columns(vec!["memory_id".to_owned()]))
                .execute()
                .await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut ids = Vec::new();
        for batch in batches {
            let Some(array) = batch
                .column_by_name("memory_id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: "memory_id",
                    details: "expected Utf8 memory_id column".to_owned(),
                });
            };

            for index in 0..array.len() {
                if array.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: "memory_id",
                        details: "memory_id column contained null".to_owned(),
                    });
                }
                ids.push(array.value(index).to_owned());
            }
        }

        Ok(ids)
    }

    fn fixed_size_embeddings_by_id(
        &self,
    ) -> Result<HashMap<String, PersistedEmbedding>, LanceDbError> {
        let Some(table) = self.memory_embeddings.as_ref() else {
            return Ok(HashMap::new());
        };

        let batches = self.block_on(async {
            let stream: SendableRecordBatchStream = table
                .query()
                .select(Select::Columns(vec![
                    "memory_id".to_owned(),
                    "embedding_model".to_owned(),
                    "embedding_updated_at_secs".to_owned(),
                    "embedding_updated_at_nanos".to_owned(),
                    "embedding".to_owned(),
                ]))
                .execute()
                .await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut embeddings = HashMap::new();
        for batch in batches {
            let Some(ids) = batch
                .column_by_name("memory_id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: "memory_id",
                    details: "expected Utf8 memory_id column".to_owned(),
                });
            };

            for index in 0..ids.len() {
                if ids.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: "memory_id",
                        details: "memory_id column contained null".to_owned(),
                    });
                }

                embeddings.insert(
                    ids.value(index).to_owned(),
                    persisted_embedding_from_fixed_size_batch(&batch, index)?,
                );
            }
        }

        Ok(embeddings)
    }

    fn query_embedded_memories_from_memories(
        &self,
        table: &Table,
        filter: Option<String>,
    ) -> Result<Vec<(MemoryRecord, Vec<f32>)>, LanceDbError> {
        let batches = self.block_on(async {
            let mut query = table.query().select(Select::Columns(vec![
                PAYLOAD_COLUMN.to_owned(),
                "embedding".to_owned(),
            ]));

            if let Some(filter) = filter {
                query = query.only_if(filter);
            }

            let stream: SendableRecordBatchStream = query.execute().await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut records = Vec::new();
        for batch in batches {
            let Some(payloads) = batch
                .column_by_name(PAYLOAD_COLUMN)
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: PAYLOAD_COLUMN,
                    details: "expected Utf8 payload column".to_owned(),
                });
            };
            let Some(embeddings) = batch
                .column_by_name("embedding")
                .and_then(|column| column.as_any().downcast_ref::<ListArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: "embedding",
                    details: "expected List<Float32> embedding column".to_owned(),
                });
            };

            for index in 0..payloads.len() {
                if payloads.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: PAYLOAD_COLUMN,
                        details: "payload column contained null".to_owned(),
                    });
                }
                if embeddings.is_null(index) {
                    continue;
                }

                let record = serde_json::from_str::<MemoryRecord>(payloads.value(index))?;
                let values = embeddings
                    .value(index)
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .ok_or(LanceDbError::InvalidData {
                        field: "embedding",
                        details: "embedding values must be Float32".to_owned(),
                    })?
                    .values()
                    .to_vec();
                records.push((record, values));
            }
        }

        Ok(records)
    }

    fn query_embedded_memories_from_fixed_size_table(
        &self,
        filter: Option<String>,
    ) -> Result<Vec<(MemoryRecord, Vec<f32>)>, LanceDbError> {
        let Some(table) = self.memory_embeddings.as_ref() else {
            return Ok(Vec::new());
        };

        let batches = self.block_on(async {
            let stream: SendableRecordBatchStream = table
                .query()
                .select(Select::Columns(vec![
                    "memory_id".to_owned(),
                    "embedding".to_owned(),
                ]))
                .execute()
                .await?;
            let batches: Vec<RecordBatch> = stream.try_collect().await?;
            Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
        })?;

        let mut embeddings = Vec::new();
        let mut memory_ids = Vec::new();
        for batch in batches {
            let Some(ids) = batch
                .column_by_name("memory_id")
                .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: "memory_id",
                    details: "expected Utf8 memory_id column".to_owned(),
                });
            };
            let Some(embeddings_array) = batch
                .column_by_name("embedding")
                .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
            else {
                return Err(LanceDbError::InvalidData {
                    field: "embedding",
                    details: "expected FixedSizeList<Float32> embedding column".to_owned(),
                });
            };

            for index in 0..ids.len() {
                if ids.is_null(index) {
                    return Err(LanceDbError::InvalidData {
                        field: "memory_id",
                        details: "memory_id column contained null".to_owned(),
                    });
                }
                if embeddings_array.is_null(index) {
                    continue;
                }

                let id = ids.value(index).to_owned();
                let values = embeddings_array
                    .value(index)
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .ok_or(LanceDbError::InvalidData {
                        field: "embedding",
                        details: "embedding values must be Float32".to_owned(),
                    })?
                    .values()
                    .to_vec();
                memory_ids.push(id.clone());
                embeddings.push((id, values));
            }
        }

        let records_by_id = self.query_memory_records_by_ids(&memory_ids, filter)?;
        Ok(embeddings
            .into_iter()
            .filter_map(|(id, embedding)| {
                records_by_id
                    .get(&id)
                    .cloned()
                    .map(|record| (record, embedding))
            })
            .collect())
    }

    fn query_embedded_memories(
        &self,
        filter: Option<String>,
    ) -> Result<Vec<(MemoryRecord, Vec<f32>)>, LanceDbError> {
        if self.supports_indexable_embedding_storage() {
            return self.query_embedded_memories_from_fixed_size_table(filter);
        }

        self.query_embedded_memories_from_memories(&self.memories, filter)
    }

    fn lexical_candidates(
        &self,
        filter: Option<String>,
        limit: usize,
        text: Option<&str>,
    ) -> Result<Vec<MemoryRecord>, LanceDbError> {
        let payloads = self.query_payloads(
            &self.memories,
            filter,
            Some(limit),
            text.map(ToOwned::to_owned),
        )?;
        decode_memory_records(payloads)
    }

    fn semantic_candidates(
        &self,
        filter: Option<String>,
        text: Option<&str>,
    ) -> Result<Vec<SemanticCandidate>, LanceDbError> {
        let Some(query_embedding) = self.semantic_query_embedding(text)? else {
            return Ok(Vec::new());
        };

        let mut candidates = self
            .query_embedded_memories(filter)?
            .into_iter()
            .filter_map(|(record, embedding)| {
                cosine_similarity(&query_embedding, &embedding)
                    .map(|score| SemanticCandidate { record, score })
            })
            .collect::<Vec<_>>();
        candidates.sort_by(semantic_candidate_cmp);
        Ok(candidates)
    }

    fn require_semantic_capability(&self) -> Result<(), LanceDbError> {
        if self.supports_semantic_retrieval() {
            Ok(())
        } else {
            Err(LanceDbError::Core(CoreError::CapabilityUnavailable {
                capability: "semantic_search",
            }))
        }
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
        let embedding = if self.can_embed_on_write() {
            self.embedding_for_record(record)?
        } else {
            None
        };
        self.block_on_backend(insert_memory_record_async(
            table,
            record,
            self.schema_version,
            embedding.as_ref(),
        ))?;

        if let Some(embedding) = embedding.as_ref() {
            let is_main_memories_table =
                self.table_uri(table)? == self.table_uri(&self.memories)?;
            if is_main_memories_table {
                self.upsert_fixed_size_memory_embedding(record.id(), embedding)?;
            }
        }

        Ok(())
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
        let vector_config_uri = self.table_uri(&self.vector_config)?;
        let memory_embeddings_uri = self
            .memory_embeddings
            .as_ref()
            .map(|table| self.table_uri(table))
            .transpose()?;

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
        self.clone_table_dataset(
            &vector_config_uri,
            &Self::clone_destination_uri(destination, VECTOR_CONFIG_TABLE),
            kind,
        )?;
        if let Some(memory_embeddings_uri) = memory_embeddings_uri.as_deref() {
            self.clone_table_dataset(
                memory_embeddings_uri,
                &Self::clone_destination_uri(destination, MEMORY_EMBEDDINGS_TABLE),
                kind,
            )?;
        }

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
        &mut self,
        source_backend: &LanceDbBackend,
        source_table: &Table,
        branch_table: &Table,
    ) -> Result<u64, LanceDbError> {
        let source_records = source_backend.read_stored_memory_records(
            source_table,
            source_backend.schema_version,
            None,
        )?;
        let source_fixed_embeddings = source_backend.fixed_size_embeddings_by_id()?;

        let mut staged_records = 0_u64;
        for stored in source_records {
            if self.block_on_backend(table_contains_memory_id_async(
                branch_table,
                stored.record.id(),
            ))? {
                continue;
            }

            self.block_on_backend(insert_memory_record_async(
                branch_table,
                &stored.record,
                self.schema_version,
                stored.embedding.as_ref(),
            ))?;

            if self.schema_version >= INDEXABLE_EMBEDDING_SCHEMA_VERSION
                && let Some(embedding) = source_fixed_embeddings.get(stored.record.id().as_str())
            {
                self.ensure_memory_embeddings_table(embedding.dimensions)?;
                self.upsert_fixed_size_memory_embedding(stored.record.id(), embedding)?;
            }

            staged_records += 1;
        }

        Ok(staged_records)
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
        let mut capabilities = vec![
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
        ];
        if self.supports_semantic_retrieval() {
            capabilities.push(BackendCapability::SemanticSearch);
            capabilities.push(BackendCapability::HybridSearch);
        }

        BackendCapabilities::new(capabilities)
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

impl BrowseBackend for LanceDbBackend {
    fn list_memories(
        &self,
        scope: Option<&ScopeId>,
        limit: QueryLimit,
    ) -> Result<Vec<MemoryRecord>, Self::Error> {
        LanceDbBackend::list_memories(self, scope, limit)
    }

    fn list_pinned_memories(
        &self,
        scope: Option<&ScopeId>,
        limit: QueryLimit,
    ) -> Result<Vec<MemoryRecord>, Self::Error> {
        LanceDbBackend::list_pinned_memories(self, scope, limit)
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
        if query.retrieval_mode() == RetrievalMode::SemanticOnly && query.text().is_some() {
            self.require_semantic_capability()?;
        }
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
        Ok(self
            .search_matches(query)?
            .into_iter()
            .map(SearchMatch::into_record)
            .collect())
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
            self.stage_import_records(&source_backend, &source_backend.memories, &branch_table)
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

        let ranked = match query.retrieval_mode() {
            RetrievalMode::LexicalOnly => {
                let mut ranked = self
                    .lexical_candidates(filter, recall_fetch_limit(limit), query.text())?
                    .into_iter()
                    .enumerate()
                    .map(|(index, record)| RankedMemoryRecord {
                        record,
                        lexical_rank: query.text().map(|_| index + 1),
                        semantic_rank: None,
                        semantic_score: None,
                    })
                    .collect::<Vec<_>>();
                ranked.sort_by(recall_ranked_candidate_cmp);
                ranked
            }
            RetrievalMode::SemanticOnly => {
                if query.text().is_some() {
                    self.require_semantic_capability()?;
                }
                let mut ranked = self
                    .semantic_candidates(filter, query.text())?
                    .into_iter()
                    .enumerate()
                    .map(|(index, candidate)| RankedMemoryRecord {
                        record: candidate.record,
                        lexical_rank: None,
                        semantic_rank: Some(index + 1),
                        semantic_score: Some(candidate.score),
                    })
                    .collect::<Vec<_>>();
                ranked.sort_by(recall_ranked_candidate_cmp);
                ranked
            }
            RetrievalMode::Hybrid => {
                let mut ranked = if !self.supports_semantic_retrieval() || query.text().is_none() {
                    self.lexical_candidates(filter, recall_fetch_limit(limit), query.text())?
                        .into_iter()
                        .enumerate()
                        .map(|(index, record)| RankedMemoryRecord {
                            record,
                            lexical_rank: query.text().map(|_| index + 1),
                            semantic_rank: None,
                            semantic_score: None,
                        })
                        .collect::<Vec<_>>()
                } else {
                    let lexical = self.lexical_candidates(
                        filter.clone(),
                        recall_fetch_limit(limit),
                        query.text(),
                    )?;
                    let semantic = self.semantic_candidates(filter, query.text())?;
                    merge_ranked_candidates(lexical, semantic)
                };
                ranked.sort_by(recall_ranked_candidate_cmp);
                ranked
            }
        };
        let explain_context = RecallExplainContext::from_records(
            &ranked
                .iter()
                .map(|candidate| candidate.record.clone())
                .collect::<Vec<_>>(),
        );

        Ok(ranked
            .into_iter()
            .take(limit)
            .map(|candidate| {
                let signals = MatchSignals {
                    lexical: candidate.lexical_rank.is_some() && query.text().is_some(),
                    semantic: candidate.semantic_rank.is_some() && query.text().is_some(),
                };
                recall_entry(candidate.record, layer, query, &explain_context, signals)
            })
            .collect())
    }

    fn ranked_search_candidates(
        &self,
        query: &SearchQuery,
    ) -> Result<Vec<RankedMemoryRecord>, LanceDbError> {
        let scope_filter = query
            .scope()
            .map(|scope| string_filter("scope_id", scope.as_str()));
        let limit = usize::from(query.limit().value());

        let mut ranked = match query.retrieval_mode() {
            RetrievalMode::LexicalOnly => self
                .lexical_candidates(scope_filter, limit, Some(query.text()))?
                .into_iter()
                .enumerate()
                .map(|(index, record)| RankedMemoryRecord {
                    record,
                    lexical_rank: Some(index + 1),
                    semantic_rank: None,
                    semantic_score: None,
                })
                .collect::<Vec<_>>(),
            RetrievalMode::SemanticOnly => {
                self.require_semantic_capability()?;
                self.semantic_candidates(scope_filter, Some(query.text()))?
                    .into_iter()
                    .enumerate()
                    .map(|(index, candidate)| RankedMemoryRecord {
                        record: candidate.record,
                        lexical_rank: None,
                        semantic_rank: Some(index + 1),
                        semantic_score: Some(candidate.score),
                    })
                    .collect::<Vec<_>>()
            }
            RetrievalMode::Hybrid => {
                if self.supports_semantic_retrieval() {
                    let lexical = self.lexical_candidates(
                        scope_filter.clone(),
                        recall_fetch_limit(limit),
                        Some(query.text()),
                    )?;
                    let semantic = self.semantic_candidates(scope_filter, Some(query.text()))?;
                    merge_ranked_candidates(lexical, semantic)
                } else {
                    self.lexical_candidates(scope_filter, limit, Some(query.text()))?
                        .into_iter()
                        .enumerate()
                        .map(|(index, record)| RankedMemoryRecord {
                            record,
                            lexical_rank: Some(index + 1),
                            semantic_rank: None,
                            semantic_score: None,
                        })
                        .collect::<Vec<_>>()
                }
            }
        };

        ranked.sort_by(search_ranked_candidate_cmp);
        ranked.truncate(limit);
        Ok(ranked)
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

fn memories_schema(schema_version: u64) -> Arc<Schema> {
    if schema_version <= LEGACY_SCHEMA_VERSION {
        return Arc::new(Schema::new(vec![
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
        ]));
    }

    if schema_version <= VECTOR_CONFIG_SCHEMA_VERSION {
        return Arc::new(Schema::new(vec![
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
            Field::new("embedding_model", DataType::Utf8, true),
            Field::new("embedding_dimensions", DataType::UInt32, true),
            Field::new("embedding_updated_at_secs", DataType::UInt64, true),
            Field::new("embedding_updated_at_nanos", DataType::UInt32, true),
            Field::new("pinned", DataType::Boolean, false),
            Field::new(PAYLOAD_COLUMN, DataType::Utf8, false),
        ]));
    }

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
        Field::new("embedding_model", DataType::Utf8, true),
        Field::new("embedding_dimensions", DataType::UInt32, true),
        Field::new("embedding_updated_at_secs", DataType::UInt64, true),
        Field::new("embedding_updated_at_nanos", DataType::UInt32, true),
        Field::new(
            "embedding",
            DataType::List(Arc::new(Field::new_list_field(DataType::Float32, true))),
            true,
        ),
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

fn vector_config_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("vectors_enabled", DataType::Boolean, false),
        Field::new("auto_embed_on_write", DataType::Boolean, false),
        Field::new("embedding_model", DataType::Utf8, true),
        Field::new("embedding_dimensions", DataType::UInt32, true),
    ]))
}

fn memory_embeddings_schema(dimensions: u32) -> Result<Arc<Schema>, LanceDbError> {
    if dimensions == 0 {
        return Err(LanceDbError::InvalidVectorSettings {
            details: "memory_embeddings schema requires a non-zero embedding dimension".to_owned(),
        });
    }

    let dimension_length =
        i32::try_from(dimensions).map_err(|_| LanceDbError::InvalidVectorSettings {
            details: format!("embedding dimension `{dimensions}` exceeds supported Arrow size"),
        })?;

    Ok(Arc::new(Schema::new(vec![
        Field::new("memory_id", DataType::Utf8, false),
        Field::new("embedding_model", DataType::Utf8, false),
        Field::new("embedding_updated_at_secs", DataType::UInt64, false),
        Field::new("embedding_updated_at_nanos", DataType::UInt32, false),
        Field::new(
            "embedding",
            DataType::FixedSizeList(
                Arc::new(Field::new_list_field(DataType::Float32, true)),
                dimension_length,
            ),
            false,
        ),
    ])))
}

fn memory_embedding_batch(
    id: &MemoryId,
    embedding: &PersistedEmbedding,
) -> Result<(Arc<Schema>, RecordBatch), LanceDbError> {
    let (updated_secs, updated_nanos) =
        system_time_to_parts(embedding.updated_at, "embedding_updated_at")?;
    let schema = memory_embeddings_schema(embedding.dimensions)?;
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(StringArray::from(vec![Some(id.as_str())])) as Arc<dyn Array>,
            Arc::new(StringArray::from(vec![Some(embedding.model.as_str())])) as Arc<dyn Array>,
            Arc::new(UInt64Array::from(vec![updated_secs])) as Arc<dyn Array>,
            Arc::new(UInt32Array::from(vec![updated_nanos])) as Arc<dyn Array>,
            Arc::new(fixed_size_embedding_array(
                embedding.values.as_slice(),
                embedding.dimensions,
            )) as Arc<dyn Array>,
        ],
    )?;
    Ok((schema, batch))
}

fn persisted_embedding_from_memory_batch(
    batch: &RecordBatch,
    index: usize,
    schema_version: u64,
) -> Result<Option<PersistedEmbedding>, LanceDbError> {
    if schema_version <= LEGACY_SCHEMA_VERSION {
        return Ok(None);
    }

    let Some(models) = batch
        .column_by_name("embedding_model")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_model",
            details: "expected Utf8 embedding_model column".to_owned(),
        });
    };
    let Some(dimensions) = batch
        .column_by_name("embedding_dimensions")
        .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_dimensions",
            details: "expected UInt32 embedding_dimensions column".to_owned(),
        });
    };
    let Some(updated_secs) = batch
        .column_by_name("embedding_updated_at_secs")
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_updated_at_secs",
            details: "expected UInt64 embedding_updated_at_secs column".to_owned(),
        });
    };
    let Some(updated_nanos) = batch
        .column_by_name("embedding_updated_at_nanos")
        .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_updated_at_nanos",
            details: "expected UInt32 embedding_updated_at_nanos column".to_owned(),
        });
    };
    let Some(embeddings) = batch
        .column_by_name("embedding")
        .and_then(|column| column.as_any().downcast_ref::<ListArray>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding",
            details: "expected List<Float32> embedding column".to_owned(),
        });
    };

    let has_embedding = !models.is_null(index)
        || !dimensions.is_null(index)
        || !updated_secs.is_null(index)
        || !updated_nanos.is_null(index)
        || !embeddings.is_null(index);
    if !has_embedding {
        return Ok(None);
    }
    if models.is_null(index)
        || dimensions.is_null(index)
        || updated_secs.is_null(index)
        || updated_nanos.is_null(index)
        || embeddings.is_null(index)
    {
        return Err(LanceDbError::InvalidData {
            field: "embedding",
            details: "embedding columns must be populated together".to_owned(),
        });
    }

    Ok(Some(PersistedEmbedding {
        model: models.value(index).to_owned(),
        dimensions: dimensions.value(index),
        updated_at: parts_to_system_time(
            updated_secs.value(index),
            updated_nanos.value(index),
            "embedding_updated_at",
        )?,
        values: embeddings
            .value(index)
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or(LanceDbError::InvalidData {
                field: "embedding",
                details: "embedding values must be Float32".to_owned(),
            })?
            .values()
            .to_vec(),
    }))
}

fn persisted_embedding_from_fixed_size_batch(
    batch: &RecordBatch,
    index: usize,
) -> Result<PersistedEmbedding, LanceDbError> {
    let Some(models) = batch
        .column_by_name("embedding_model")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_model",
            details: "expected Utf8 embedding_model column".to_owned(),
        });
    };
    let Some(updated_secs) = batch
        .column_by_name("embedding_updated_at_secs")
        .and_then(|column| column.as_any().downcast_ref::<UInt64Array>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_updated_at_secs",
            details: "expected UInt64 embedding_updated_at_secs column".to_owned(),
        });
    };
    let Some(updated_nanos) = batch
        .column_by_name("embedding_updated_at_nanos")
        .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding_updated_at_nanos",
            details: "expected UInt32 embedding_updated_at_nanos column".to_owned(),
        });
    };
    let Some(embeddings) = batch
        .column_by_name("embedding")
        .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
    else {
        return Err(LanceDbError::InvalidData {
            field: "embedding",
            details: "expected FixedSizeList<Float32> embedding column".to_owned(),
        });
    };
    if models.is_null(index)
        || updated_secs.is_null(index)
        || updated_nanos.is_null(index)
        || embeddings.is_null(index)
    {
        return Err(LanceDbError::InvalidData {
            field: "embedding",
            details: "fixed-size embedding row contained null values".to_owned(),
        });
    }

    Ok(PersistedEmbedding {
        model: models.value(index).to_owned(),
        dimensions: u32::try_from(embeddings.value_length()).map_err(|_| {
            LanceDbError::InvalidVectorSettings {
                details: "fixed-size embedding dimensions exceeded supported size".to_owned(),
            }
        })?,
        updated_at: parts_to_system_time(
            updated_secs.value(index),
            updated_nanos.value(index),
            "embedding_updated_at",
        )?,
        values: embeddings
            .value(index)
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or(LanceDbError::InvalidData {
                field: "embedding",
                details: "fixed-size embedding values must be Float32".to_owned(),
            })?
            .values()
            .to_vec(),
    })
}

fn string_filter(column: &str, value: &str) -> String {
    format!("{column} = '{}'", sql_escape(value))
}

fn string_in_filter(column: &str, values: &[String]) -> String {
    let values = values
        .iter()
        .map(|value| sql_string_literal(value))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{column} IN ({values})")
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

fn memory_record_batch(
    record: &MemoryRecord,
    schema_version: u64,
    embedding: Option<&PersistedEmbedding>,
) -> Result<(Arc<Schema>, RecordBatch), LanceDbError> {
    let payload_json = serde_json::to_string(record)?;
    let (created_secs, created_nanos) =
        system_time_to_parts(record.created_at().value(), "created_at")?;
    let (updated_secs, updated_nanos) =
        system_time_to_parts(record.updated_at().value(), "updated_at")?;
    let embedding_parts = embedding
        .map(|value| system_time_to_parts(value.updated_at, "embedding_updated_at"))
        .transpose()?;
    let schema = memories_schema(schema_version);
    let mut columns: Vec<Arc<dyn Array>> = vec![
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
    ];
    if schema_version > LEGACY_SCHEMA_VERSION {
        columns.extend([
            Arc::new(StringArray::from(vec![
                embedding.map(|value| value.model.as_str()),
            ])) as Arc<dyn Array>,
            Arc::new(UInt32Array::from(vec![
                embedding.map(|value| value.dimensions),
            ])) as Arc<dyn Array>,
            Arc::new(UInt64Array::from(vec![
                embedding_parts.map(|(secs, _)| secs),
            ])) as Arc<dyn Array>,
            Arc::new(UInt32Array::from(vec![
                embedding_parts.map(|(_, nanos)| nanos),
            ])) as Arc<dyn Array>,
        ]);
    }
    if schema_version > VECTOR_CONFIG_SCHEMA_VERSION {
        columns.push(Arc::new(embedding_list_array(
            embedding.map(|value| value.values.as_slice()),
        )) as Arc<dyn Array>);
    }
    columns.extend([
        Arc::new(BooleanArray::from(vec![record.pin_state().is_pinned()])) as Arc<dyn Array>,
        Arc::new(StringArray::from(vec![Some(payload_json.as_str())])) as Arc<dyn Array>,
    ]);
    let batch = RecordBatch::try_new(schema.clone(), columns)?;

    Ok((schema, batch))
}

async fn insert_memory_record_async(
    table: &Table,
    record: &MemoryRecord,
    schema_version: u64,
    embedding: Option<&PersistedEmbedding>,
) -> Result<(), LanceDbError> {
    let (schema, batch) = memory_record_batch(record, schema_version, embedding)?;
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

fn merge_ranked_candidates(
    lexical: Vec<MemoryRecord>,
    semantic: Vec<SemanticCandidate>,
) -> Vec<RankedMemoryRecord> {
    let mut merged: HashMap<String, RankedMemoryRecord> = HashMap::new();

    for (index, record) in lexical.into_iter().enumerate() {
        merged.insert(
            record.id().as_str().to_owned(),
            RankedMemoryRecord {
                record,
                lexical_rank: Some(index + 1),
                semantic_rank: None,
                semantic_score: None,
            },
        );
    }

    for (index, candidate) in semantic.into_iter().enumerate() {
        let key = candidate.record.id().as_str().to_owned();
        if let Some(existing) = merged.get_mut(&key) {
            existing.semantic_rank = Some(index + 1);
            existing.semantic_score = Some(candidate.score);
        } else {
            merged.insert(
                key,
                RankedMemoryRecord {
                    record: candidate.record,
                    lexical_rank: None,
                    semantic_rank: Some(index + 1),
                    semantic_score: Some(candidate.score),
                },
            );
        }
    }

    let mut ranked = merged.into_values().collect::<Vec<_>>();
    ranked.sort_by(search_ranked_candidate_cmp);
    ranked
}

fn recall_rank_key(record: &MemoryRecord) -> (u8, u8, u8, SystemTime) {
    (
        u8::from(record.pin_state().is_pinned()),
        u8::from(matches!(record.kind(), MemoryKind::Summary)),
        record.importance().value(),
        record.updated_at().value(),
    )
}

fn embedding_list_array(values: Option<&[f32]>) -> ListArray {
    ListArray::from_iter_primitive::<Float32Type, _, _>(vec![values.map(|items| {
        items
            .iter()
            .copied()
            .map(Some)
            .collect::<Vec<Option<f32>>>()
    })])
}

fn fixed_size_embedding_array(values: &[f32], dimensions: u32) -> FixedSizeListArray {
    FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
        vec![Some(
            values
                .iter()
                .copied()
                .map(Some)
                .collect::<Vec<Option<f32>>>(),
        )],
        i32::try_from(dimensions).expect("fixed-size embedding dimensions should fit within i32"),
    )
}

fn sql_float32_list_literal(values: &[f32]) -> String {
    let values = values
        .iter()
        .map(|value| format!("{value:.9}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn cosine_similarity(left: &[f32], right: &[f32]) -> Option<f32> {
    if left.len() != right.len() || left.is_empty() {
        return None;
    }

    let mut dot = 0.0_f32;
    let mut left_norm = 0.0_f32;
    let mut right_norm = 0.0_f32;
    for (lhs, rhs) in left.iter().zip(right.iter()) {
        dot += lhs * rhs;
        left_norm += lhs * lhs;
        right_norm += rhs * rhs;
    }

    let denom = left_norm.sqrt() * right_norm.sqrt();
    (denom > 0.0).then_some(dot / denom)
}

fn reciprocal_rank(rank: Option<usize>) -> f32 {
    rank.map_or(0.0, |value| {
        let value = u16::try_from(value).expect("rank should fit within u16");
        1.0 / (10.0 + f32::from(value))
    })
}

fn semantic_candidate_cmp(left: &SemanticCandidate, right: &SemanticCandidate) -> Ordering {
    right
        .score
        .total_cmp(&left.score)
        .then_with(|| recall_rank_key(&right.record).cmp(&recall_rank_key(&left.record)))
        .then_with(|| left.record.id().as_str().cmp(right.record.id().as_str()))
}

fn search_ranked_candidate_cmp(left: &RankedMemoryRecord, right: &RankedMemoryRecord) -> Ordering {
    let left_score = reciprocal_rank(left.lexical_rank) + reciprocal_rank(left.semantic_rank);
    let right_score = reciprocal_rank(right.lexical_rank) + reciprocal_rank(right.semantic_rank);

    right_score
        .total_cmp(&left_score)
        .then_with(|| {
            right
                .semantic_score
                .unwrap_or_default()
                .total_cmp(&left.semantic_score.unwrap_or_default())
        })
        .then_with(|| {
            right
                .record
                .updated_at()
                .value()
                .cmp(&left.record.updated_at().value())
        })
        .then_with(|| left.record.id().as_str().cmp(right.record.id().as_str()))
}

fn recall_ranked_candidate_cmp(left: &RankedMemoryRecord, right: &RankedMemoryRecord) -> Ordering {
    let left_score = reciprocal_rank(left.lexical_rank) + reciprocal_rank(left.semantic_rank);
    let right_score = reciprocal_rank(right.lexical_rank) + reciprocal_rank(right.semantic_rank);

    right_score
        .total_cmp(&left_score)
        .then_with(|| {
            right
                .semantic_score
                .unwrap_or_default()
                .total_cmp(&left.semantic_score.unwrap_or_default())
        })
        .then_with(|| recall_rank_key(&right.record).cmp(&recall_rank_key(&left.record)))
        .then_with(|| left.record.id().as_str().cmp(right.record.id().as_str()))
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
    signals: MatchSignals,
) -> RecallEntry {
    let mut reasons = Vec::new();
    if record.pin_state().is_pinned() {
        reasons.push(RecallReason::Pinned);
    }
    if query.scope().is_some() {
        reasons.push(RecallReason::ScopeFilter);
    }
    if signals.lexical && signals.semantic {
        reasons.push(RecallReason::HybridMatch);
    } else if signals.lexical {
        reasons.push(RecallReason::TextMatch);
    } else if signals.semantic {
        reasons.push(RecallReason::SemanticMatch);
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

fn parts_to_system_time(
    secs: u64,
    nanos: u32,
    field: &'static str,
) -> Result<SystemTime, LanceDbError> {
    UNIX_EPOCH
        .checked_add(Duration::new(secs, nanos))
        .ok_or(LanceDbError::InvalidTimestamp { field })
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
    use std::{collections::BTreeMap, path::Path, sync::Arc};

    use arrow_array::Float32Array;
    use mnemix_core::{
        BranchName, BranchRequest, CheckpointName, CheckpointSelector, CleanupMode, CloneKind,
        Confidence, DisclosureDepth, ImportStageRequest, Importance, OptimizeRequest,
        PreOperationCheckpointPolicy, QueryLimit, RecallQuery, RestoreRequest, RetentionPolicy,
        RetrievalMode, ScopeId, SearchQuery, StatsQuery, TagName,
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

    struct TestEmbeddingProvider {
        model_id: &'static str,
        dimensions: u32,
    }

    impl EmbeddingProvider for TestEmbeddingProvider {
        fn model_id(&self) -> &str {
            self.model_id
        }

        fn dimensions(&self) -> u32 {
            self.dimensions
        }

        fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingProviderError> {
            let value = text_len_value(text);
            let dimensions =
                usize::try_from(self.dimensions).expect("dimensions should fit within usize");
            Ok(vec![value; dimensions])
        }
    }

    struct SemanticTestEmbeddingProvider;

    impl EmbeddingProvider for SemanticTestEmbeddingProvider {
        fn model_id(&self) -> &'static str {
            "semantic-test"
        }

        fn dimensions(&self) -> u32 {
            3
        }

        fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingProviderError> {
            let lowered = text.to_ascii_lowercase();
            if lowered.contains("format")
                || lowered.contains("style")
                || lowered.contains("handbook")
                || lowered.contains("output")
            {
                Ok(vec![1.0, 0.0, 0.0])
            } else if lowered.contains("checkpoint") || lowered.contains("restore") {
                Ok(vec![0.0, 1.0, 0.0])
            } else {
                Ok(vec![0.0, 0.0, 1.0])
            }
        }
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

    fn embedding_snapshot_for_table(
        backend: &LanceDbBackend,
        table: &Table,
        id: &str,
    ) -> (Option<String>, Option<u32>, Option<Vec<f32>>) {
        let batches = backend
            .block_on(async {
                let stream: SendableRecordBatchStream = table
                    .query()
                    .select(Select::Columns(vec![
                        "embedding_model".to_owned(),
                        "embedding_dimensions".to_owned(),
                        "embedding".to_owned(),
                    ]))
                    .only_if(string_filter("id", id))
                    .limit(1)
                    .execute()
                    .await?;
                let batches: Vec<RecordBatch> = stream.try_collect().await?;
                Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
            })
            .expect("embedding query should succeed");
        let batch = batches.first().expect("embedding row should exist");

        let model = batch
            .column_by_name("embedding_model")
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .and_then(|column| (!column.is_null(0)).then(|| column.value(0).to_owned()));
        let dimensions = batch
            .column_by_name("embedding_dimensions")
            .and_then(|column| column.as_any().downcast_ref::<UInt32Array>())
            .and_then(|column| (!column.is_null(0)).then(|| column.value(0)));
        let values = batch
            .column_by_name("embedding")
            .and_then(|column| column.as_any().downcast_ref::<ListArray>())
            .and_then(|column| {
                (!column.is_null(0)).then(|| {
                    column
                        .value(0)
                        .as_any()
                        .downcast_ref::<Float32Array>()
                        .expect("embedding values should be Float32")
                        .values()
                        .to_vec()
                })
            });

        (model, dimensions, values)
    }

    fn embedding_snapshot(
        backend: &LanceDbBackend,
        id: &str,
    ) -> (Option<String>, Option<u32>, Option<Vec<f32>>) {
        embedding_snapshot_for_table(backend, &backend.memories, id)
    }

    fn fixed_size_embedding_snapshot(
        backend: &LanceDbBackend,
        id: &str,
    ) -> (Option<String>, Option<Vec<f32>>) {
        let table = backend
            .memory_embeddings
            .as_ref()
            .expect("memory_embeddings table should exist");
        let batches = backend
            .block_on(async {
                let stream: SendableRecordBatchStream = table
                    .query()
                    .select(Select::Columns(vec![
                        "embedding_model".to_owned(),
                        "embedding".to_owned(),
                    ]))
                    .only_if(string_filter("memory_id", id))
                    .limit(1)
                    .execute()
                    .await?;
                let batches: Vec<RecordBatch> = stream.try_collect().await?;
                Ok::<Vec<RecordBatch>, lancedb::Error>(batches)
            })
            .expect("fixed-size embedding query should succeed");
        let batch = batches
            .first()
            .expect("fixed-size embedding row should exist");

        let model = batch
            .column_by_name("embedding_model")
            .and_then(|column| column.as_any().downcast_ref::<StringArray>())
            .and_then(|column| (!column.is_null(0)).then(|| column.value(0).to_owned()));
        let values = batch
            .column_by_name("embedding")
            .and_then(|column| column.as_any().downcast_ref::<FixedSizeListArray>())
            .map(|column| {
                column
                    .value(0)
                    .as_any()
                    .downcast_ref::<Float32Array>()
                    .expect("fixed-size embedding values should be Float32")
                    .values()
                    .to_vec()
            });

        (model, values)
    }

    fn clear_list_embedding(backend: &LanceDbBackend, id: &str) {
        backend
            .block_on(
                backend
                    .memories
                    .update()
                    .only_if(string_filter("id", id))
                    .column("embedding_model", "NULL")
                    .column("embedding_dimensions", "NULL")
                    .column("embedding_updated_at_secs", "NULL")
                    .column("embedding_updated_at_nanos", "NULL")
                    .column("embedding", "NULL")
                    .execute(),
            )
            .expect("embedding columns should clear");
    }

    fn write_vector_settings(
        backend: &LanceDbBackend,
        settings: &VectorSettings,
    ) -> Result<(), LanceDbError> {
        backend.write_vector_settings(settings)
    }

    fn text_len_value(text: &str) -> f32 {
        let len = u16::try_from(text.len()).expect("test text length should fit within u16");
        f32::from(len)
    }

    fn init_legacy_store(path: &Path) {
        std::fs::create_dir_all(path).expect("legacy store path should be created");

        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime should build");
        let uri = path.to_string_lossy().to_string();
        let connection = runtime
            .block_on(connect(&uri).execute())
            .expect("legacy connection should open");

        let memories = runtime
            .block_on(
                connection
                    .create_empty_table(MEMORIES_TABLE, memories_schema(LEGACY_SCHEMA_VERSION))
                    .execute(),
            )
            .expect("legacy memories table should create");
        runtime
            .block_on(
                connection
                    .create_empty_table(CHECKPOINTS_TABLE, checkpoints_schema())
                    .execute(),
            )
            .expect("legacy checkpoints table should create");
        let schema_metadata = runtime
            .block_on(
                connection
                    .create_empty_table(SCHEMA_METADATA_TABLE, schema_metadata_schema())
                    .execute(),
            )
            .expect("legacy schema metadata should create");

        let schema = schema_metadata_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(UInt64Array::from(vec![LEGACY_SCHEMA_VERSION]))],
        )
        .expect("legacy schema batch should build");
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        runtime
            .block_on(schema_metadata.add(reader).execute())
            .expect("legacy schema version row should insert");

        let memory = build_memory(
            "memory:legacy",
            "repo:mnemix",
            "Legacy memory",
            "Legacy stores should remain writable after vector scaffolding lands.",
        );
        let (schema, batch) = memory_record_batch(&memory, LEGACY_SCHEMA_VERSION, None)
            .expect("legacy memory batch should build");
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        runtime
            .block_on(memories.add(reader).execute())
            .expect("legacy memory should insert");
    }

    fn init_v3_store(path: &Path) {
        std::fs::create_dir_all(path).expect("v3 store path should be created");

        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("runtime should build");
        let uri = path.to_string_lossy().to_string();
        let connection = runtime
            .block_on(connect(&uri).execute())
            .expect("v3 connection should open");

        runtime
            .block_on(
                connection
                    .create_empty_table(
                        MEMORIES_TABLE,
                        memories_schema(PERSISTED_EMBEDDING_SCHEMA_VERSION),
                    )
                    .execute(),
            )
            .expect("v3 memories table should create");
        runtime
            .block_on(
                connection
                    .create_empty_table(CHECKPOINTS_TABLE, checkpoints_schema())
                    .execute(),
            )
            .expect("v3 checkpoints table should create");
        let schema_metadata = runtime
            .block_on(
                connection
                    .create_empty_table(SCHEMA_METADATA_TABLE, schema_metadata_schema())
                    .execute(),
            )
            .expect("v3 schema metadata should create");
        runtime
            .block_on(
                connection
                    .create_empty_table(VECTOR_CONFIG_TABLE, vector_config_schema())
                    .execute(),
            )
            .expect("v3 vector config should create");

        let schema = schema_metadata_schema();
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![Arc::new(UInt64Array::from(vec![
                PERSISTED_EMBEDDING_SCHEMA_VERSION,
            ]))],
        )
        .expect("v3 schema batch should build");
        let reader = Box::new(RecordBatchIterator::new(
            vec![Ok(batch)].into_iter(),
            schema,
        ));
        runtime
            .block_on(schema_metadata.add(reader).execute())
            .expect("v3 schema version row should insert");
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
    fn init_sets_default_vector_config() {
        let (_temp_dir, backend) = new_backend();

        assert_eq!(backend.vector_settings(), &VectorSettings::default());
        assert!(!backend.has_embedding_provider());
        assert!(!backend.can_embed_on_write());
        let status = backend.vector_status().expect("vector status should load");
        assert_eq!(status.total_memories(), 0);
        assert_eq!(status.embedded_memories(), 0);
        assert_eq!(status.embedding_coverage_percent(), 0);
        assert!(!status.semantic_retrieval_available());
        assert!(!status.vector_index().available());
        assert!(!status.indexable_embedding_storage());
    }

    #[test]
    fn enable_vectors_persists_settings_on_current_store() {
        let (_temp_dir, mut backend) = new_backend();

        let settings = backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed");

        assert!(settings.vectors_enabled());
        assert_eq!(settings.embedding_model(), Some("test-embedder"));
        assert_eq!(settings.embedding_dimensions(), Some(3));
        assert_eq!(backend.vector_settings(), &settings);
    }

    #[test]
    fn remember_with_auto_embed_persists_embedding_payload() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");

        let memory = build_memory(
            "memory:auto-embed",
            "repo:mnemix",
            "Auto embedded memory",
            "Remember writes should persist embedding values when enabled.",
        );
        let expected_value = text_len_value(memory.fts_text());
        backend
            .remember(memory)
            .expect("memory should store with embedding");

        let (model, dimensions, embedding) = embedding_snapshot(&backend, "memory:auto-embed");
        assert_eq!(model.as_deref(), Some("test-embedder"));
        assert_eq!(dimensions, Some(3));
        assert_eq!(embedding, Some(vec![expected_value; 3]));
        let (fixed_model, fixed_embedding) =
            fixed_size_embedding_snapshot(&backend, "memory:auto-embed");
        assert_eq!(fixed_model.as_deref(), Some("test-embedder"));
        assert_eq!(fixed_embedding, Some(vec![expected_value; 3]));
        let status = backend.vector_status().expect("vector status should load");
        assert_eq!(status.embedded_memories(), 1);
        assert_eq!(status.total_memories(), 1);
        assert_eq!(status.embedding_coverage_percent(), 100);
    }

    #[test]
    fn vector_enabled_backend_advertises_semantic_capabilities() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed");

        assert!(backend.capabilities().supports_semantic_search());
        assert!(backend.capabilities().supports_hybrid_search());
    }

    #[test]
    fn enable_vectors_on_v4_store_creates_indexable_embedding_table() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");

        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed");

        assert!(backend.memory_embeddings.is_some());
        let status = backend.vector_status().expect("vector status should load");
        assert!(status.persisted_embedding_storage());
        assert!(status.indexable_embedding_storage());
        assert!(
            status
                .vector_index()
                .reason()
                .expect("vector index reason")
                .contains("no persisted embeddings yet")
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
    fn open_legacy_store_preserves_schema_version_and_writes() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        init_legacy_store(temp_dir.path());

        let mut backend = LanceDbBackend::open(temp_dir.path()).expect("backend should open");
        assert_eq!(
            backend.schema_version().expect("schema version"),
            LEGACY_SCHEMA_VERSION
        );
        assert_eq!(backend.vector_settings(), &VectorSettings::default());

        backend
            .remember(build_memory(
                "memory:new-on-legacy",
                "repo:mnemix",
                "Legacy write path",
                "Legacy stores should continue using the v1 row shape.",
            ))
            .expect("legacy store should remain writable");

        let legacy = backend
            .get(&MemoryId::try_from("memory:legacy").expect("valid legacy id"))
            .expect("legacy lookup should succeed");
        let inserted = backend
            .get(&MemoryId::try_from("memory:new-on-legacy").expect("valid inserted id"))
            .expect("inserted lookup should succeed");
        let results = backend
            .search(
                &SearchQuery::new(
                    "write path",
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
                    QueryLimit::new(10).expect("valid limit"),
                )
                .expect("search query should build"),
            )
            .expect("search should succeed");

        assert!(legacy.is_some());
        assert!(inserted.is_some());
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn open_v3_store_reports_non_indexable_embedding_storage() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        init_v3_store(temp_dir.path());

        let mut backend = LanceDbBackend::open(temp_dir.path()).expect("backend should open");
        assert_eq!(
            backend.schema_version().expect("schema version"),
            PERSISTED_EMBEDDING_SCHEMA_VERSION
        );
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed on v3 store");

        assert!(backend.memory_embeddings.is_none());
        let status = backend.vector_status().expect("vector status should load");
        assert!(status.persisted_embedding_storage());
        assert!(!status.indexable_embedding_storage());
        assert!(
            status
                .vector_index()
                .reason()
                .expect("vector index reason")
                .contains("schema-v3")
        );
    }

    #[test]
    fn enable_vectors_rejects_legacy_store() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        init_legacy_store(temp_dir.path());
        let mut backend = LanceDbBackend::open(temp_dir.path()).expect("backend should open");

        let result = backend.enable_vectors(
            &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
        );

        assert!(matches!(
            result,
            Err(LanceDbError::InvalidVectorSettings { details })
            if details.contains("schema version 3")
        ));
    }

    #[test]
    fn backfill_plan_counts_candidate_memories() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .remember(build_memory(
                "memory:backfill",
                "repo:mnemix",
                "Backfill candidate",
                "Plan mode should count this memory.",
            ))
            .expect("memory should store");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed");

        let result = backend
            .backfill_embeddings(&EmbeddingBackfillRequest::plan())
            .expect("backfill plan should succeed");

        assert!(!result.apply_writes());
        assert_eq!(result.candidate_memories(), 1);
        assert_eq!(result.updated_memories(), 0);
    }

    #[test]
    fn backfill_apply_requires_embedding_provider() {
        let (_temp_dir, mut backend) = new_backend();
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed");

        let result = backend.backfill_embeddings(&EmbeddingBackfillRequest::apply());

        assert!(matches!(
            result,
            Err(LanceDbError::Core(CoreError::CapabilityUnavailable {
                capability: "embedding_provider"
            }))
        ));
    }

    #[test]
    fn backfill_apply_persists_embeddings() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should initialize");
        let memory = build_memory(
            "memory:backfill-apply",
            "repo:mnemix",
            "Backfill target",
            "Apply mode should persist embeddings for existing rows.",
        );
        let expected_value = text_len_value(memory.fts_text());
        backend
            .remember(memory)
            .expect("memory should store before enablement");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3).expect("enable request should build"),
            )
            .expect("vector enablement should succeed");

        let result = backend
            .backfill_embeddings(&EmbeddingBackfillRequest::apply())
            .expect("backfill apply should succeed");

        assert!(result.apply_writes());
        assert_eq!(result.candidate_memories(), 1);
        assert_eq!(result.updated_memories(), 1);

        let (model, dimensions, embedding) = embedding_snapshot(&backend, "memory:backfill-apply");
        assert_eq!(model.as_deref(), Some("test-embedder"));
        assert_eq!(dimensions, Some(3));
        assert_eq!(embedding, Some(vec![expected_value; 3]));
        let (fixed_model, fixed_embedding) =
            fixed_size_embedding_snapshot(&backend, "memory:backfill-apply");
        assert_eq!(fixed_model.as_deref(), Some("test-embedder"));
        assert_eq!(fixed_embedding, Some(vec![expected_value; 3]));
    }

    #[test]
    fn backfill_only_targets_missing_embeddings() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should initialize");
        backend
            .remember(build_memory(
                "memory:needs-backfill",
                "repo:mnemix",
                "Needs backfill",
                "Stored before vectors were enabled.",
            ))
            .expect("pre-vector memory should store");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:already-embedded",
                "repo:mnemix",
                "Already embedded",
                "Stored after vectors were enabled.",
            ))
            .expect("post-vector memory should store");

        let plan = backend
            .backfill_embeddings(&EmbeddingBackfillRequest::plan())
            .expect("backfill plan should succeed");
        assert_eq!(plan.candidate_memories(), 1);
        assert_eq!(plan.updated_memories(), 0);

        let apply = backend
            .backfill_embeddings(&EmbeddingBackfillRequest::apply())
            .expect("backfill apply should succeed");
        assert_eq!(apply.candidate_memories(), 1);
        assert_eq!(apply.updated_memories(), 1);
    }

    #[test]
    fn vector_status_reports_partial_embedding_coverage() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should initialize");
        backend
            .remember(build_memory(
                "memory:plain-one",
                "repo:mnemix",
                "Plain memory",
                "Stored before vectors are enabled.",
            ))
            .expect("plain memory should store");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:embedded-two",
                "repo:mnemix",
                "Embedded memory",
                "Stored after vectors are enabled.",
            ))
            .expect("embedded memory should store");

        let status = backend.vector_status().expect("vector status should load");
        assert_eq!(status.total_memories(), 2);
        assert_eq!(status.embedded_memories(), 1);
        assert_eq!(status.embedding_coverage_percent(), 50);
        assert!(status.semantic_retrieval_available());
        assert!(status.indexable_embedding_storage());
        assert!(
            status
                .vector_index()
                .reason()
                .expect("vector index reason")
                .contains("populated")
        );
    }

    #[test]
    fn semantic_only_search_returns_embedding_matches() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:semantic-style",
                "repo:mnemix",
                "Formatting handbook",
                "Formatting guidance for CLI output snapshots.",
            ))
            .expect("semantic candidate should store");
        backend
            .remember(build_memory(
                "memory:semantic-restore",
                "repo:mnemix",
                "Restore guide",
                "Checkpoint recovery workflow for snapshots.",
            ))
            .expect("non-matching candidate should store");

        let results = backend
            .search(
                &SearchQuery::new_with_mode(
                    "style guide",
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
                    QueryLimit::new(5).expect("valid limit"),
                    RetrievalMode::SemanticOnly,
                )
                .expect("search query should build"),
            )
            .expect("semantic search should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id().as_str(), "memory:semantic-style");
    }

    #[test]
    fn semantic_only_search_matches_mark_semantic_results() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:semantic-match",
                "repo:mnemix",
                "Formatting handbook",
                "Formatting guidance for CLI output snapshots.",
            ))
            .expect("semantic candidate should store");
        backend
            .remember(build_memory(
                "memory:semantic-other",
                "repo:mnemix",
                "Restore guide",
                "Checkpoint recovery workflow for snapshots.",
            ))
            .expect("non-matching candidate should store");

        let results = backend
            .search_matches(
                &SearchQuery::new_with_mode(
                    "style guide",
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
                    QueryLimit::new(5).expect("valid limit"),
                    RetrievalMode::SemanticOnly,
                )
                .expect("search query should build"),
            )
            .expect("semantic search should succeed");

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].record().id().as_str(), "memory:semantic-match");
        assert!(!results[0].lexical_match());
        assert!(results[0].semantic_match());
        assert!(results[0].semantic_score().is_some());
    }

    #[test]
    fn semantic_only_search_uses_fixed_size_embeddings_on_v4_store() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:semantic-v4",
                "repo:mnemix",
                "Formatting handbook",
                "Formatting guidance for CLI output snapshots.",
            ))
            .expect("semantic candidate should store");

        clear_list_embedding(&backend, "memory:semantic-v4");
        let (model, dimensions, list_embedding) =
            embedding_snapshot(&backend, "memory:semantic-v4");
        assert_eq!(model, None);
        assert_eq!(dimensions, None);
        assert_eq!(list_embedding, None);
        let (fixed_model, fixed_embedding) =
            fixed_size_embedding_snapshot(&backend, "memory:semantic-v4");
        assert_eq!(fixed_model.as_deref(), Some("semantic-test"));
        assert_eq!(fixed_embedding, Some(vec![1.0, 0.0, 0.0]));

        let results = backend
            .search(
                &SearchQuery::new_with_mode(
                    "style guide",
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
                    QueryLimit::new(5).expect("valid limit"),
                    RetrievalMode::SemanticOnly,
                )
                .expect("search query should build"),
            )
            .expect("semantic search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id().as_str(), "memory:semantic-v4");
    }

    #[test]
    fn semantic_only_search_falls_back_to_v3_list_embeddings() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        init_v3_store(temp_dir.path());

        let mut backend = LanceDbBackend::open_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should open");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed on v3 store");
        backend
            .remember(build_memory(
                "memory:semantic-v3",
                "repo:mnemix",
                "Formatting handbook",
                "Formatting guidance for CLI output snapshots.",
            ))
            .expect("semantic candidate should store");

        let (model, dimensions, list_embedding) =
            embedding_snapshot(&backend, "memory:semantic-v3");
        assert_eq!(model.as_deref(), Some("semantic-test"));
        assert_eq!(dimensions, Some(3));
        assert_eq!(list_embedding, Some(vec![1.0, 0.0, 0.0]));
        assert!(backend.memory_embeddings.is_none());

        let results = backend
            .search(
                &SearchQuery::new_with_mode(
                    "style guide",
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
                    QueryLimit::new(5).expect("valid limit"),
                    RetrievalMode::SemanticOnly,
                )
                .expect("search query should build"),
            )
            .expect("semantic search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id().as_str(), "memory:semantic-v3");
    }

    #[test]
    fn open_with_matching_embedding_provider_reports_runtime_capability() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        let settings = VectorSettings {
            vectors_enabled: true,
            auto_embed_on_write: true,
            embedding_model: Some("test-embedder".to_string()),
            embedding_dimensions: Some(3),
        };
        write_vector_settings(&backend, &settings).expect("vector settings should persist");
        drop(backend);

        let backend = LanceDbBackend::open_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should open with matching provider");

        assert_eq!(backend.vector_settings(), &settings);
        assert!(backend.has_embedding_provider());
        assert!(backend.can_embed_on_write());
    }

    #[test]
    fn open_rejects_provider_model_mismatch() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        write_vector_settings(
            &backend,
            &VectorSettings {
                vectors_enabled: true,
                auto_embed_on_write: true,
                embedding_model: Some("expected-model".to_string()),
                embedding_dimensions: Some(3),
            },
        )
        .expect("vector settings should persist");
        drop(backend);

        let result = LanceDbBackend::open_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "different-model",
                dimensions: 3,
            })),
        );

        assert!(matches!(
            result,
            Err(LanceDbError::InvalidVectorSettings { details })
            if details.contains("expected-model") && details.contains("different-model")
        ));
    }

    #[test]
    fn open_rejects_provider_dimension_mismatch() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        write_vector_settings(
            &backend,
            &VectorSettings {
                vectors_enabled: true,
                auto_embed_on_write: true,
                embedding_model: Some("test-embedder".to_string()),
                embedding_dimensions: Some(8),
            },
        )
        .expect("vector settings should persist");
        drop(backend);

        let result = LanceDbBackend::open_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        );

        assert!(matches!(
            result,
            Err(LanceDbError::InvalidVectorSettings { details })
            if details.contains('8') && details.contains('3')
        ));
    }

    #[test]
    fn open_rejects_auto_embed_without_vectors_enabled() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let backend = LanceDbBackend::init(temp_dir.path()).expect("backend should initialize");
        write_vector_settings(
            &backend,
            &VectorSettings {
                vectors_enabled: false,
                auto_embed_on_write: true,
                embedding_model: None,
                embedding_dimensions: None,
            },
        )
        .expect("vector settings should persist");
        drop(backend);

        let result = LanceDbBackend::open(temp_dir.path());

        assert!(matches!(
            result,
            Err(LanceDbError::InvalidVectorSettings { details })
            if details.contains("auto-embed-on-write")
        ));
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
    fn recall_semantic_only_marks_semantic_matches() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:semantic-summary",
                scope: "repo:mnemix",
                kind: MemoryKind::Summary,
                title: "Formatting handbook",
                summary: "Snapshot presentation guidance",
                detail: "Formatting conventions for CLI output.",
                importance: 90,
                pin_reason: None,
                updated_at_secs: 5,
            }))
            .expect("semantic summary should store");

        let result = backend
            .recall(
                &RecallQuery::builder()
                    .scope(ScopeId::try_from("repo:mnemix").expect("valid scope"))
                    .text("style guide")
                    .expect("text should be valid")
                    .retrieval_mode(RetrievalMode::SemanticOnly)
                    .build()
                    .expect("query should build"),
            )
            .expect("semantic recall should succeed");

        assert_eq!(result.summaries().len(), 1);
        let reasons = result.summaries()[0].explanation().reasons();
        assert!(reasons.contains(&RecallReason::SemanticMatch));
        assert!(!reasons.contains(&RecallReason::TextMatch));
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
    fn recall_hybrid_marks_hybrid_matches() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory_with(MemoryFixture {
                id: "memory:hybrid-summary",
                scope: "repo:mnemix",
                kind: MemoryKind::Summary,
                title: "Formatting handbook",
                summary: "Formatting handbook",
                detail: "Formatting handbook for output rendering.",
                importance: 90,
                pin_reason: None,
                updated_at_secs: 5,
            }))
            .expect("hybrid summary should store");

        let result = backend
            .recall(
                &RecallQuery::builder()
                    .scope(ScopeId::try_from("repo:mnemix").expect("valid scope"))
                    .text("output")
                    .expect("text should be valid")
                    .retrieval_mode(RetrievalMode::Hybrid)
                    .build()
                    .expect("query should build"),
            )
            .expect("hybrid recall should succeed");

        assert_eq!(result.summaries().len(), 1);
        let reasons = result.summaries()[0].explanation().reasons();
        assert!(reasons.contains(&RecallReason::HybridMatch));
        assert!(!reasons.contains(&RecallReason::TextMatch));
        assert!(!reasons.contains(&RecallReason::SemanticMatch));
    }

    #[test]
    fn search_matches_mark_hybrid_results() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(SemanticTestEmbeddingProvider)),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("semantic-test", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:hybrid-search",
                "repo:mnemix",
                "Formatting handbook",
                "Formatting handbook for output rendering.",
            ))
            .expect("hybrid candidate should store");

        let results = backend
            .search_matches(
                &SearchQuery::new_with_mode(
                    "output",
                    Some(ScopeId::try_from("repo:mnemix").expect("valid scope")),
                    QueryLimit::new(5).expect("valid limit"),
                    RetrievalMode::Hybrid,
                )
                .expect("search query should build"),
            )
            .expect("hybrid search should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].record().id().as_str(), "memory:hybrid-search");
        assert!(results[0].lexical_match());
        assert!(results[0].semantic_match());
        assert!(results[0].semantic_score().is_some());
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
    fn staged_import_preserves_embeddings_for_imported_memories() {
        let source_dir = TempDir::new().expect("tempdir should be created");
        let mut source = LanceDbBackend::init_with_options(
            source_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("source backend should initialize");
        source
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("source vector enablement should succeed");
        source
            .remember(build_memory(
                "memory:import-embedded",
                "repo:import-source",
                "Imported memory",
                "Should keep persisted embeddings on the staging branch.",
            ))
            .expect("source memory should store");

        let (_target_dir, mut target) = new_backend();
        let staged = target
            .stage_import(&ImportStageRequest::new(source.path()).with_branch_name(
                BranchName::try_from("imports/with-embeddings").expect("valid branch name"),
            ))
            .expect("import should stage");
        let branch_table = branch_table(&target, staged.branch_name());

        let (model, dimensions, embedding) =
            embedding_snapshot_for_table(&target, &branch_table, "memory:import-embedded");
        assert_eq!(model.as_deref(), Some("test-embedder"));
        assert_eq!(dimensions, Some(3));
        assert!(embedding.is_some());
        let (fixed_model, fixed_embedding) =
            fixed_size_embedding_snapshot(&target, "memory:import-embedded");
        assert_eq!(fixed_model.as_deref(), Some("test-embedder"));
        assert!(fixed_embedding.is_some());
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
    fn export_store_preserves_vector_state_and_fixed_embeddings() {
        let temp_dir = TempDir::new().expect("tempdir should be created");
        let mut backend = LanceDbBackend::init_with_options(
            temp_dir.path(),
            LanceDbOpenOptions::new().embedding_provider(Arc::new(TestEmbeddingProvider {
                model_id: "test-embedder",
                dimensions: 3,
            })),
        )
        .expect("backend should initialize");
        backend
            .enable_vectors(
                &VectorEnableRequest::new("test-embedder", 3)
                    .expect("enable request should build")
                    .with_auto_embed_on_write(true),
            )
            .expect("vector enablement should succeed");
        backend
            .remember(build_memory(
                "memory:export-embedded",
                "repo:mnemix",
                "Embedded export memory",
                "Export should preserve vector state and fixed embeddings.",
            ))
            .expect("memory should store");

        let export_dir = TempDir::new().expect("tempdir should be created");
        let export_path = export_dir.path().join("export-store");
        backend
            .export_store(&export_path)
            .expect("export should succeed");

        let exported = LanceDbBackend::open(&export_path).expect("exported store should open");
        assert!(exported.vector_settings().vectors_enabled());
        assert_eq!(
            exported.vector_settings().embedding_model(),
            Some("test-embedder")
        );
        assert_eq!(exported.vector_settings().embedding_dimensions(), Some(3));
        let (fixed_model, fixed_embedding) =
            fixed_size_embedding_snapshot(&exported, "memory:export-embedded");
        assert_eq!(fixed_model.as_deref(), Some("test-embedder"));
        assert!(fixed_embedding.is_some());
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
