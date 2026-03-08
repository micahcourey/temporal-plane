//! LanceDB-backed storage integration for Temporal Plane.
//!
//! Concrete storage logic is intentionally deferred until later milestones.

pub mod backend;

/// Returns a short description of the crate role.
#[must_use]
pub fn crate_role() -> &'static str {
    "lancedb-backend"
}
