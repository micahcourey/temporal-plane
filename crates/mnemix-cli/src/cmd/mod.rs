use std::{collections::BTreeMap, path::Path};

use mnemix_core::{
    CheckpointSelector, Confidence, Importance, MemoryRecord, OptimizeRequest, OptimizeResult,
    PinState, PolicyDecision, QueryLimit, RestoreResult, RetrievalMode,
};
use mnemix_lancedb::{LanceDbBackend, LanceDbError, LanceDbOpenOptions};

use crate::{
    cli::{Command, RememberArgs, UiArgs},
    errors::CliError,
    output::{
        CheckpointResultView, CommandOutput, MemoryListView, MemoryResultView, MemorySummaryView,
        OptimizeResultView, OptimizeRetentionView, ProviderProfileListView,
        ProviderProfileResultView, ProviderProfileView, RecallEntryView, RecallResultView,
        RestoreResultView, RestoreTargetView, StatsResultView, StatusView, VersionListView,
        checkpoint_view, disclosure_depth_name, memory_detail_view, memory_summary_view,
        recall_entry_view, retrieval_mode_name, stats_view, version_view,
    },
    providers_runtime::{ResolvedProvider, resolve_named_provider},
};

mod checkpoint;
mod export;
mod history;
mod import;
mod init;
mod optimize;
mod pins;
mod policy;
mod providers;
mod recall;
mod remember;
mod restore;
mod search;
mod show;
mod stats;
mod ui;
mod vectors;
mod versions;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ProviderStoreCompatibility {
    StoreNotInitialized,
    VectorsDisabled,
    MissingModel,
    MissingDimensions,
    Compatible {
        store_model: String,
        store_dimensions: u32,
    },
    ModelMismatch {
        store_model: String,
        provider_model: String,
    },
    DimensionMismatch {
        model: String,
        store_dimensions: u32,
        provider_dimensions: u32,
    },
}

pub(crate) fn execute(command: &Command, store_path: &Path) -> Result<CommandOutput, CliError> {
    match command {
        Command::Init => init::run(store_path),
        Command::Ui(_) => unreachable!("interactive UI is handled separately"),
        Command::Remember(args) => remember::run(store_path, args),
        Command::Recall(args) => recall::run(store_path, args),
        Command::Search(args) => search::run(store_path, args),
        Command::Show(args) => show::run(store_path, args),
        Command::Pins(args) => pins::run(store_path, args),
        Command::History(args) => history::run(store_path, args),
        Command::Checkpoint(args) => checkpoint::run(store_path, args),
        Command::Versions(args) => versions::run(store_path, args),
        Command::Restore(args) => restore::run(store_path, args),
        Command::Optimize(args) => optimize::run(store_path, args),
        Command::Providers(args) => providers::run(store_path, args),
        Command::Vectors(args) => vectors::run(store_path, args),
        Command::Stats(args) => stats::run(store_path, args),
        Command::Export(args) => export::run(store_path, args),
        Command::Import(args) => import::run(store_path, args),
        Command::Policy(args) => policy::run(store_path, args),
    }
}

pub(crate) fn run_ui(store_path: &Path, args: &UiArgs) -> Result<(), CliError> {
    ui::run(store_path, args)
}

pub(super) fn recall_result(
    scope: Option<String>,
    query_text: Option<String>,
    retrieval_mode: RetrievalMode,
    provider: Option<String>,
    result: &mnemix_core::RecallResult,
) -> CommandOutput {
    CommandOutput::Recall(Box::new(RecallResultView {
        command: "recall",
        scope,
        query_text,
        disclosure_depth: disclosure_depth_name(result.disclosure_depth()),
        retrieval_mode: retrieval_mode_name(retrieval_mode),
        provider,
        count: result.count(),
        pinned_context: result
            .pinned_context()
            .iter()
            .map(recall_entry_view)
            .collect::<Vec<RecallEntryView>>(),
        summaries: result
            .summaries()
            .iter()
            .map(recall_entry_view)
            .collect::<Vec<RecallEntryView>>(),
        archival: result
            .archival()
            .iter()
            .map(recall_entry_view)
            .collect::<Vec<RecallEntryView>>(),
    }))
}

pub(super) fn open_store(store_path: &Path) -> Result<LanceDbBackend, CliError> {
    LanceDbBackend::open(store_path).map_err(|error| match error {
        LanceDbError::InvalidStorePath { path, .. } => CliError::StoreNotInitialized { path },
        other => CliError::Backend(other),
    })
}

pub(super) fn open_store_with_provider(
    store_path: &Path,
    provider_name: Option<&str>,
) -> Result<LanceDbBackend, CliError> {
    if let Some(provider_name) = provider_name {
        let provider = resolve_named_provider(provider_name)?;
        open_store_with_resolved_provider(store_path, provider)
    } else {
        open_store(store_path)
    }
}

pub(super) fn open_store_with_resolved_provider(
    store_path: &Path,
    provider: ResolvedProvider,
) -> Result<LanceDbBackend, CliError> {
    LanceDbBackend::open_with_options(
        store_path,
        LanceDbOpenOptions::new().embedding_provider(provider.provider),
    )
    .map_err(|error| match error {
        LanceDbError::InvalidStorePath { path, .. } => CliError::StoreNotInitialized { path },
        other => CliError::Backend(other),
    })
}

pub(super) fn open_or_init_store(store_path: &Path) -> Result<LanceDbBackend, CliError> {
    LanceDbBackend::connect_or_init(store_path).map_err(Into::into)
}

pub(super) fn query_limit(value: u16) -> Result<QueryLimit, CliError> {
    QueryLimit::new(value).map_err(Into::into)
}

pub(super) fn provider_store_compatibility(
    store_path: &Path,
    provider: &ResolvedProvider,
) -> Result<ProviderStoreCompatibility, CliError> {
    let backend = match open_store(store_path) {
        Ok(backend) => backend,
        Err(CliError::StoreNotInitialized { .. }) => {
            return Ok(ProviderStoreCompatibility::StoreNotInitialized);
        }
        Err(error) => return Err(error),
    };
    let status = backend.vector_status()?;
    let settings = status.settings();

    if !settings.vectors_enabled() {
        return Ok(ProviderStoreCompatibility::VectorsDisabled);
    }

    let Some(store_model) = settings.embedding_model() else {
        return Ok(ProviderStoreCompatibility::MissingModel);
    };
    let Some(store_dimensions) = settings.embedding_dimensions() else {
        return Ok(ProviderStoreCompatibility::MissingDimensions);
    };

    if store_model != provider.model_id {
        return Ok(ProviderStoreCompatibility::ModelMismatch {
            store_model: store_model.to_owned(),
            provider_model: provider.model_id.clone(),
        });
    }

    if store_dimensions != provider.dimensions {
        return Ok(ProviderStoreCompatibility::DimensionMismatch {
            model: store_model.to_owned(),
            store_dimensions,
            provider_dimensions: provider.dimensions,
        });
    }

    Ok(ProviderStoreCompatibility::Compatible {
        store_model: store_model.to_owned(),
        store_dimensions,
    })
}

impl ProviderStoreCompatibility {
    pub(super) const fn label(&self) -> &'static str {
        match self {
            Self::StoreNotInitialized => "store_not_initialized",
            Self::VectorsDisabled => "vectors_disabled",
            Self::MissingModel => "missing_model",
            Self::MissingDimensions => "missing_dimensions",
            Self::Compatible { .. } => "matched",
            Self::ModelMismatch { .. } => "model_mismatch",
            Self::DimensionMismatch { .. } => "dimension_mismatch",
        }
    }

    pub(super) const fn is_compatible(&self) -> bool {
        matches!(self, Self::Compatible { .. })
    }

    pub(super) fn detail(&self) -> String {
        match self {
            Self::StoreNotInitialized => "store has not been initialized yet".to_owned(),
            Self::VectorsDisabled => "store vectors are not enabled yet".to_owned(),
            Self::MissingModel => {
                "store vectors are enabled but embedding_model is not configured".to_owned()
            }
            Self::MissingDimensions => {
                "store vectors are enabled but embedding_dimensions is not configured".to_owned()
            }
            Self::Compatible {
                store_model,
                store_dimensions,
            } => format!(
                "store model={store_model} dimensions={store_dimensions} match the provider"
            ),
            Self::ModelMismatch {
                store_model,
                provider_model,
            } => format!(
                "store expects model `{store_model}` but provider resolved model `{provider_model}`"
            ),
            Self::DimensionMismatch {
                model,
                store_dimensions,
                provider_dimensions,
            } => format!(
                "store model `{model}` expects {store_dimensions} dimensions but provider resolved {provider_dimensions}"
            ),
        }
    }
}

pub(super) fn ensure_provider_store_compatible(
    store_path: &Path,
    provider_name: &str,
    provider: &ResolvedProvider,
    command: &'static str,
) -> Result<(), CliError> {
    match provider_store_compatibility(store_path, provider)? {
        ProviderStoreCompatibility::Compatible { .. } => Ok(()),
        ProviderStoreCompatibility::StoreNotInitialized => Err(CliError::StoreNotInitialized {
            path: store_path.to_path_buf(),
        }),
        state => Err(CliError::ProviderStoreIncompatible {
            name: provider_name.to_owned(),
            command,
            details: state.detail(),
        }),
    }
}

pub(super) fn build_memory_record(args: &RememberArgs) -> Result<MemoryRecord, CliError> {
    let metadata = args
        .metadata
        .iter()
        .map(|entry| (entry.key.clone(), entry.value.clone()))
        .collect::<BTreeMap<_, _>>();

    let builder = MemoryRecord::builder(args.id.clone(), args.scope.clone(), args.kind.into())
        .title(&args.title)?
        .summary(&args.summary)?
        .detail(&args.detail)?
        .importance(Importance::new(args.importance)?)
        .confidence(Confidence::new(args.confidence)?);

    let builder = args
        .tag
        .iter()
        .cloned()
        .fold(builder, mnemix_core::MemoryRecordBuilder::add_tag);
    let builder = args
        .entity
        .iter()
        .cloned()
        .fold(builder, mnemix_core::MemoryRecordBuilder::add_entity);
    let builder = if let Some(pin_reason) = &args.pin_reason {
        builder.pin_state(PinState::pinned(pin_reason)?)
    } else {
        builder
    };
    let builder = if let Some(source_session_id) = &args.source_session_id {
        builder.source_session_id(source_session_id.clone())
    } else {
        builder
    };
    let builder = if let Some(source_tool) = &args.source_tool {
        builder.source_tool(source_tool.clone())
    } else {
        builder
    };
    let builder = if let Some(source_ref) = &args.source_ref {
        builder.source_ref(source_ref.clone())
    } else {
        builder
    };

    builder.metadata(metadata).build().map_err(Into::into)
}

pub(super) fn memory_result(
    command: &'static str,
    action: &'static str,
    record: &MemoryRecord,
) -> CommandOutput {
    CommandOutput::Memory(Box::new(MemoryResultView {
        command,
        action,
        memory: memory_detail_view(record),
    }))
}

pub(super) fn memory_list_result(
    command: &'static str,
    scope: Option<String>,
    query_text: Option<String>,
    records: &[MemoryRecord],
) -> CommandOutput {
    CommandOutput::MemoryList(Box::new(MemoryListView {
        command,
        scope,
        query_text,
        retrieval_mode: None,
        provider: None,
        count: records.len(),
        memories: records.iter().map(memory_summary_view).collect(),
    }))
}

pub(super) fn search_result(
    scope: Option<String>,
    query_text: String,
    retrieval_mode: RetrievalMode,
    provider: Option<String>,
    memories: Vec<MemorySummaryView>,
) -> CommandOutput {
    CommandOutput::MemoryList(Box::new(MemoryListView {
        command: "search",
        scope,
        query_text: Some(query_text),
        retrieval_mode: Some(retrieval_mode_name(retrieval_mode)),
        provider,
        count: memories.len(),
        memories,
    }))
}

pub(super) fn checkpoint_result(
    action: &'static str,
    checkpoint: &mnemix_core::Checkpoint,
) -> CommandOutput {
    CommandOutput::Checkpoint(Box::new(CheckpointResultView {
        command: "checkpoint",
        action,
        checkpoint: checkpoint_view(checkpoint),
    }))
}

pub(super) fn policy_result(
    action: &'static str,
    trigger: String,
    workflow_key: Option<String>,
    decision: &PolicyDecision,
) -> CommandOutput {
    CommandOutput::Policy(Box::new(crate::output::policy_decision_view(
        action,
        trigger,
        workflow_key,
        decision,
    )))
}

pub(super) fn version_list_result(
    command: &'static str,
    scope: Option<String>,
    versions: &[mnemix_core::VersionRecord],
) -> CommandOutput {
    CommandOutput::VersionList(Box::new(VersionListView {
        command,
        count: versions.len(),
        scope,
        versions: versions.iter().map(version_view).collect(),
    }))
}

pub(super) fn restore_result(result: &RestoreResult) -> CommandOutput {
    let target = match result.target() {
        CheckpointSelector::Named(name) => RestoreTargetView {
            kind: "checkpoint",
            name: Some(name.as_str().to_owned()),
            version: result.restored_version().value(),
        },
        CheckpointSelector::Version(version) => RestoreTargetView {
            kind: "version",
            name: None,
            version: version.value(),
        },
    };

    CommandOutput::Restore(Box::new(RestoreResultView {
        command: "restore",
        target,
        previous_version: result.previous_version().value(),
        restored_version: result.restored_version().value(),
        current_version: result.current_version().value(),
        pre_restore_checkpoint: result.pre_restore_checkpoint().map(checkpoint_view),
    }))
}

pub(super) fn optimize_result(request: &OptimizeRequest, result: &OptimizeResult) -> CommandOutput {
    let retention = request.retention_policy();

    CommandOutput::Optimize(Box::new(OptimizeResultView {
        command: "optimize",
        previous_version: result.previous_version().value(),
        current_version: result.current_version().value(),
        compacted: result.compacted(),
        prune_old_versions: request.prune_old_versions(),
        pruned_versions: result.pruned_versions(),
        bytes_removed: result.bytes_removed(),
        retention: OptimizeRetentionView {
            minimum_age_days: retention.minimum_age_days(),
            delete_unverified: retention.delete_unverified(),
            error_if_tagged_old_versions: retention.error_if_tagged_old_versions(),
        },
        pre_optimize_checkpoint: result.pre_optimize_checkpoint().map(checkpoint_view),
    }))
}

pub(super) fn stats_result(stats: &mnemix_core::StatsSnapshot) -> CommandOutput {
    CommandOutput::Stats(Box::new(StatsResultView {
        command: "stats",
        stats: stats_view(stats),
    }))
}

pub(super) fn status_result(
    command: &'static str,
    status: &'static str,
    message: String,
    path: Option<String>,
    schema_version: Option<u64>,
) -> CommandOutput {
    CommandOutput::Status(Box::new(StatusView {
        command,
        status,
        message,
        path,
        schema_version,
    }))
}

pub(super) fn provider_profile_result(
    command: &'static str,
    action: &'static str,
    config_path: String,
    profile: ProviderProfileView,
) -> CommandOutput {
    CommandOutput::ProviderProfile(Box::new(ProviderProfileResultView {
        command,
        action,
        config_path,
        profile,
    }))
}

pub(super) fn provider_profile_list_result(
    command: &'static str,
    config_path: String,
    profiles: Vec<ProviderProfileView>,
) -> CommandOutput {
    CommandOutput::ProviderProfileList(Box::new(ProviderProfileListView {
        command,
        count: profiles.len(),
        config_path,
        profiles,
    }))
}
