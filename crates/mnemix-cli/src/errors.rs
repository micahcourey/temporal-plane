use std::path::PathBuf;

use thiserror::Error;

use mnemix_core::CoreError;
use mnemix_lancedb::LanceDbError;

#[derive(Debug, Error)]
pub(crate) enum CliError {
    #[error(transparent)]
    Core(#[from] CoreError),

    #[error(transparent)]
    Backend(#[from] LanceDbError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("memory `{id}` was not found")]
    MemoryNotFound { id: String },

    #[error("scoped history is not implemented yet")]
    ScopedHistoryNotSupported,

    #[error("store `{path}` has not been initialized; run `mnemix init` first")]
    StoreNotInitialized { path: PathBuf },
}

impl CliError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Core(_) => "core_error",
            Self::Backend(_) => "backend_error",
            Self::Json(_) => "json_error",
            Self::MemoryNotFound { .. } => "memory_not_found",
            Self::ScopedHistoryNotSupported => "scoped_history_not_supported",
            Self::StoreNotInitialized { .. } => "store_not_initialized",
        }
    }
}
