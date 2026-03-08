//! Core domain abstractions for Temporal Plane.
//!
//! This crate intentionally owns product semantics rather than concrete storage,
//! CLI formatting, or host-specific adapter behavior.

pub mod errors;
pub mod traits;

/// Returns a short description of the crate role.
#[must_use]
pub fn crate_role() -> &'static str {
    "core-domain"
}
