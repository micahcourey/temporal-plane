use std::{collections::BTreeMap, time::SystemTime};

use humantime::format_rfc3339;
use serde::Serialize;
use temporal_plane_core::{
    Checkpoint, MemoryRecord, PinState, StatsSnapshot, VersionRecord, memory::MemoryKind,
};

use crate::errors::CliError;

mod human;
mod json;

pub(crate) use human::render_human;
pub(crate) use json::render_json;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OutputFormat {
    Human,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub(crate) enum CommandOutput {
    Status(Box<StatusView>),
    Memory(Box<MemoryResultView>),
    MemoryList(Box<MemoryListView>),
    Checkpoint(Box<CheckpointResultView>),
    VersionList(Box<VersionListView>),
    Stats(Box<StatsResultView>),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct StatusView {
    pub(crate) command: &'static str,
    pub(crate) status: &'static str,
    pub(crate) message: String,
    pub(crate) path: Option<String>,
    pub(crate) schema_version: Option<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct MemoryResultView {
    pub(crate) command: &'static str,
    pub(crate) action: &'static str,
    pub(crate) memory: MemoryDetailView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct MemoryListView {
    pub(crate) command: &'static str,
    pub(crate) scope: Option<String>,
    pub(crate) query_text: Option<String>,
    pub(crate) count: usize,
    pub(crate) memories: Vec<MemorySummaryView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CheckpointResultView {
    pub(crate) command: &'static str,
    pub(crate) action: &'static str,
    pub(crate) checkpoint: CheckpointView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VersionListView {
    pub(crate) command: &'static str,
    pub(crate) count: usize,
    pub(crate) scope: Option<String>,
    pub(crate) versions: Vec<VersionView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct StatsResultView {
    pub(crate) command: &'static str,
    pub(crate) stats: StatsView,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct MemorySummaryView {
    pub(crate) id: String,
    pub(crate) scope_id: String,
    pub(crate) kind: &'static str,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) pinned: bool,
    pub(crate) pin_reason: Option<String>,
    pub(crate) importance: u8,
    pub(crate) confidence: u8,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) tags: Vec<String>,
    pub(crate) entities: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct MemoryDetailView {
    pub(crate) id: String,
    pub(crate) scope_id: String,
    pub(crate) kind: &'static str,
    pub(crate) title: String,
    pub(crate) summary: String,
    pub(crate) detail: String,
    pub(crate) pinned: bool,
    pub(crate) pin_reason: Option<String>,
    pub(crate) importance: u8,
    pub(crate) confidence: u8,
    pub(crate) created_at: String,
    pub(crate) updated_at: String,
    pub(crate) source_session_id: Option<String>,
    pub(crate) source_tool: Option<String>,
    pub(crate) source_ref: Option<String>,
    pub(crate) tags: Vec<String>,
    pub(crate) entities: Vec<String>,
    pub(crate) metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CheckpointView {
    pub(crate) name: String,
    pub(crate) version: u64,
    pub(crate) created_at: String,
    pub(crate) description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct VersionView {
    pub(crate) version: u64,
    pub(crate) recorded_at: String,
    pub(crate) checkpoint_name: Option<String>,
    pub(crate) checkpoint_version: Option<u64>,
    pub(crate) summary: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct StatsView {
    pub(crate) scope: Option<String>,
    pub(crate) total_memories: u64,
    pub(crate) pinned_memories: u64,
    pub(crate) version_count: u64,
    pub(crate) latest_checkpoint: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorEnvelope<'a> {
    kind: &'static str,
    message: String,
    code: &'a str,
}

const ERROR_KIND: &str = "error";

pub(crate) fn render_output(
    output: &CommandOutput,
    format: OutputFormat,
) -> Result<String, CliError> {
    match format {
        OutputFormat::Human => Ok(render_human(output)),
        OutputFormat::Json => render_json(output).map_err(Into::into),
    }
}

pub(crate) fn render_error(error: &CliError, format: OutputFormat) -> String {
    match format {
        OutputFormat::Human => format!("error: {error}\n"),
        OutputFormat::Json => {
            let envelope = ErrorEnvelope {
                kind: ERROR_KIND,
                message: error.to_string(),
                code: error.code(),
            };
            match serde_json::to_string_pretty(&envelope) {
                Ok(json) => format!("{json}\n"),
                Err(_) => format!(
                    "{{\"kind\":\"error\",\"message\":{:?},\"code\":{:?}}}\n",
                    error.to_string(),
                    error.code()
                ),
            }
        }
    }
}

pub(crate) fn memory_detail_view(record: &MemoryRecord) -> MemoryDetailView {
    MemoryDetailView {
        id: record.id().as_str().to_owned(),
        scope_id: record.scope_id().as_str().to_owned(),
        kind: memory_kind_name(record.kind()),
        title: record.title().to_owned(),
        summary: record.summary().to_owned(),
        detail: record.detail().to_owned(),
        pinned: record.pin_state().is_pinned(),
        pin_reason: pin_reason(record.pin_state()),
        importance: record.importance().value(),
        confidence: record.confidence().value(),
        created_at: format_timestamp(record.created_at().value()),
        updated_at: format_timestamp(record.updated_at().value()),
        source_session_id: record
            .source_session_id()
            .map(|value| value.as_str().to_owned()),
        source_tool: record.source_tool().map(|value| value.as_str().to_owned()),
        source_ref: record.source_ref().map(|value| value.as_str().to_owned()),
        tags: record
            .tags()
            .iter()
            .map(|value| value.as_str().to_owned())
            .collect(),
        entities: record
            .entities()
            .iter()
            .map(|value| value.as_str().to_owned())
            .collect(),
        metadata: record.metadata().clone(),
    }
}

pub(crate) fn memory_summary_view(record: &MemoryRecord) -> MemorySummaryView {
    MemorySummaryView {
        id: record.id().as_str().to_owned(),
        scope_id: record.scope_id().as_str().to_owned(),
        kind: memory_kind_name(record.kind()),
        title: record.title().to_owned(),
        summary: record.summary().to_owned(),
        pinned: record.pin_state().is_pinned(),
        pin_reason: pin_reason(record.pin_state()),
        importance: record.importance().value(),
        confidence: record.confidence().value(),
        created_at: format_timestamp(record.created_at().value()),
        updated_at: format_timestamp(record.updated_at().value()),
        tags: record
            .tags()
            .iter()
            .map(|value| value.as_str().to_owned())
            .collect(),
        entities: record
            .entities()
            .iter()
            .map(|value| value.as_str().to_owned())
            .collect(),
    }
}

pub(crate) fn checkpoint_view(checkpoint: &Checkpoint) -> CheckpointView {
    CheckpointView {
        name: checkpoint.name().as_str().to_owned(),
        version: checkpoint.version().value(),
        created_at: format_timestamp(checkpoint.created_at().value()),
        description: checkpoint.description().map(ToOwned::to_owned),
    }
}

pub(crate) fn version_view(record: &VersionRecord) -> VersionView {
    VersionView {
        version: record.version().value(),
        recorded_at: format_timestamp(record.recorded_at().value()),
        checkpoint_name: record
            .checkpoint()
            .map(|value| value.name().as_str().to_owned()),
        checkpoint_version: record.checkpoint().map(|value| value.version().value()),
        summary: record.summary().map(ToOwned::to_owned),
    }
}

pub(crate) fn stats_view(stats: &StatsSnapshot) -> StatsView {
    StatsView {
        scope: stats.scope().map(|value| value.as_str().to_owned()),
        total_memories: stats.total_memories(),
        pinned_memories: stats.pinned_memories(),
        version_count: stats.version_count(),
        latest_checkpoint: stats
            .latest_checkpoint()
            .map(|value| value.as_str().to_owned()),
    }
}

pub(crate) fn format_timestamp(value: SystemTime) -> String {
    format_rfc3339(value).to_string()
}

fn pin_reason(pin_state: &PinState) -> Option<String> {
    pin_state.reason().map(ToOwned::to_owned)
}

fn memory_kind_name(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Observation => "observation",
        MemoryKind::Decision => "decision",
        MemoryKind::Preference => "preference",
        MemoryKind::Summary => "summary",
        MemoryKind::Fact => "fact",
        MemoryKind::Procedure => "procedure",
        MemoryKind::Warning => "warning",
    }
}
