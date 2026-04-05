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

    #[error("provider config could not be parsed: {0}")]
    ProviderConfigParse(String),

    #[error("provider config could not be serialized: {0}")]
    ProviderConfigSerialize(String),

    #[error("policy config version `{actual}` is unsupported (expected {expected})")]
    UnsupportedPolicyConfigVersion { actual: u16, expected: u16 },

    #[error("provider config version `{actual}` is unsupported (expected {expected})")]
    UnsupportedProviderConfigVersion { actual: u16, expected: u16 },

    #[error("policy state could not be parsed: {0}")]
    PolicyStateParse(String),

    #[error("provider config home is unavailable; set `MNEMIX_CONFIG_HOME` or a home directory")]
    ProviderConfigHomeUnavailable,

    #[error("provider profile `{name}` was not found")]
    ProviderProfileNotFound { name: String },

    #[error("provider profile `{name}` requires environment variable `{env}`")]
    ProviderSecretMissing { name: String, env: String },

    #[error("provider `{name}` validation or runtime setup failed: {details}")]
    ProviderRuntime { name: String, details: String },

    #[error("provider `{name}` is not compatible with `{command}`: {details}")]
    ProviderStoreIncompatible {
        name: String,
        command: &'static str,
        details: String,
    },

    #[error("`{command}` requires `--provider <NAME>`")]
    ProviderRequired { command: &'static str },

    #[error("memory `{id}` was not found")]
    MemoryNotFound { id: String },

    #[error("scoped history is not implemented yet")]
    ScopedHistoryNotSupported,

    #[error("store `{path}` has not been initialized; run `mnemix init` first")]
    StoreNotInitialized { path: PathBuf },

    #[error("`mnemix ui` does not support `--json`")]
    UiJsonUnsupported,
}

impl CliError {
    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Core(_) => "core_error",
            Self::Backend(_) => "backend_error",
            Self::Json(_) => "json_error",
            Self::Io(_) => "io_error",
            Self::PolicyConfigParse(_) => "policy_config_parse_error",
            Self::ProviderConfigParse(_) => "provider_config_parse_error",
            Self::ProviderConfigSerialize(_) => "provider_config_serialize_error",
            Self::UnsupportedPolicyConfigVersion { .. } => "unsupported_policy_config_version",
            Self::UnsupportedProviderConfigVersion { .. } => "unsupported_provider_config_version",
            Self::PolicyStateParse(_) => "policy_state_parse_error",
            Self::ProviderConfigHomeUnavailable => "provider_config_home_unavailable",
            Self::ProviderProfileNotFound { .. } => "provider_profile_not_found",
            Self::ProviderSecretMissing { .. } => "provider_secret_missing",
            Self::ProviderRuntime { .. } => "provider_runtime_error",
            Self::ProviderStoreIncompatible { .. } => "provider_store_incompatible",
            Self::ProviderRequired { .. } => "provider_required",
            Self::MemoryNotFound { .. } => "memory_not_found",
            Self::ScopedHistoryNotSupported => "scoped_history_not_supported",
            Self::StoreNotInitialized { .. } => "store_not_initialized",
            Self::UiJsonUnsupported => "ui_json_unsupported",
        }
    }
}
