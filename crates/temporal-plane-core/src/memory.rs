//! Durable memory records and related value objects.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{
    CoreError, EntityName, MemoryId, RecordedAt, ScopeId, SessionId, SourceRef, TagName, ToolName,
};

const TEXT_MAX_LEN: usize = 4_096;

fn validate_text(field: &'static str, value: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyValue { field });
    }
    let actual = trimmed.chars().count();
    if actual > TEXT_MAX_LEN {
        return Err(CoreError::TooLong {
            field,
            max: TEXT_MAX_LEN,
            actual,
        });
    }
    Ok(trimmed.to_owned())
}

/// Classifies the durable role a memory plays in the product model.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MemoryKind {
    /// A captured observation about the world or codebase.
    Observation,
    /// A durable decision that should be revisited later.
    Decision,
    /// A stable preference or behavioral instruction.
    Preference,
    /// A distilled summary standing in for deeper detail.
    Summary,
    /// A factual statement worth retrieving directly.
    Fact,
    /// A procedural or workflow-oriented memory.
    Procedure,
    /// A warning, risk, or caveat that should be surfaced carefully.
    Warning,
}

fn validate_percentage(field: &'static str, value: u8) -> Result<u8, CoreError> {
    if value > 100 {
        return Err(CoreError::OutOfRange {
            field,
            min: 0,
            max: 100,
            actual: u64::from(value),
        });
    }

    Ok(value)
}

/// A percentage-like importance score in the range `0..=100`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Importance(u8);

impl Importance {
    /// Creates a validated importance score.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the score is outside the supported
    /// percentage range of `0..=100`.
    pub fn new(value: u8) -> Result<Self, CoreError> {
        Self::try_from(value)
    }

    /// Returns the raw score value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl Default for Importance {
    fn default() -> Self {
        Self(50)
    }
}

impl TryFrom<u8> for Importance {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(validate_percentage("importance", value)?))
    }
}

impl From<Importance> for u8 {
    fn from(value: Importance) -> Self {
        value.0
    }
}

/// A confidence score describing how trustworthy a memory is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct Confidence(u8);

impl Confidence {
    /// Creates a validated confidence score.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the score is outside the supported
    /// percentage range of `0..=100`.
    pub fn new(value: u8) -> Result<Self, CoreError> {
        Self::try_from(value)
    }

    /// Returns the raw score value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(100)
    }
}

impl TryFrom<u8> for Confidence {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(Self(validate_percentage("confidence", value)?))
    }
}

impl From<Confidence> for u8 {
    fn from(value: Confidence) -> Self {
        value.0
    }
}

/// Captures whether a memory is pinned into the higher-priority recall layer.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum PinState {
    /// The memory is not pinned.
    #[default]
    NotPinned,
    /// The memory is pinned with a user-meaningful reason.
    Pinned {
        /// The explanation for why the memory is pinned.
        reason: String,
    },
}

impl PinState {
    /// Creates a pinned state after validating the provided reason.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the reason is blank or exceeds the supported
    /// text length.
    pub fn pinned(reason: impl Into<String>) -> Result<Self, CoreError> {
        Ok(Self::Pinned {
            reason: validate_text("pin_reason", &reason.into())?,
        })
    }

    /// Returns `true` when the memory is pinned.
    #[must_use]
    pub const fn is_pinned(&self) -> bool {
        matches!(self, Self::Pinned { .. })
    }

    /// Returns the pin reason when present.
    #[must_use]
    pub fn reason(&self) -> Option<&str> {
        match self {
            Self::NotPinned => None,
            Self::Pinned { reason } => Some(reason.as_str()),
        }
    }
}

/// A durable memory record stored by Temporal Plane.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MemoryRecord {
    id: MemoryId,
    scope_id: ScopeId,
    kind: MemoryKind,
    title: String,
    summary: String,
    detail: String,
    fts_text: String,
    importance: Importance,
    confidence: Confidence,
    created_at: RecordedAt,
    updated_at: RecordedAt,
    source_session_id: Option<SessionId>,
    source_tool: Option<ToolName>,
    source_ref: Option<SourceRef>,
    tags: BTreeSet<TagName>,
    entities: BTreeSet<EntityName>,
    pin_state: PinState,
    metadata: BTreeMap<String, String>,
}

impl MemoryRecord {
    /// Starts building a memory record.
    #[must_use]
    pub fn builder(id: MemoryId, scope_id: ScopeId, kind: MemoryKind) -> MemoryRecordBuilder {
        MemoryRecordBuilder {
            id,
            scope_id,
            kind,
            title: None,
            summary: None,
            detail: None,
            fts_text: None,
            importance: Importance::default(),
            confidence: Confidence::default(),
            created_at: None,
            updated_at: None,
            source_session_id: None,
            source_tool: None,
            source_ref: None,
            tags: BTreeSet::new(),
            entities: BTreeSet::new(),
            pin_state: PinState::default(),
            metadata: BTreeMap::new(),
        }
    }

    /// Returns the stable identifier.
    #[must_use]
    pub const fn id(&self) -> &MemoryId {
        &self.id
    }

    /// Returns the associated scope identifier.
    #[must_use]
    pub const fn scope_id(&self) -> &ScopeId {
        &self.scope_id
    }

    /// Returns the memory kind.
    #[must_use]
    pub const fn kind(&self) -> MemoryKind {
        self.kind
    }

    /// Returns the short title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the compact summary.
    #[must_use]
    pub fn summary(&self) -> &str {
        &self.summary
    }

    /// Returns the longer-form detail.
    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// Returns the composed full-text retrieval field.
    #[must_use]
    pub fn fts_text(&self) -> &str {
        &self.fts_text
    }

    /// Returns the importance score.
    #[must_use]
    pub const fn importance(&self) -> Importance {
        self.importance
    }

    /// Returns the confidence score.
    #[must_use]
    pub const fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created_at(&self) -> RecordedAt {
        self.created_at
    }

    /// Returns the most recent update timestamp.
    #[must_use]
    pub const fn updated_at(&self) -> RecordedAt {
        self.updated_at
    }

    /// Returns the optional source session identifier.
    #[must_use]
    pub const fn source_session_id(&self) -> Option<&SessionId> {
        self.source_session_id.as_ref()
    }

    /// Returns the optional source tool.
    #[must_use]
    pub const fn source_tool(&self) -> Option<&ToolName> {
        self.source_tool.as_ref()
    }

    /// Returns the optional source reference.
    #[must_use]
    pub const fn source_ref(&self) -> Option<&SourceRef> {
        self.source_ref.as_ref()
    }

    /// Returns the retrieval tags.
    #[must_use]
    pub const fn tags(&self) -> &BTreeSet<TagName> {
        &self.tags
    }

    /// Returns the extracted entity labels.
    #[must_use]
    pub const fn entities(&self) -> &BTreeSet<EntityName> {
        &self.entities
    }

    /// Returns the pin state.
    #[must_use]
    pub const fn pin_state(&self) -> &PinState {
        &self.pin_state
    }

    /// Returns the string metadata map.
    #[must_use]
    pub const fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }
}

/// Builder for [`MemoryRecord`].
#[derive(Clone, Debug)]
pub struct MemoryRecordBuilder {
    id: MemoryId,
    scope_id: ScopeId,
    kind: MemoryKind,
    title: Option<String>,
    summary: Option<String>,
    detail: Option<String>,
    fts_text: Option<String>,
    importance: Importance,
    confidence: Confidence,
    created_at: Option<RecordedAt>,
    updated_at: Option<RecordedAt>,
    source_session_id: Option<SessionId>,
    source_tool: Option<ToolName>,
    source_ref: Option<SourceRef>,
    tags: BTreeSet<TagName>,
    entities: BTreeSet<EntityName>,
    pin_state: PinState,
    metadata: BTreeMap<String, String>,
}

impl MemoryRecordBuilder {
    /// Sets the short title.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the title is blank or exceeds the supported
    /// text length.
    pub fn title(mut self, value: impl Into<String>) -> Result<Self, CoreError> {
        self.title = Some(validate_text("title", &value.into())?);
        Ok(self)
    }

    /// Sets the compact summary.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the summary is blank or exceeds the
    /// supported text length.
    pub fn summary(mut self, value: impl Into<String>) -> Result<Self, CoreError> {
        self.summary = Some(validate_text("summary", &value.into())?);
        Ok(self)
    }

    /// Sets the longer-form detail.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the detail is blank or exceeds the supported
    /// text length.
    pub fn detail(mut self, value: impl Into<String>) -> Result<Self, CoreError> {
        self.detail = Some(validate_text("detail", &value.into())?);
        Ok(self)
    }

    /// Sets the explicit full-text retrieval payload.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the payload is blank or exceeds the
    /// supported text length.
    pub fn fts_text(mut self, value: impl Into<String>) -> Result<Self, CoreError> {
        self.fts_text = Some(validate_text("fts_text", &value.into())?);
        Ok(self)
    }

    /// Sets the importance score.
    #[must_use]
    pub fn importance(mut self, value: Importance) -> Self {
        self.importance = value;
        self
    }

    /// Sets the confidence score.
    #[must_use]
    pub fn confidence(mut self, value: Confidence) -> Self {
        self.confidence = value;
        self
    }

    /// Sets the creation timestamp.
    #[must_use]
    pub fn created_at(mut self, value: RecordedAt) -> Self {
        self.created_at = Some(value);
        self
    }

    /// Sets the updated timestamp.
    #[must_use]
    pub fn updated_at(mut self, value: RecordedAt) -> Self {
        self.updated_at = Some(value);
        self
    }

    /// Sets the source session identifier.
    #[must_use]
    pub fn source_session_id(mut self, value: SessionId) -> Self {
        self.source_session_id = Some(value);
        self
    }

    /// Sets the source tool name.
    #[must_use]
    pub fn source_tool(mut self, value: ToolName) -> Self {
        self.source_tool = Some(value);
        self
    }

    /// Sets the source reference.
    #[must_use]
    pub fn source_ref(mut self, value: SourceRef) -> Self {
        self.source_ref = Some(value);
        self
    }

    /// Adds a retrieval tag.
    #[must_use]
    pub fn add_tag(mut self, value: TagName) -> Self {
        self.tags.insert(value);
        self
    }

    /// Adds an entity label.
    #[must_use]
    pub fn add_entity(mut self, value: EntityName) -> Self {
        self.entities.insert(value);
        self
    }

    /// Sets the pin state.
    #[must_use]
    pub fn pin_state(mut self, value: PinState) -> Self {
        self.pin_state = value;
        self
    }

    /// Replaces the metadata map.
    #[must_use]
    pub fn metadata(mut self, value: BTreeMap<String, String>) -> Self {
        self.metadata = value;
        self
    }

    /// Builds the memory record after validating required fields.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when required fields are missing.
    pub fn build(self) -> Result<MemoryRecord, CoreError> {
        let title = self
            .title
            .ok_or(CoreError::MissingField { field: "title" })?;
        let summary = self
            .summary
            .ok_or(CoreError::MissingField { field: "summary" })?;
        let detail = self
            .detail
            .ok_or(CoreError::MissingField { field: "detail" })?;

        let created_at = self.created_at.unwrap_or_else(RecordedAt::now);
        let updated_at = self.updated_at.unwrap_or(created_at);
        let fts_text = if let Some(text) = self.fts_text {
            text
        } else {
            let tags = self.tags.iter().map(TagName::as_str);
            let entities = self.entities.iter().map(EntityName::as_str);
            [title.as_str(), summary.as_str(), detail.as_str()]
                .into_iter()
                .chain(tags)
                .chain(entities)
                .collect::<Vec<_>>()
                .join(" ")
        };

        Ok(MemoryRecord {
            id: self.id,
            scope_id: self.scope_id,
            kind: self.kind,
            title,
            summary,
            detail,
            fts_text,
            importance: self.importance,
            confidence: self.confidence,
            created_at,
            updated_at,
            source_session_id: self.source_session_id,
            source_tool: self.source_tool,
            source_ref: self.source_ref,
            tags: self.tags,
            entities: self.entities,
            pin_state: self.pin_state,
            metadata: self.metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EntityName, TagName};

    fn base_builder() -> MemoryRecordBuilder {
        MemoryRecord::builder(
            MemoryId::try_from("memory:1").expect("valid memory id"),
            ScopeId::try_from("repo:temporal-plane").expect("valid scope"),
            MemoryKind::Observation,
        )
    }

    #[test]
    fn builder_requires_title_summary_and_detail() {
        let result = base_builder().build();

        assert_eq!(result, Err(CoreError::MissingField { field: "title" }));
    }

    #[test]
    fn build_composes_fts_text_when_not_provided() {
        let record = base_builder()
            .title("Core domain")
            .expect("title should validate")
            .summary("Storage agnostic")
            .expect("summary should validate")
            .detail("The model lives in temporal-plane-core")
            .expect("detail should validate")
            .add_tag(TagName::try_from("milestone-1").expect("valid tag"))
            .add_entity(EntityName::try_from("TemporalPlane").expect("valid entity"))
            .build()
            .expect("record should build");

        assert!(record.fts_text().contains("Core domain"));
        assert!(record.fts_text().contains("milestone-1"));
        assert!(record.fts_text().contains("TemporalPlane"));
    }

    #[test]
    fn pin_state_requires_reason() {
        let result = PinState::pinned("   ");

        assert_eq!(
            result,
            Err(CoreError::EmptyValue {
                field: "pin_reason"
            })
        );
    }

    #[test]
    fn metadata_and_sources_are_preserved() {
        let record = base_builder()
            .title("Source-aware")
            .expect("title should validate")
            .summary("Track provenance")
            .expect("summary should validate")
            .detail("Source metadata remains available")
            .expect("detail should validate")
            .source_session_id(SessionId::try_from("session:demo").expect("session id"))
            .source_tool(ToolName::try_from("copilot").expect("tool name"))
            .source_ref(SourceRef::try_from("docs/temporal-plane-plan-v3.md").expect("source ref"))
            .metadata(BTreeMap::from([(
                "importance".to_string(),
                "high".to_string(),
            )]))
            .build()
            .expect("record should build");

        assert_eq!(record.source_tool().map(ToolName::as_str), Some("copilot"));
        assert_eq!(
            record.metadata().get("importance"),
            Some(&"high".to_string())
        );
    }

    #[test]
    fn importance_rejects_out_of_range_values() {
        let result = Importance::new(101);

        assert_eq!(
            result,
            Err(CoreError::OutOfRange {
                field: "importance",
                min: 0,
                max: 100,
                actual: 101,
            })
        );
    }

    #[test]
    fn confidence_rejects_out_of_range_values() {
        let result = Confidence::new(255);

        assert_eq!(
            result,
            Err(CoreError::OutOfRange {
                field: "confidence",
                min: 0,
                max: 100,
                actual: 255,
            })
        );
    }
}
