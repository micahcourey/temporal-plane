use std::path::Path;

use mnemix_lancedb::{EmbeddingBackfillRequest, LanceDbBackend, VectorEnableRequest, VectorStatus};

use crate::{
    cli::{VectorBackfillArgs, VectorEnableArgs, VectorRuntimeArgs, VectorsArgs, VectorsCommand},
    cmd::{
        ProviderStoreCompatibility, ensure_provider_store_compatible, open_store,
        open_store_with_provider, open_store_with_resolved_provider, provider_store_compatibility,
        status_result,
    },
    errors::CliError,
    output::CommandOutput,
    providers_runtime::resolve_named_provider,
};

pub(crate) fn run(store_path: &Path, args: &VectorsArgs) -> Result<CommandOutput, CliError> {
    match &args.command {
        VectorsCommand::Show(args) => show(store_path, args),
        VectorsCommand::Enable(args) => enable(store_path, args),
        VectorsCommand::Backfill(args) => backfill(store_path, args),
    }
}

fn show(store_path: &Path, args: &VectorRuntimeArgs) -> Result<CommandOutput, CliError> {
    let provider_name = args.provider.as_deref().unwrap_or("<none>");

    if let Some(provider_name) = args.provider.as_deref() {
        let provider = resolve_named_provider(provider_name)?;
        let compatibility = provider_store_compatibility(store_path, &provider)?;
        match compatibility {
            ProviderStoreCompatibility::Compatible { .. } => {
                let backend = open_store_with_resolved_provider(store_path, provider.clone())?;
                return vector_show_result(
                    store_path,
                    &backend,
                    "ok",
                    provider_name,
                    Some((&provider.model_id, provider.dimensions)),
                    Some(&compatibility),
                );
            }
            ProviderStoreCompatibility::StoreNotInitialized => {
                return Err(CliError::StoreNotInitialized {
                    path: store_path.to_path_buf(),
                });
            }
            compatibility => {
                let backend = open_store(store_path)?;
                return vector_show_result(
                    store_path,
                    &backend,
                    "mismatch",
                    provider_name,
                    Some((&provider.model_id, provider.dimensions)),
                    Some(&compatibility),
                );
            }
        }
    }

    let backend = open_store_with_provider(store_path, None)?;
    vector_show_result(store_path, &backend, "ok", provider_name, None, None)
}

fn enable(store_path: &Path, args: &VectorEnableArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let (provider_name, model, dimensions) = if let Some(provider_name) = args.provider.as_deref() {
        let provider = resolve_named_provider(provider_name)?;
        (
            Some(provider_name.to_owned()),
            provider.model_id,
            provider.dimensions,
        )
    } else {
        (
            None,
            args.model
                .clone()
                .expect("clap should require model when provider is omitted"),
            args.dimensions
                .expect("clap should require dimensions when provider is omitted"),
        )
    };
    let request = VectorEnableRequest::new(&model, dimensions)?
        .with_auto_embed_on_write(args.auto_embed_on_write);
    let settings = backend.enable_vectors(&request)?;

    Ok(status_result(
        "vectors enable",
        "ok",
        format!(
            "enabled vectors with provider={} model={} dimensions={} auto_embed_on_write={}",
            provider_name.as_deref().unwrap_or("<manual>"),
            settings.embedding_model().unwrap_or("<none>"),
            settings.embedding_dimensions().unwrap_or_default(),
            settings.auto_embed_on_write(),
        ),
        Some(store_path.display().to_string()),
        Some(backend.schema_version()?),
    ))
}

fn backfill(store_path: &Path, args: &VectorBackfillArgs) -> Result<CommandOutput, CliError> {
    if args.apply && args.provider.is_none() {
        return Err(CliError::ProviderRequired {
            command: "mnemix vectors backfill --apply",
        });
    }

    let mut backend = if let Some(provider_name) = args.provider.as_deref() {
        let provider = resolve_named_provider(provider_name)?;
        ensure_provider_store_compatible(
            store_path,
            provider_name,
            &provider,
            "mnemix vectors backfill --apply",
        )?;
        open_store_with_resolved_provider(store_path, provider)?
    } else {
        open_store_with_provider(store_path, None)?
    };
    let request = if args.apply {
        EmbeddingBackfillRequest::apply()
    } else {
        EmbeddingBackfillRequest::plan()
    };
    let result = backend.backfill_embeddings(&request)?;
    let provider_name = args.provider.as_deref().unwrap_or("<none>");

    Ok(status_result(
        "vectors backfill",
        "ok",
        format!(
            "provider={} apply={} candidate_memories={} updated_memories={}",
            provider_name,
            result.apply_writes(),
            result.candidate_memories(),
            result.updated_memories(),
        ),
        Some(store_path.display().to_string()),
        Some(backend.schema_version()?),
    ))
}

fn vector_show_result(
    store_path: &Path,
    backend: &LanceDbBackend,
    status_label: &'static str,
    provider_name: &str,
    provider_details: Option<(&str, u32)>,
    compatibility: Option<&ProviderStoreCompatibility>,
) -> Result<CommandOutput, CliError> {
    let status = backend.vector_status()?;

    Ok(status_result(
        "vectors show",
        status_label,
        vector_show_message(provider_name, provider_details, compatibility, &status),
        Some(store_path.display().to_string()),
        Some(backend.schema_version()?),
    ))
}

fn vector_show_message(
    provider_name: &str,
    provider_details: Option<(&str, u32)>,
    compatibility: Option<&ProviderStoreCompatibility>,
    status: &VectorStatus,
) -> String {
    let settings = status.settings();
    let provider_prefix = provider_details.map_or_else(
        || format!("provider={provider_name} "),
        |(model, dimensions)| {
            format!(
                "provider={} provider_model={} provider_dimensions={} provider_compatibility={} compatibility_detail={} ",
                provider_name,
                model,
                dimensions,
                compatibility.map_or("unknown", ProviderStoreCompatibility::label),
                compatibility.map_or_else(
                    || "compatibility not checked".to_owned(),
                    ProviderStoreCompatibility::detail,
                ),
            )
        },
    );

    format!(
        "{}vectors_enabled={} auto_embed_on_write={} model={} dimensions={} has_provider={} can_embed_on_write={} semantic_retrieval_available={} persisted_embedding_storage={} indexable_embedding_storage={} embedded_memories={}/{} embedding_coverage_percent={} vector_index_available={} vector_index_reason={}",
        provider_prefix,
        settings.vectors_enabled(),
        settings.auto_embed_on_write(),
        settings.embedding_model().unwrap_or("<none>"),
        settings
            .embedding_dimensions()
            .map_or_else(|| "<none>".to_owned(), |value| value.to_string()),
        status.has_embedding_provider(),
        status.can_embed_on_write(),
        status.semantic_retrieval_available(),
        status.persisted_embedding_storage(),
        status.indexable_embedding_storage(),
        status.embedded_memories(),
        status.total_memories(),
        status.embedding_coverage_percent(),
        status.vector_index().available(),
        status.vector_index().reason().unwrap_or("<none>"),
    )
}
