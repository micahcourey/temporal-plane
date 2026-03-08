//! Error types for product-level Temporal Plane logic.

use thiserror::Error;

/// Top-level error type for the core crate.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Placeholder variant used until milestone 1 defines the full error model.
    #[error("core functionality is not implemented yet")]
    NotImplemented,
}
