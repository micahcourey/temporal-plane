use std::{collections::BTreeSet, time::SystemTime};

use humantime::format_rfc3339;
use mnemix_core::{
    MemoryRecord, QueryLimit, ScopeId, SearchQuery,
    traits::{BrowseBackend, RecallBackend},
};

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
}

impl MemoryEntry {
    pub(crate) fn from_record(record: MemoryRecord) -> Self {
        Self {
            created_date: record_date(record.created_at().value()),
            updated_date: record_date(record.updated_at().value()),
            record,
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

pub(crate) fn search_entries<B>(
    backend: &B,
    query_text: &str,
    scope: Option<ScopeId>,
    limit: QueryLimit,
) -> Result<Vec<MemoryEntry>, CliError>
where
    B: RecallBackend,
    CliError: From<B::Error>,
{
    let query = SearchQuery::new(query_text.to_owned(), scope, limit)?;
    let records = backend.search(&query)?;
    Ok(records.into_iter().map(MemoryEntry::from_record).collect())
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

#[cfg(test)]
mod tests {
    use mnemix_core::{MemoryId, MemoryRecord, RecordedAt, ScopeId, memory::MemoryKind};

    use super::{MemoryEntry, matches_date_range, validate_date_filter};

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
}
