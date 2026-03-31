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

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("policy config could not be parsed: {0}")]
    PolicyConfigParse(String),

    #[error("policy config version `{actual}` is unsupported (expected {expected})")]
    UnsupportedPolicyConfigVersion { actual: u16, expected: u16 },

    #[error("policy state could not be parsed: {0}")]
    PolicyStateParse(String),

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
            Self::Io(_) => "io_error",
            Self::PolicyConfigParse(_) => "policy_config_parse_error",
            Self::UnsupportedPolicyConfigVersion { .. } => "unsupported_policy_config_version",
            Self::PolicyStateParse(_) => "policy_state_parse_error",
            Self::MemoryNotFound { .. } => "memory_not_found",
            Self::ScopedHistoryNotSupported => "scoped_history_not_supported",
            Self::StoreNotInitialized { .. } => "store_not_initialized",
        }
    }
}
