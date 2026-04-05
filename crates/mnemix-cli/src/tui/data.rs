use std::{collections::BTreeSet, time::SystemTime};

use humantime::format_rfc3339;
use mnemix_core::{
    MemoryRecord, QueryLimit, RetrievalMode, ScopeId, SearchQuery, traits::BrowseBackend,
};
use mnemix_lancedb::{LanceDbBackend, SearchMatch};

use crate::errors::CliError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BrowseMode {
    Recent,
    Pinned,
    Search,
}

impl BrowseMode {
    pub(crate) const ALL: [Self; 3] = [Self::Recent, Self::Pinned, Self::Search];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Recent => "Recent",
            Self::Pinned => "Pinned",
            Self::Search => "Search",
        }
    }
}

pub(crate) const RETRIEVAL_MODES: [RetrievalMode; 3] = [
    RetrievalMode::LexicalOnly,
    RetrievalMode::SemanticOnly,
    RetrievalMode::Hybrid,
];

pub(crate) const fn retrieval_mode_label(mode: RetrievalMode) -> &'static str {
    match mode {
        RetrievalMode::LexicalOnly => "Lexical",
        RetrievalMode::SemanticOnly => "Semantic",
        RetrievalMode::Hybrid => "Hybrid",
    }
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct VectorSummary {
    vectors_enabled: bool,
    auto_embed_on_write: bool,
    embedding_model: Option<String>,
    embedding_dimensions: Option<u32>,
    has_embedding_provider: bool,
    semantic_retrieval_available: bool,
    embedded_memories: u64,
    total_memories: u64,
    embedding_coverage_percent: u8,
    vector_index_available: bool,
    vector_index_reason: Option<String>,
}

impl VectorSummary {
    pub(crate) fn load(backend: &LanceDbBackend) -> Result<Self, CliError> {
        let status = backend.vector_status()?;
        let settings = status.settings();

        Ok(Self {
            vectors_enabled: settings.vectors_enabled(),
            auto_embed_on_write: settings.auto_embed_on_write(),
            embedding_model: settings.embedding_model().map(ToOwned::to_owned),
            embedding_dimensions: settings.embedding_dimensions(),
            has_embedding_provider: status.has_embedding_provider(),
            semantic_retrieval_available: status.semantic_retrieval_available(),
            embedded_memories: status.embedded_memories(),
            total_memories: status.total_memories(),
            embedding_coverage_percent: status.embedding_coverage_percent(),
            vector_index_available: status.vector_index().available(),
            vector_index_reason: status.vector_index().reason().map(ToOwned::to_owned),
        })
    }

    #[cfg(test)]
    pub(crate) fn from_parts(
        vectors_enabled: bool,
        has_embedding_provider: bool,
        semantic_retrieval_available: bool,
    ) -> Self {
        Self {
            vectors_enabled,
            auto_embed_on_write: false,
            embedding_model: None,
            embedding_dimensions: None,
            has_embedding_provider,
            semantic_retrieval_available,
            embedded_memories: 0,
            total_memories: 0,
            embedding_coverage_percent: 0,
            vector_index_available: false,
            vector_index_reason: Some("not yet indexed".to_owned()),
        }
    }

    pub(crate) const fn supports_mode(&self, mode: RetrievalMode) -> bool {
        match mode {
            RetrievalMode::LexicalOnly => true,
            RetrievalMode::SemanticOnly | RetrievalMode::Hybrid => {
                self.semantic_retrieval_available
            }
        }
    }

    pub(crate) fn unavailable_reason(&self, mode: RetrievalMode) -> Option<String> {
        if self.supports_mode(mode) {
            return None;
        }

        let mode_label = retrieval_mode_label(mode);
        let reason = if !self.vectors_enabled {
            "vectors are not enabled for this store".to_owned()
        } else if !self.has_embedding_provider {
            "this CLI session does not have an embedding provider attached".to_owned()
        } else {
            "semantic retrieval is not available for the current store".to_owned()
        };

        Some(format!("{mode_label} search unavailable: {reason}."))
    }

    pub(crate) fn show_detailed_snapshot(&self) -> bool {
        self.vectors_enabled
            || self.has_embedding_provider
            || self.embedding_model.is_some()
            || self.embedding_dimensions.is_some()
            || self.vector_index_available
            || self.total_memories > 0
    }

    pub(crate) fn compact_snapshot_line(&self) -> String {
        let summary = if !self.vectors_enabled {
            "disabled".to_owned()
        } else if self.semantic_retrieval_available {
            format!(
                "ready (model={}, coverage={}%)",
                self.embedding_model.as_deref().unwrap_or("configured"),
                self.embedding_coverage_percent
            )
        } else if !self.has_embedding_provider {
            "provider missing".to_owned()
        } else {
            "semantic unavailable".to_owned()
        };
        format!("Vector snapshot: {summary}")
    }

    pub(crate) fn detailed_snapshot_line(&self) -> String {
        format!(
            "Vector snapshot: enabled={} model={} dims={} provider={} semantic={} auto-embed={} coverage={}% ({}/{}) index={}{}",
            self.vectors_enabled,
            self.embedding_model.as_deref().unwrap_or("none"),
            self.embedding_dimensions
                .map_or_else(|| "none".to_owned(), |value| value.to_string()),
            yes_no(self.has_embedding_provider),
            yes_no(self.semantic_retrieval_available),
            yes_no(self.auto_embed_on_write),
            self.embedding_coverage_percent,
            self.embedded_memories,
            self.total_memories,
            yes_no(self.vector_index_available),
            self.vector_index_reason
                .as_deref()
                .map(|reason| format!(" ({reason})"))
                .unwrap_or_default(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScopeOption {
    label: String,
    value: Option<ScopeId>,
}

impl ScopeOption {
    pub(crate) fn all() -> Self {
        Self {
            label: "All scopes".to_owned(),
            value: None,
        }
    }

    pub(crate) fn new(scope: ScopeId) -> Self {
        Self {
            label: scope.as_str().to_owned(),
            value: Some(scope),
        }
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }

    pub(crate) fn value(&self) -> Option<&ScopeId> {
        self.value.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MemoryEntry {
    record: MemoryRecord,
    created_date: String,
    updated_date: String,
    search_match: Option<SearchMatchDetails>,
}

impl MemoryEntry {
    pub(crate) fn from_record(record: MemoryRecord) -> Self {
        Self {
            created_date: record_date(record.created_at().value()),
            updated_date: record_date(record.updated_at().value()),
            record,
            search_match: None,
        }
    }

    pub(crate) fn from_search_match(search_match: SearchMatch) -> Self {
        let details = SearchMatchDetails::from_search_match(&search_match);
        let record = search_match.into_record();
        Self {
            created_date: record_date(record.created_at().value()),
            updated_date: record_date(record.updated_at().value()),
            record,
            search_match: Some(details),
        }
    }

    pub(crate) fn record(&self) -> &MemoryRecord {
        &self.record
    }

    pub(crate) fn created_date(&self) -> &str {
        &self.created_date
    }

    pub(crate) fn updated_date(&self) -> &str {
        &self.updated_date
    }

    pub(crate) fn search_match(&self) -> Option<&SearchMatchDetails> {
        self.search_match.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SearchMatchDetails {
    lexical_match: bool,
    semantic_match: bool,
    semantic_score: Option<String>,
}

impl SearchMatchDetails {
    fn from_search_match(search_match: &SearchMatch) -> Self {
        Self {
            lexical_match: search_match.lexical_match(),
            semantic_match: search_match.semantic_match(),
            semantic_score: search_match
                .semantic_score()
                .map(|score| format!("{score:.3}")),
        }
    }

    pub(crate) const fn label(&self) -> Option<&'static str> {
        match (self.lexical_match, self.semantic_match) {
            (true, true) => Some("hybrid"),
            (true, false) => Some("lexical"),
            (false, true) => Some("semantic"),
            (false, false) => None,
        }
    }

    pub(crate) fn semantic_score(&self) -> Option<&str> {
        self.semantic_score.as_deref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BrowserData {
    recent: Vec<MemoryEntry>,
    pinned: Vec<MemoryEntry>,
    scopes: Vec<ScopeOption>,
}

impl BrowserData {
    pub(crate) fn load<B>(backend: &B, limit: QueryLimit) -> Result<Self, CliError>
    where
        B: BrowseBackend,
        CliError: From<B::Error>,
    {
        let recent = backend
            .list_memories(None, limit)?
            .into_iter()
            .map(MemoryEntry::from_record)
            .collect::<Vec<_>>();
        let pinned = backend
            .list_pinned_memories(None, limit)?
            .into_iter()
            .map(MemoryEntry::from_record)
            .collect::<Vec<_>>();
        let scopes = derive_scopes(recent.iter().chain(&pinned));

        Ok(Self {
            recent,
            pinned,
            scopes,
        })
    }

    #[cfg(test)]
    pub(crate) fn from_parts(
        recent: Vec<MemoryEntry>,
        pinned: Vec<MemoryEntry>,
        scopes: Vec<ScopeOption>,
    ) -> Self {
        Self {
            recent,
            pinned,
            scopes,
        }
    }

    pub(crate) fn recent(&self) -> &[MemoryEntry] {
        &self.recent
    }

    pub(crate) fn pinned(&self) -> &[MemoryEntry] {
        &self.pinned
    }

    pub(crate) fn scopes(&self) -> &[ScopeOption] {
        &self.scopes
    }
}

/// The TUI uses `LanceDB` directly here so it can render search provenance from
/// `search_matches`; if another backend needs TUI support, this is the seam to
/// generalize.
pub(crate) fn search_entries(
    backend: &LanceDbBackend,
    query_text: &str,
    scope: Option<ScopeId>,
    limit: QueryLimit,
    retrieval_mode: RetrievalMode,
) -> Result<Vec<MemoryEntry>, CliError> {
    let query = SearchQuery::new_with_mode(query_text.to_owned(), scope, limit, retrieval_mode)?;
    let records = backend.search_matches(&query)?;
    Ok(records
        .into_iter()
        .map(MemoryEntry::from_search_match)
        .collect())
}

pub(crate) fn record_date(value: SystemTime) -> String {
    format_rfc3339(value).to_string()[..10].to_owned()
}

pub(crate) fn validate_date_filter(value: &str) -> Result<(), &'static str> {
    if value.is_empty() {
        return Ok(());
    }

    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err("dates must use YYYY-MM-DD");
    }
    if bytes
        .iter()
        .enumerate()
        .any(|(index, byte)| index != 4 && index != 7 && !byte.is_ascii_digit())
    {
        return Err("dates must use YYYY-MM-DD");
    }

    let year: u32 = value[0..4].parse().map_err(|_| "invalid year")?;
    let month: u32 = value[5..7].parse().map_err(|_| "invalid month")?;
    let day: u32 = value[8..10].parse().map_err(|_| "invalid day")?;

    if year == 0 {
        return Err("year must be greater than 0000");
    }
    if !(1..=12).contains(&month) {
        return Err("month must be between 01 and 12");
    }

    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!("validated month"),
    };
    if !(1..=max_day).contains(&day) {
        return Err("day is out of range for the selected month");
    }

    Ok(())
}

pub(crate) fn matches_date_range(
    entry: &MemoryEntry,
    from: Option<&str>,
    to: Option<&str>,
) -> bool {
    let updated = entry.updated_date();
    let after_from = from.is_none_or(|from_date| updated >= from_date);
    let before_to = to.is_none_or(|to_date| updated <= to_date);
    after_from && before_to
}

fn derive_scopes<'a>(entries: impl Iterator<Item = &'a MemoryEntry>) -> Vec<ScopeOption> {
    let mut scopes = entries
        .map(|entry| entry.record().scope_id().clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(ScopeOption::new)
        .collect::<Vec<_>>();
    scopes.insert(0, ScopeOption::all());
    scopes
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use mnemix_core::{MemoryId, MemoryRecord, RecordedAt, ScopeId, memory::MemoryKind};

    use super::{
        MemoryEntry, RetrievalMode, VectorSummary, matches_date_range, validate_date_filter,
    };

    fn entry_with_updated_date(updated_at: std::time::SystemTime) -> MemoryEntry {
        let record = MemoryRecord::builder(
            MemoryId::new("memory:test").expect("id"),
            ScopeId::new("scope:test").expect("scope"),
            MemoryKind::Decision,
        )
        .title("Title")
        .expect("title")
        .summary("Summary")
        .expect("summary")
        .detail("Detail")
        .expect("detail")
        .updated_at(RecordedAt::new(updated_at))
        .build()
        .expect("record");
        MemoryEntry::from_record(record)
    }

    #[test]
    fn validates_date_filters() {
        assert!(validate_date_filter("").is_ok());
        assert!(validate_date_filter("2026-03-30").is_ok());
        assert!(validate_date_filter("2026-02-29").is_err());
        assert!(validate_date_filter("2026/03/30").is_err());
    }

    #[test]
    fn matches_date_ranges_lexically() {
        let entry = entry_with_updated_date(
            std::time::UNIX_EPOCH + std::time::Duration::from_secs(1_774_867_200),
        );

        assert!(matches_date_range(
            &entry,
            Some("2026-03-30"),
            Some("2026-03-30")
        ));
        assert!(!matches_date_range(&entry, Some("2026-03-31"), None));
        assert!(!matches_date_range(&entry, None, Some("2026-03-29")));
    }

    #[test]
    fn vector_summary_reports_unavailable_semantic_modes() {
        let summary = VectorSummary::from_parts(true, false, false);

        assert!(summary.supports_mode(RetrievalMode::LexicalOnly));
        assert!(!summary.supports_mode(RetrievalMode::SemanticOnly));
        assert_eq!(
            summary.unavailable_reason(RetrievalMode::Hybrid),
            Some(
                "Hybrid search unavailable: this CLI session does not have an embedding provider attached.".to_owned()
            )
        );
    }

    #[test]
    fn vector_summary_uses_compact_snapshot_when_vectors_disabled() {
        let summary = VectorSummary::from_parts(false, false, false);

        assert!(!summary.show_detailed_snapshot());
        assert_eq!(summary.compact_snapshot_line(), "Vector snapshot: disabled");
    }

    #[test]
    fn search_match_details_hide_absent_provenance() {
        let details = super::SearchMatchDetails {
            lexical_match: false,
            semantic_match: false,
            semantic_score: None,
        };

        assert_eq!(details.label(), None);
    }
}
