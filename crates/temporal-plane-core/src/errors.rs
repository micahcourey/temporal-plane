//! Error types for storage-agnostic Temporal Plane domain logic.

use thiserror::Error;

/// Top-level error type for the core crate.
#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum CoreError {
    /// A required field was not provided before construction.
    #[error("missing required field `{field}`")]
    MissingField {
        /// The missing field name.
        field: &'static str,
    },

    /// A string field was empty after trimming.
    #[error("field `{field}` cannot be empty")]
    EmptyValue {
        /// The invalid field name.
        field: &'static str,
    },

    /// A field exceeded its supported maximum length.
    #[error("field `{field}` exceeds maximum length of {max} (actual: {actual})")]
    TooLong {
        /// The invalid field name.
        field: &'static str,
        /// The supported maximum length.
        max: usize,
        /// The actual observed length.
        actual: usize,
    },

    /// A numeric field exceeded its allowed range.
    #[error("field `{field}` must be between {min} and {max} inclusive (actual: {actual})")]
    OutOfRange {
        /// The invalid field name.
        field: &'static str,
        /// The inclusive minimum.
        min: u64,
        /// The inclusive maximum.
        max: u64,
        /// The actual observed value.
        actual: u64,
    },

    /// A field contained an unsupported character.
    #[error("field `{field}` contains unsupported character `{character}`")]
    InvalidCharacter {
        /// The invalid field name.
        field: &'static str,
        /// The first unsupported character.
        character: char,
    },

    /// A field used a reserved value that is not allowed by the domain model.
    #[error("field `{field}` uses reserved value `{value}`")]
    ReservedValue {
        /// The invalid field name.
        field: &'static str,
        /// The reserved value that was rejected.
        value: String,
    },

    /// A backend capability was requested from an unsupported storage layer.
    #[error("backend capability `{capability}` is not available")]
    CapabilityUnavailable {
        /// The missing capability name.
        capability: &'static str,
    },
}
