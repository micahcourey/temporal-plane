//! Placeholder backend implementation.

use temporal_plane_core::traits::{BackendCapabilities, StorageBackend};
use thiserror::Error;

/// Backend-local error type for the `LanceDB` adapter crate.
#[derive(Debug, Error)]
pub enum LanceDbError {
    /// Placeholder until concrete `LanceDB` operations are implemented.
    #[error("lancedb backend operation is not implemented yet")]
    NotImplemented,
}

/// Placeholder backend type for the workspace baseline.
#[derive(Debug, Default)]
pub struct LanceDbBackend;

impl StorageBackend for LanceDbBackend {
    type Error = LanceDbError;

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::new([])
    }
}
