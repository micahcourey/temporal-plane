//! Storage-facing traits owned by the core product layer.

use std::collections::BTreeSet;
use std::path::Path;

use crate::{
    BranchListResult, BranchName, BranchRecord, BranchRequest, Checkpoint, CheckpointRequest,
    CloneInfo, HistoryQuery, ImportStageRequest, ImportStageResult, MemoryId, MemoryRecord,
    OptimizeRequest, OptimizeResult, RecallQuery, RecallResult, RestoreRequest, RestoreResult,
    SearchQuery, StatsQuery, StatsSnapshot, VersionRecord,
};

/// An individually advertised backend capability.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum BackendCapability {
    /// The backend can store and fetch durable memories.
    Remember,
    /// The backend can explicitly pin and unpin memories.
    Pinning,
    /// The backend can execute recall and search flows.
    Search,
    /// The backend can inspect version history.
    History,
    /// The backend can restore a prior state as a new head version.
    Restore,
    /// The backend can create and list checkpoints.
    Checkpoints,
    /// The backend can run explicit maintenance and cleanup flows.
    Optimize,
    /// The backend can create experimental branches.
    BranchCreate,
    /// The backend can inspect available branches.
    BranchList,
    /// The backend can stage imports onto isolated branches.
    ImportStaging,
    /// The backend can create lightweight shallow clones.
    ShallowClone,
    /// The backend can create full deep clones.
    DeepClone,
}

/// Declares which product-level operations a backend supports.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BackendCapabilities(BTreeSet<BackendCapability>);

impl BackendCapabilities {
    /// Creates a new capability set.
    #[must_use]
    pub fn new(capabilities: impl IntoIterator<Item = BackendCapability>) -> Self {
        Self(capabilities.into_iter().collect())
    }

    /// Returns `true` when the backend supports storing memories.
    #[must_use]
    pub fn supports_remember(&self) -> bool {
        self.0.contains(&BackendCapability::Remember)
    }

    /// Returns `true` when the backend supports explicit pinning operations.
    #[must_use]
    pub fn supports_pinning(&self) -> bool {
        self.0.contains(&BackendCapability::Pinning)
    }

    /// Returns `true` when the backend supports recall and search.
    #[must_use]
    pub fn supports_search(&self) -> bool {
        self.0.contains(&BackendCapability::Search)
    }

    /// Returns `true` when the backend can inspect history or versions.
    #[must_use]
    pub fn supports_history(&self) -> bool {
        self.0.contains(&BackendCapability::History)
    }

    /// Returns `true` when the backend can restore from history.
    #[must_use]
    pub fn supports_restore(&self) -> bool {
        self.0.contains(&BackendCapability::Restore)
    }

    /// Returns `true` when the backend can create and list checkpoints.
    #[must_use]
    pub fn supports_checkpoints(&self) -> bool {
        self.0.contains(&BackendCapability::Checkpoints)
    }

    /// Returns `true` when the backend can run explicit maintenance flows.
    #[must_use]
    pub fn supports_optimize(&self) -> bool {
        self.0.contains(&BackendCapability::Optimize)
    }

    /// Returns `true` when the backend can create branches.
    #[must_use]
    pub fn supports_branch_create(&self) -> bool {
        self.0.contains(&BackendCapability::BranchCreate)
    }

    /// Returns `true` when the backend can inspect branches.
    #[must_use]
    pub fn supports_branch_list(&self) -> bool {
        self.0.contains(&BackendCapability::BranchList)
    }

    /// Returns `true` when the backend can stage imports onto branches.
    #[must_use]
    pub fn supports_import_staging(&self) -> bool {
        self.0.contains(&BackendCapability::ImportStaging)
    }

    /// Returns `true` when the backend can create shallow clones.
    #[must_use]
    pub fn supports_shallow_clone(&self) -> bool {
        self.0.contains(&BackendCapability::ShallowClone)
    }

    /// Returns `true` when the backend can create deep clones.
    #[must_use]
    pub fn supports_deep_clone(&self) -> bool {
        self.0.contains(&BackendCapability::DeepClone)
    }
}

/// Base trait for storage backends used by Mnemix.
pub trait StorageBackend {
    /// Backend-specific operational error type.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Describes the product capabilities exposed by the backend.
    fn capabilities(&self) -> BackendCapabilities;
}

/// Stores and retrieves durable memory records.
pub trait MemoryRepository: StorageBackend {
    /// Persists a memory record.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the backend cannot
    /// store the record.
    fn remember(&mut self, record: MemoryRecord) -> Result<MemoryRecord, Self::Error>;

    /// Looks up a memory record by identifier.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the backend cannot
    /// perform the lookup.
    fn get(&self, id: &MemoryId) -> Result<Option<MemoryRecord>, Self::Error>;
}

/// Supports explicit pinning state transitions for existing memories.
pub trait PinningBackend: StorageBackend {
    /// Pins a memory, optionally updating its existing pin reason.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the backend cannot
    /// perform the update.
    fn pin(&mut self, id: &MemoryId, reason: &str) -> Result<Option<MemoryRecord>, Self::Error>;

    /// Removes a pin from a memory.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the backend cannot
    /// perform the update.
    fn unpin(&mut self, id: &MemoryId) -> Result<Option<MemoryRecord>, Self::Error>;
}

/// Supports recall and text-first retrieval flows.
pub trait RecallBackend: StorageBackend {
    /// Returns layered memory results relevant to a recall request.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when recall is
    /// unsupported or execution fails.
    fn recall(&self, query: &RecallQuery) -> Result<RecallResult, Self::Error>;

    /// Returns memory records relevant to a text-first search request.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when search is
    /// unsupported or execution fails.
    fn search(&self, query: &SearchQuery) -> Result<Vec<MemoryRecord>, Self::Error>;
}

/// Supports history inspection and version listing.
pub trait HistoryBackend: StorageBackend {
    /// Returns version records matching a history request.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when history inspection
    /// is unsupported or fails.
    fn history(&self, query: &HistoryQuery) -> Result<Vec<VersionRecord>, Self::Error>;
}

/// Supports restore flows that create a new current state from history.
pub trait RestoreBackend: StorageBackend {
    /// Restores the current head from a historical version or checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the restore target
    /// cannot be resolved or the restore operation fails.
    fn restore(&mut self, request: &RestoreRequest) -> Result<RestoreResult, Self::Error>;
}

/// Supports creation and inspection of stable checkpoints.
pub trait CheckpointBackend: StorageBackend {
    /// Creates a named checkpoint.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when checkpoint creation
    /// is unsupported or fails.
    fn checkpoint(&mut self, request: &CheckpointRequest) -> Result<Checkpoint, Self::Error>;

    /// Lists available checkpoints.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when checkpoint listing
    /// is unsupported or fails.
    fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, Self::Error>;
}

/// Supports explicit maintenance and cleanup operations.
pub trait OptimizeBackend: StorageBackend {
    /// Runs maintenance for the current store.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when optimization or
    /// cleanup fails.
    fn optimize(&mut self, request: &OptimizeRequest) -> Result<OptimizeResult, Self::Error>;
}

/// Supports advanced branch-aware and clone-aware storage workflows.
pub trait AdvancedStorageBackend: StorageBackend {
    /// Creates a named storage branch.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when branch creation is
    /// unsupported or fails.
    fn create_branch(&mut self, request: &BranchRequest) -> Result<BranchRecord, Self::Error>;

    /// Lists visible storage branches.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when branch listing is
    /// unsupported or fails.
    fn list_branches(&self) -> Result<BranchListResult, Self::Error>;

    /// Deletes a branch that no longer contains staged changes.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the branch cannot
    /// be deleted safely.
    fn delete_branch(&mut self, name: &BranchName) -> Result<(), Self::Error>;

    /// Stages an import onto an isolated branch.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when the import cannot
    /// be staged safely.
    fn stage_import(
        &mut self,
        request: &ImportStageRequest,
    ) -> Result<ImportStageResult, Self::Error>;

    /// Creates a shallow clone of the current store.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when cloning fails.
    fn shallow_clone(&self, destination: &Path) -> Result<CloneInfo, Self::Error>;

    /// Creates a deep clone of the current store.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when cloning fails.
    fn deep_clone(&self, destination: &Path) -> Result<CloneInfo, Self::Error>;
}

/// Supports human-readable and machine-readable inspection statistics.
pub trait StatsBackend: StorageBackend {
    /// Returns a snapshot of product-level statistics.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when stats inspection is
    /// unsupported or fails.
    fn stats(&self, query: &StatsQuery) -> Result<StatsSnapshot, Self::Error>;
}
