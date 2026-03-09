//! LanceDB-backed storage integration for Temporal Plane.
//!
//! This crate owns the concrete local storage implementation for Temporal
//! Plane, keeping all `LanceDB` details behind the storage traits defined in
//! `temporal-plane-core`.

pub mod backend;

pub use backend::{LanceDbBackend, LanceDbError};
