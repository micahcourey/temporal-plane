//! Core domain abstractions for Mnemix.
//!
//! This crate owns the storage-agnostic product model for Mnemix.
//! It defines typed identifiers, domain records, query contracts, retention
//! policies, checkpoint abstractions, and backend capability traits without
//! leaking `LanceDB`, `CLI`, or binding-specific details into the public API.
//!
//! # Example
//!
//! ```
//! use std::collections::BTreeMap;
//!
//! use mnemix_core::{
//!     BackendCapabilities, BackendCapability, Checkpoint, CheckpointBackend, CheckpointRequest,
//!     Confidence, CoreError, DisclosureDepth, HistoryBackend, HistoryQuery, Importance,
//!     MemoryId, MemoryKind, MemoryRecord, MemoryRepository, RecallBackend, RecallEntry,
//!     RecallExplanation, RecallLayer, RecallQuery, RecallReason, RecallResult, ScopeId,
//!     SearchQuery, StatsBackend, StatsQuery, StorageBackend, TagName, VersionNumber,
//! };
//!
//! #[derive(Default)]
//! struct ExampleMemoryStore {
//!     records: Vec<MemoryRecord>,
//!     checkpoints: Vec<Checkpoint>,
//! }
//!
//! impl StorageBackend for ExampleMemoryStore {
//!     type Error = CoreError;
//!
//!     fn capabilities(&self) -> BackendCapabilities {
//!         BackendCapabilities::new([
//!             BackendCapability::Remember,
//!             BackendCapability::Search,
//!             BackendCapability::History,
//!             BackendCapability::Checkpoints,
//!         ])
//!     }
//! }
//!
//! impl MemoryRepository for ExampleMemoryStore {
//!     fn remember(&mut self, record: MemoryRecord) -> Result<MemoryRecord, Self::Error> {
//!         self.records.push(record.clone());
//!         Ok(record)
//!     }
//!
//!     fn get(&self, id: &MemoryId) -> Result<Option<MemoryRecord>, Self::Error> {
//!         Ok(self.records.iter().find(|record| record.id() == id).cloned())
//!     }
//! }
//!
//! impl RecallBackend for ExampleMemoryStore {
//!     fn recall(&self, query: &RecallQuery) -> Result<RecallResult, Self::Error> {
//!         let mut summaries = self
//!             .records
//!             .iter()
//!             .filter(|record| {
//!                 query.scope().is_none_or(|scope| record.scope_id() == scope)
//!                     && query.text().is_none_or(|text| record.fts_text().contains(text))
//!                     && matches!(record.kind(), MemoryKind::Summary)
//!             })
//!             .cloned()
//!             .map(|memory| {
//!                 RecallEntry::new(
//!                     memory,
//!                     RecallExplanation::new(
//!                         RecallLayer::Summary,
//!                         vec![RecallReason::SummaryKind, RecallReason::TextMatch],
//!                     ),
//!                 )
//!             })
//!             .collect::<Vec<_>>();
//!         summaries.truncate(query.limit().value().into());
//!         Ok(RecallResult::new(
//!             query.disclosure_depth(),
//!             Vec::new(),
//!             summaries,
//!             Vec::new(),
//!         ))
//!     }
//!
//!     fn search(&self, query: &SearchQuery) -> Result<Vec<MemoryRecord>, Self::Error> {
//!         let mut matches: Vec<_> = self
//!             .records
//!             .iter()
//!             .filter(|record| {
//!                 record.fts_text().contains(query.text())
//!                     && query.scope().is_none_or(|scope| record.scope_id() == scope)
//!             })
//!             .cloned()
//!             .collect();
//!         matches.truncate(query.limit().value().into());
//!         Ok(matches)
//!     }
//! }
//!
//! impl HistoryBackend for ExampleMemoryStore {
//!     fn history(&self, _query: &HistoryQuery) -> Result<Vec<mnemix_core::VersionRecord>, Self::Error> {
//!         Ok(Vec::new())
//!     }
//! }
//!
//! impl CheckpointBackend for ExampleMemoryStore {
//!     fn checkpoint(&mut self, request: &CheckpointRequest) -> Result<Checkpoint, Self::Error> {
//!         let checkpoint = Checkpoint::new(
//!             request.name().clone(),
//!             VersionNumber::new(self.records.len() as u64),
//!             request.description().map(ToOwned::to_owned),
//!         );
//!         self.checkpoints.push(checkpoint.clone());
//!         Ok(checkpoint)
//!     }
//!
//!     fn list_checkpoints(&self) -> Result<Vec<Checkpoint>, Self::Error> {
//!         Ok(self.checkpoints.clone())
//!     }
//! }
//!
//! impl StatsBackend for ExampleMemoryStore {
//!     fn stats(&self, query: &StatsQuery) -> Result<mnemix_core::StatsSnapshot, Self::Error> {
//!         let pinned = self.records.iter().filter(|record| record.pin_state().is_pinned()).count() as u64;
//!         Ok(mnemix_core::StatsSnapshot::new(
//!             self.records.len() as u64,
//!             pinned,
//!             self.checkpoints.len() as u64,
//!             query.scope().cloned(),
//!             self.checkpoints.last().map(|checkpoint| checkpoint.name().clone()),
//!         ))
//!     }
//! }
//!
//! let scope = ScopeId::try_from("repo:mnemix")?;
//! let memory = MemoryRecord::builder(
//!     MemoryId::try_from("memory:core-domain")?,
//!     scope.clone(),
//!     MemoryKind::Summary,
//! )
//! .title("Milestone 1 contract")?
//! .summary("Freeze the storage-agnostic domain contract")?
//! .detail("Typed IDs, queries, checkpoints, retention, and traits live in core.")?
//! .importance(Importance::new(90)?)
//! .confidence(Confidence::new(95)?)
//! .add_tag(TagName::try_from("milestone-1")?)
//! .metadata(BTreeMap::from([("owner".to_string(), "core".to_string())]))
//! .build()?;
//!
//! let mut store = ExampleMemoryStore::default();
//! let stored = store.remember(memory)?;
//! let recall = RecallQuery::builder()
//!     .scope(scope)
//!     .text("contract")?
//!     .disclosure_depth(DisclosureDepth::SummaryThenPinned)
//!     .build()?;
//! let results = store.recall(&recall)?;
//! let checkpoint = store.checkpoint(
//!     &CheckpointRequest::new(
//!         mnemix_core::CheckpointName::try_from("milestone-1-freeze")?,
//!         Some("first domain contract".to_string()),
//!     ),
//! )?;
//!
//! assert_eq!(stored.title(), "Milestone 1 contract");
//! assert_eq!(results.count(), 1);
//! assert_eq!(checkpoint.version().value(), 1);
//! # Ok::<(), CoreError>(())
//! ```

mod identifiers;

pub mod branches;
pub mod checkpoints;
pub mod errors;
pub mod maintenance;
pub mod memory;
pub mod query;
pub mod retention;
pub mod traits;

pub use branches::{BranchListResult, BranchName, BranchRecord, BranchRequest, BranchStatus};
pub use checkpoints::{
    Checkpoint, CheckpointRequest, CheckpointSelector, CheckpointSummary, RestoreRequest,
    RestoreResult, VersionNumber, VersionRecord,
};
pub use errors::CoreError;
pub use identifiers::{
    CheckpointName, EntityName, MemoryId, RecordedAt, ScopeId, SessionId, SourceRef, TagName,
    ToolName,
};
pub use maintenance::{
    CloneInfo, CloneKind, ImportStageRequest, ImportStageResult, OptimizeRequest, OptimizeResult,
};
pub use memory::{Confidence, Importance, MemoryKind, MemoryRecord, MemoryRecordBuilder, PinState};
pub use query::{
    DisclosureDepth, HistoryQuery, QueryLimit, RecallEntry, RecallExplanation, RecallLayer,
    RecallQuery, RecallReason, RecallResult, SearchQuery, StatsQuery, StatsSnapshot,
};
pub use retention::{
    CheckpointProtection, CleanupMode, PreCleanupCheckpointPolicy, PreOperationCheckpointPolicy,
    RetentionPolicy,
};
pub use traits::{
    AdvancedStorageBackend, BackendCapabilities, BackendCapability, CheckpointBackend,
    HistoryBackend, MemoryRepository, OptimizeBackend, PinningBackend, RecallBackend,
    RestoreBackend, StatsBackend, StorageBackend,
};
