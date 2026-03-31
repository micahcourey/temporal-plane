use std::path::PathBuf;

use clap::{ArgGroup, Parser, Subcommand, ValueEnum};
use mnemix_core::{
    CheckpointName, DisclosureDepth, EntityName, MemoryId, PolicyAction, PolicyTrigger, ScopeId,
    SessionId, SourceRef, TagName, ToolName,
};

use crate::output::OutputFormat;

const DEFAULT_STORE_PATH: &str = ".mnemix";
const DEFAULT_HISTORY_LIMIT: u16 = 20;
const DEFAULT_LIST_LIMIT: u16 = 20;
const DEFAULT_SEARCH_LIMIT: u16 = 10;

#[derive(Parser, Debug)]
#[command(author, version, about = "Human-first CLI for Mnemix")]
pub(crate) struct Cli {
    #[arg(long, global = true, default_value = DEFAULT_STORE_PATH, value_name = "PATH")]
    pub(crate) store: PathBuf,

    #[arg(long, global = true)]
    pub(crate) json: bool,

    #[command(subcommand)]
    pub(crate) command: Command,
}

impl Cli {
    pub(crate) const fn output_format(&self) -> OutputFormat {
        if self.json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    Init,
    Remember(Box<RememberArgs>),
    Recall(RecallArgs),
    Search(SearchArgs),
    Show(ShowArgs),
    Pins(PinsArgs),
    History(HistoryArgs),
    Checkpoint(CheckpointArgs),
    Versions(VersionsArgs),
    Restore(RestoreArgs),
    Optimize(OptimizeArgs),
    Stats(StatsArgs),
    Export(ExportArgs),
    Import(ImportArgs),
    Policy(PolicyArgs),
}

#[derive(clap::Args, Debug)]
pub(crate) struct PolicyArgs {
    #[command(subcommand)]
    pub(crate) command: PolicyCommand,
}

#[derive(Subcommand, Debug)]
pub(crate) enum PolicyCommand {
    Check(PolicyCheckArgs),
    Explain(PolicyCheckArgs),
    Record(PolicyRecordArgs),
}

#[derive(clap::Args, Debug)]
pub(crate) struct PolicyCheckArgs {
    #[arg(long, value_parser = parse_policy_trigger)]
    pub(crate) trigger: PolicyTrigger,

    #[arg(long)]
    pub(crate) workflow_key: Option<String>,

    #[arg(long)]
    pub(crate) host: Option<String>,

    #[arg(long)]
    pub(crate) task_kind: Option<String>,

    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: Option<ScopeId>,

    #[arg(long = "path")]
    pub(crate) path: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub(crate) struct PolicyRecordArgs {
    #[arg(long)]
    pub(crate) workflow_key: String,

    #[arg(long, value_parser = parse_policy_action)]
    pub(crate) action: PolicyAction,

    #[arg(long)]
    pub(crate) reason: Option<String>,
}

#[derive(clap::Args, Debug)]
pub(crate) struct RecallArgs {
    #[arg(long)]
    pub(crate) text: Option<String>,

    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: Option<ScopeId>,

    #[arg(long, value_enum, default_value_t = DisclosureDepthArg::SummaryThenPinned)]
    pub(crate) disclosure_depth: DisclosureDepthArg,

    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=1000), default_value_t = DEFAULT_SEARCH_LIMIT)]
    pub(crate) limit: u16,
}

#[derive(clap::Args, Debug)]
pub(crate) struct RememberArgs {
    #[arg(long, value_parser = parse_memory_id)]
    pub(crate) id: MemoryId,

    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: ScopeId,

    #[arg(long, value_enum)]
    pub(crate) kind: MemoryKindArg,

    #[arg(long)]
    pub(crate) title: String,

    #[arg(long)]
    pub(crate) summary: String,

    #[arg(long)]
    pub(crate) detail: String,

    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), default_value_t = 50)]
    pub(crate) importance: u8,

    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100), default_value_t = 100)]
    pub(crate) confidence: u8,

    #[arg(long, value_parser = parse_tag_name)]
    pub(crate) tag: Vec<TagName>,

    #[arg(long, value_parser = parse_entity_name)]
    pub(crate) entity: Vec<EntityName>,

    #[arg(long)]
    pub(crate) pin_reason: Option<String>,

    #[arg(long, value_parser = parse_metadata_entry)]
    pub(crate) metadata: Vec<MetadataEntry>,

    #[arg(long, value_parser = parse_session_id)]
    pub(crate) source_session_id: Option<SessionId>,

    #[arg(long, value_parser = parse_tool_name)]
    pub(crate) source_tool: Option<ToolName>,

    #[arg(long, value_parser = parse_source_ref)]
    pub(crate) source_ref: Option<SourceRef>,
}

#[derive(clap::Args, Debug)]
pub(crate) struct SearchArgs {
    #[arg(long)]
    pub(crate) text: String,

    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: Option<ScopeId>,

    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=1000), default_value_t = DEFAULT_SEARCH_LIMIT)]
    pub(crate) limit: u16,
}

#[derive(clap::Args, Debug)]
pub(crate) struct ShowArgs {
    #[arg(long, value_parser = parse_memory_id)]
    pub(crate) id: MemoryId,
}

#[derive(clap::Args, Debug)]
pub(crate) struct PinsArgs {
    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: Option<ScopeId>,

    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=1000), default_value_t = DEFAULT_LIST_LIMIT)]
    pub(crate) limit: u16,
}

#[derive(clap::Args, Debug)]
pub(crate) struct HistoryArgs {
    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: Option<ScopeId>,

    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=1000), default_value_t = DEFAULT_HISTORY_LIMIT)]
    pub(crate) limit: u16,
}

#[derive(clap::Args, Debug)]
pub(crate) struct CheckpointArgs {
    #[arg(long, value_parser = parse_checkpoint_name)]
    pub(crate) name: CheckpointName,

    #[arg(long)]
    pub(crate) description: Option<String>,
}

#[derive(clap::Args, Debug)]
pub(crate) struct VersionsArgs {
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..=1000), default_value_t = DEFAULT_HISTORY_LIMIT)]
    pub(crate) limit: u16,
}

#[derive(clap::Args, Debug)]
#[command(group(
    ArgGroup::new("restore_target")
        .required(true)
        .multiple(false)
        .args(["checkpoint", "version"])
))]
pub(crate) struct RestoreArgs {
    #[arg(long, value_parser = parse_checkpoint_name, group = "restore_target")]
    pub(crate) checkpoint: Option<CheckpointName>,

    #[arg(long, group = "restore_target")]
    pub(crate) version: Option<u64>,
}

#[derive(clap::Args, Debug)]
pub(crate) struct OptimizeArgs {
    #[arg(long)]
    pub(crate) prune: bool,

    #[arg(long, value_parser = clap::value_parser!(u16).range(0..=3650), default_value_t = 30)]
    pub(crate) older_than_days: u16,
}

#[derive(clap::Args, Debug)]
pub(crate) struct StatsArgs {
    #[arg(long, value_parser = parse_scope_id)]
    pub(crate) scope: Option<ScopeId>,
}

#[derive(clap::Args, Debug)]
pub(crate) struct ExportArgs {
    #[arg(long, value_name = "PATH")]
    pub(crate) destination: PathBuf,
}

#[derive(clap::Args, Debug)]
pub(crate) struct ImportArgs {
    #[arg(long, value_name = "PATH")]
    pub(crate) source: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MetadataEntry {
    pub(crate) key: String,
    pub(crate) value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum MemoryKindArg {
    Observation,
    Decision,
    Preference,
    Summary,
    Fact,
    Procedure,
    Warning,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum DisclosureDepthArg {
    SummaryOnly,
    SummaryThenPinned,
    Full,
}

impl From<MemoryKindArg> for mnemix_core::MemoryKind {
    fn from(value: MemoryKindArg) -> Self {
        match value {
            MemoryKindArg::Observation => Self::Observation,
            MemoryKindArg::Decision => Self::Decision,
            MemoryKindArg::Preference => Self::Preference,
            MemoryKindArg::Summary => Self::Summary,
            MemoryKindArg::Fact => Self::Fact,
            MemoryKindArg::Procedure => Self::Procedure,
            MemoryKindArg::Warning => Self::Warning,
        }
    }
}

impl From<DisclosureDepthArg> for DisclosureDepth {
    fn from(value: DisclosureDepthArg) -> Self {
        match value {
            DisclosureDepthArg::SummaryOnly => Self::SummaryOnly,
            DisclosureDepthArg::SummaryThenPinned => Self::SummaryThenPinned,
            DisclosureDepthArg::Full => Self::Full,
        }
    }
}

fn parse_memory_id(value: &str) -> Result<MemoryId, String> {
    MemoryId::try_from(value).map_err(|error| error.to_string())
}

fn parse_scope_id(value: &str) -> Result<ScopeId, String> {
    ScopeId::try_from(value).map_err(|error| error.to_string())
}

fn parse_tag_name(value: &str) -> Result<TagName, String> {
    TagName::try_from(value).map_err(|error| error.to_string())
}

fn parse_entity_name(value: &str) -> Result<EntityName, String> {
    EntityName::try_from(value).map_err(|error| error.to_string())
}

fn parse_checkpoint_name(value: &str) -> Result<CheckpointName, String> {
    CheckpointName::try_from(value).map_err(|error| error.to_string())
}

fn parse_session_id(value: &str) -> Result<SessionId, String> {
    SessionId::try_from(value).map_err(|error| error.to_string())
}

fn parse_tool_name(value: &str) -> Result<ToolName, String> {
    ToolName::try_from(value).map_err(|error| error.to_string())
}

fn parse_source_ref(value: &str) -> Result<SourceRef, String> {
    SourceRef::try_from(value).map_err(|error| error.to_string())
}

fn parse_metadata_entry(value: &str) -> Result<MetadataEntry, String> {
    let (key, entry_value) = value
        .split_once('=')
        .ok_or_else(|| "metadata must use key=value format".to_owned())?;
    let trimmed_key = key.trim();
    let trimmed_value = entry_value.trim();
    if trimmed_key.is_empty() {
        return Err("metadata key cannot be empty".to_owned());
    }
    if trimmed_value.is_empty() {
        return Err("metadata value cannot be empty".to_owned());
    }
    Ok(MetadataEntry {
        key: trimmed_key.to_owned(),
        value: trimmed_value.to_owned(),
    })
}

fn parse_policy_trigger(value: &str) -> Result<PolicyTrigger, String> {
    value.parse()
}

fn parse_policy_action(value: &str) -> Result<PolicyAction, String> {
    value.parse()
}

#[cfg(test)]
mod tests {
    use super::{MetadataEntry, parse_metadata_entry};

    #[test]
    fn parse_metadata_entry_trims_surrounding_whitespace() {
        assert_eq!(
            parse_metadata_entry(" owner = cli "),
            Ok(MetadataEntry {
                key: "owner".to_owned(),
                value: "cli".to_owned(),
            })
        );
    }
}
