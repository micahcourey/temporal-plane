//! LanceDB-backed storage integration for Mnemix.
//!
//! This crate owns the concrete local storage implementation for Mnemix,
//! keeping all `LanceDB` details behind the storage traits defined in
//! `mnemix-core`.

pub mod backend;

pub use backend::{
    EmbeddingBackfillRequest, EmbeddingBackfillResult, EmbeddingProvider, EmbeddingProviderError,
    LanceDbBackend, LanceDbError, LanceDbOpenOptions, VectorEnableRequest, VectorIndexStatus,
    VectorSettings, VectorStatus,
};
