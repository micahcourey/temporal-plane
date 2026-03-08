//! Storage-facing traits owned by the core product layer.

use std::collections::BTreeSet;

use crate::{
    Checkpoint, CheckpointRequest, HistoryQuery, MemoryId, MemoryRecord, RecallQuery, SearchQuery,
    StatsQuery, StatsSnapshot, VersionRecord,
};

/// An individually advertised backend capability.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum BackendCapability {
    /// The backend can store and fetch durable memories.
    Remember,
    /// The backend can execute recall and search flows.
    Search,
    /// The backend can inspect version history.
    History,
    /// The backend can create and list checkpoints.
    Checkpoints,
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

    /// Returns `true` when the backend can create and list checkpoints.
    #[must_use]
    pub fn supports_checkpoints(&self) -> bool {
        self.0.contains(&BackendCapability::Checkpoints)
    }
}

/// Base trait for storage backends used by Temporal Plane.
pub trait StorageBackend {
    /// Backend-specific operational error type.
    type Error;

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

/// Supports recall and text-first retrieval flows.
pub trait RecallBackend: StorageBackend {
    /// Returns memory records relevant to a recall request.
    ///
    /// # Errors
    ///
    /// Returns [`Self::Error`](StorageBackend::Error) when recall is
    /// unsupported or execution fails.
    fn recall(&self, query: &RecallQuery) -> Result<Vec<MemoryRecord>, Self::Error>;

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
