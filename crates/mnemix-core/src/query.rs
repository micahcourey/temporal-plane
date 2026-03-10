//! Product-level query contracts and inspection results.

use serde::{Deserialize, Serialize};

use crate::{CheckpointName, CoreError, MemoryRecord, ScopeId};

const MAX_QUERY_LIMIT: u16 = 1_000;

fn validate_query_text(field: &'static str, value: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyValue { field });
    }
    Ok(trimmed.to_owned())
}

/// A bounded limit used across query requests.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct QueryLimit(u16);

impl QueryLimit {
    /// Creates a validated limit.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the limit is zero or exceeds the supported
    /// maximum.
    pub fn new(value: u16) -> Result<Self, CoreError> {
        if value == 0 {
            return Err(CoreError::OutOfRange {
                field: "limit",
                min: 1,
                max: u64::from(MAX_QUERY_LIMIT),
                actual: 0,
            });
        }
        if value > MAX_QUERY_LIMIT {
            return Err(CoreError::OutOfRange {
                field: "limit",
                min: 1,
                max: u64::from(MAX_QUERY_LIMIT),
                actual: u64::from(value),
            });
        }
        Ok(Self(value))
    }

    /// Returns the underlying limit value.
    #[must_use]
    pub const fn value(self) -> u16 {
        self.0
    }
}

impl Default for QueryLimit {
    fn default() -> Self {
        Self(10)
    }
}

/// Controls how deeply recall should expand the context window.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum DisclosureDepth {
    /// Return only summary-level context.
    SummaryOnly,
    /// Return summaries first while allowing pinned memories through.
    #[default]
    SummaryThenPinned,
    /// Return fully expanded detail immediately.
    Full,
}

/// Identifies the retrieval layer an item was surfaced from.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecallLayer {
    /// High-priority pinned context loaded ahead of general recall.
    PinnedContext,
    /// Compact summaries favored before deeper archival memory.
    Summary,
    /// Deeper archival memory expanded on demand.
    Archival,
}

/// Explains why a recall item was surfaced.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum RecallReason {
    /// The item is pinned and therefore received higher priority.
    Pinned,
    /// The item matched the requested scope filter.
    ScopeFilter,
    /// The item matched the requested text hint.
    TextMatch,
    /// The item is a summary memory favored for compact recall.
    SummaryKind,
    /// The item received an importance-based boost to its rank.
    ImportanceBoost,
    /// The item received a recency-based boost to its rank.
    RecencyBoost,
    /// The item was included because archival expansion was requested.
    ArchivalExpansion,
}

/// Machine-readable explanation metadata for a surfaced recall item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallExplanation {
    layer: RecallLayer,
    reasons: Vec<RecallReason>,
}

impl RecallExplanation {
    /// Creates a new explanation payload.
    #[must_use]
    pub fn new(layer: RecallLayer, reasons: Vec<RecallReason>) -> Self {
        Self { layer, reasons }
    }

    /// Returns the retrieval layer that surfaced the item.
    #[must_use]
    pub const fn layer(&self) -> RecallLayer {
        self.layer
    }

    /// Returns the ordered explanation reasons.
    #[must_use]
    pub fn reasons(&self) -> &[RecallReason] {
        &self.reasons
    }
}

/// A single memory item returned from progressive disclosure recall.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallEntry {
    memory: MemoryRecord,
    explanation: RecallExplanation,
}

impl RecallEntry {
    /// Creates a new recall entry.
    #[must_use]
    pub fn new(memory: MemoryRecord, explanation: RecallExplanation) -> Self {
        Self {
            memory,
            explanation,
        }
    }

    /// Returns the surfaced memory.
    #[must_use]
    pub const fn memory(&self) -> &MemoryRecord {
        &self.memory
    }

    /// Returns the explanation metadata.
    #[must_use]
    pub const fn explanation(&self) -> &RecallExplanation {
        &self.explanation
    }
}

/// Layered recall results for progressive disclosure flows.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallResult {
    disclosure_depth: DisclosureDepth,
    pinned_context: Vec<RecallEntry>,
    summaries: Vec<RecallEntry>,
    archival: Vec<RecallEntry>,
}

impl RecallResult {
    /// Creates a new layered recall result.
    #[must_use]
    pub fn new(
        disclosure_depth: DisclosureDepth,
        pinned_context: Vec<RecallEntry>,
        summaries: Vec<RecallEntry>,
        archival: Vec<RecallEntry>,
    ) -> Self {
        Self {
            disclosure_depth,
            pinned_context,
            summaries,
            archival,
        }
    }

    /// Returns the disclosure depth used to build the result.
    #[must_use]
    pub const fn disclosure_depth(&self) -> DisclosureDepth {
        self.disclosure_depth
    }

    /// Returns the pinned-context layer.
    #[must_use]
    pub fn pinned_context(&self) -> &[RecallEntry] {
        &self.pinned_context
    }

    /// Returns the summary layer.
    #[must_use]
    pub fn summaries(&self) -> &[RecallEntry] {
        &self.summaries
    }

    /// Returns the archival layer.
    #[must_use]
    pub fn archival(&self) -> &[RecallEntry] {
        &self.archival
    }

    /// Returns the total number of surfaced items across all layers.
    #[must_use]
    pub fn count(&self) -> usize {
        self.pinned_context.len() + self.summaries.len() + self.archival.len()
    }

    /// Returns `true` when every layer is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }
}

/// A recall request tuned for progressive disclosure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RecallQuery {
    scope: Option<ScopeId>,
    text: Option<String>,
    limit: QueryLimit,
    disclosure_depth: DisclosureDepth,
}

impl RecallQuery {
    /// Starts building a recall query.
    #[must_use]
    pub fn builder() -> RecallQueryBuilder {
        RecallQueryBuilder {
            scope: None,
            text: None,
            limit: QueryLimit::default(),
            disclosure_depth: DisclosureDepth::default(),
        }
    }

    /// Returns the optional scope filter.
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeId> {
        self.scope.as_ref()
    }

    /// Returns the optional text hint.
    #[must_use]
    pub fn text(&self) -> Option<&str> {
        self.text.as_deref()
    }

    /// Returns the result limit.
    #[must_use]
    pub const fn limit(&self) -> QueryLimit {
        self.limit
    }

    /// Returns the requested disclosure depth.
    #[must_use]
    pub const fn disclosure_depth(&self) -> DisclosureDepth {
        self.disclosure_depth
    }
}

/// Builder for [`RecallQuery`].
#[derive(Clone, Debug)]
pub struct RecallQueryBuilder {
    scope: Option<ScopeId>,
    text: Option<String>,
    limit: QueryLimit,
    disclosure_depth: DisclosureDepth,
}

impl RecallQueryBuilder {
    /// Filters recall to a single scope.
    #[must_use]
    pub fn scope(mut self, value: ScopeId) -> Self {
        self.scope = Some(value);
        self
    }

    /// Adds a text hint for ranking and filtering.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the text hint is blank.
    pub fn text(mut self, value: impl Into<String>) -> Result<Self, CoreError> {
        self.text = Some(validate_query_text("recall_text", &value.into())?);
        Ok(self)
    }

    /// Sets the maximum number of results.
    #[must_use]
    pub fn limit(mut self, value: QueryLimit) -> Self {
        self.limit = value;
        self
    }

    /// Selects the disclosure depth.
    #[must_use]
    pub fn disclosure_depth(mut self, value: DisclosureDepth) -> Self {
        self.disclosure_depth = value;
        self
    }

    /// Builds the query.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when both scope and text are omitted.
    pub fn build(self) -> Result<RecallQuery, CoreError> {
        if self.scope.is_none() && self.text.is_none() {
            return Err(CoreError::MissingField {
                field: "scope_or_text",
            });
        }
        Ok(RecallQuery {
            scope: self.scope,
            text: self.text,
            limit: self.limit,
            disclosure_depth: self.disclosure_depth,
        })
    }
}

/// A text-first search request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SearchQuery {
    text: String,
    scope: Option<ScopeId>,
    limit: QueryLimit,
}

impl SearchQuery {
    /// Creates a new search query.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the search text is blank.
    pub fn new(
        text: impl Into<String>,
        scope: Option<ScopeId>,
        limit: QueryLimit,
    ) -> Result<Self, CoreError> {
        Ok(Self {
            text: validate_query_text("search_text", &text.into())?,
            scope,
            limit,
        })
    }

    /// Returns the search text.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the optional scope filter.
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeId> {
        self.scope.as_ref()
    }

    /// Returns the result limit.
    #[must_use]
    pub const fn limit(&self) -> QueryLimit {
        self.limit
    }
}

/// A request for historical inspection over a scope.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoryQuery {
    scope: Option<ScopeId>,
    limit: QueryLimit,
}

impl HistoryQuery {
    /// Creates a history query.
    #[must_use]
    pub fn new(scope: Option<ScopeId>, limit: QueryLimit) -> Self {
        Self { scope, limit }
    }

    /// Returns the optional scope filter.
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeId> {
        self.scope.as_ref()
    }

    /// Returns the result limit.
    #[must_use]
    pub const fn limit(&self) -> QueryLimit {
        self.limit
    }
}

/// A request for high-level product statistics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatsQuery {
    scope: Option<ScopeId>,
}

impl StatsQuery {
    /// Creates a stats query.
    #[must_use]
    pub const fn new(scope: Option<ScopeId>) -> Self {
        Self { scope }
    }

    /// Returns the optional scope filter.
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeId> {
        self.scope.as_ref()
    }
}

/// A machine-readable snapshot of current product statistics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StatsSnapshot {
    total_memories: u64,
    pinned_memories: u64,
    version_count: u64,
    scope: Option<ScopeId>,
    latest_checkpoint: Option<CheckpointName>,
}

impl StatsSnapshot {
    /// Creates a new stats snapshot.
    #[must_use]
    pub const fn new(
        total_memories: u64,
        pinned_memories: u64,
        version_count: u64,
        scope: Option<ScopeId>,
        latest_checkpoint: Option<CheckpointName>,
    ) -> Self {
        Self {
            total_memories,
            pinned_memories,
            version_count,
            scope,
            latest_checkpoint,
        }
    }

    /// Returns the total memory count.
    #[must_use]
    pub const fn total_memories(&self) -> u64 {
        self.total_memories
    }

    /// Returns the pinned memory count.
    #[must_use]
    pub const fn pinned_memories(&self) -> u64 {
        self.pinned_memories
    }

    /// Returns the version count.
    #[must_use]
    pub const fn version_count(&self) -> u64 {
        self.version_count
    }

    /// Returns the scope if the stats were filtered.
    #[must_use]
    pub const fn scope(&self) -> Option<&ScopeId> {
        self.scope.as_ref()
    }

    /// Returns the latest known checkpoint.
    #[must_use]
    pub const fn latest_checkpoint(&self) -> Option<&CheckpointName> {
        self.latest_checkpoint.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use crate::{MemoryId, MemoryKind, ScopeId};

    use super::*;

    fn demo_memory() -> MemoryRecord {
        MemoryRecord::builder(
            MemoryId::try_from("memory:recall").expect("memory id"),
            ScopeId::try_from("repo:mnemix").expect("scope id"),
            MemoryKind::Summary,
        )
        .title("Recall summary")
        .expect("title")
        .summary("Compact recall")
        .expect("summary")
        .detail("Expanded archival detail")
        .expect("detail")
        .build()
        .expect("memory")
    }

    #[test]
    fn query_limit_rejects_zero() {
        let result = QueryLimit::new(0);

        assert_eq!(
            result,
            Err(CoreError::OutOfRange {
                field: "limit",
                min: 1,
                max: 1_000,
                actual: 0,
            })
        );
    }

    #[test]
    fn recall_query_requires_scope_or_text() {
        let result = RecallQuery::builder().build();

        assert_eq!(
            result,
            Err(CoreError::MissingField {
                field: "scope_or_text"
            })
        );
    }

    #[test]
    fn recall_query_preserves_disclosure_depth() {
        let query = RecallQuery::builder()
            .scope(ScopeId::try_from("repo:mnemix").expect("scope"))
            .disclosure_depth(DisclosureDepth::Full)
            .build()
            .expect("query should build");

        assert_eq!(query.disclosure_depth(), DisclosureDepth::Full);
    }

    #[test]
    fn recall_result_counts_all_layers() {
        let entry = RecallEntry::new(
            demo_memory(),
            RecallExplanation::new(
                RecallLayer::Summary,
                vec![RecallReason::SummaryKind, RecallReason::ImportanceBoost],
            ),
        );
        let result = RecallResult::new(
            DisclosureDepth::SummaryThenPinned,
            Vec::new(),
            vec![entry],
            Vec::new(),
        );

        assert_eq!(result.count(), 1);
        assert!(!result.is_empty());
        assert_eq!(
            result.summaries()[0].explanation().layer(),
            RecallLayer::Summary
        );
    }

    #[test]
    fn search_query_requires_non_empty_text() {
        let result = SearchQuery::new("   ", None, QueryLimit::default());

        assert_eq!(
            result,
            Err(CoreError::EmptyValue {
                field: "search_text"
            })
        );
    }

    #[test]
    fn stats_snapshot_preserves_latest_checkpoint() {
        let snapshot = StatsSnapshot::new(
            10,
            2,
            5,
            None,
            Some(CheckpointName::try_from("milestone-1").expect("checkpoint")),
        );

        assert_eq!(
            snapshot.latest_checkpoint().map(CheckpointName::as_str),
            Some("milestone-1")
        );
    }
}
