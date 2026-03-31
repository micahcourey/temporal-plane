use std::path::Path;

use mnemix_lancedb::{EmbeddingBackfillRequest, VectorEnableRequest};

use crate::{
    cli::{VectorBackfillArgs, VectorEnableArgs, VectorsArgs, VectorsCommand},
    cmd::{open_store, status_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &VectorsArgs) -> Result<CommandOutput, CliError> {
    match &args.command {
        VectorsCommand::Show => show(store_path),
        VectorsCommand::Enable(args) => enable(store_path, args),
        VectorsCommand::Backfill(args) => backfill(store_path, args),
    }
}

fn show(store_path: &Path) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let status = backend.vector_status()?;
    let settings = status.settings();
    let vector_index_reason = status.vector_index().reason().unwrap_or("<none>");

    Ok(status_result(
        "vectors show",
        "ok",
        format!(
            "vectors_enabled={} auto_embed_on_write={} model={} dimensions={} has_provider={} can_embed_on_write={} semantic_retrieval_available={} persisted_embedding_storage={} indexable_embedding_storage={} embedded_memories={}/{} embedding_coverage_percent={} vector_index_available={} vector_index_reason={}",
            settings.vectors_enabled(),
            settings.auto_embed_on_write(),
            settings.embedding_model().unwrap_or("<none>"),
            settings
                .embedding_dimensions()
                .map(|value| value.to_string())
                .unwrap_or_else(|| "<none>".to_owned()),
            status.has_embedding_provider(),
            status.can_embed_on_write(),
            status.semantic_retrieval_available(),
            status.persisted_embedding_storage(),
            status.indexable_embedding_storage(),
            status.embedded_memories(),
            status.total_memories(),
            status.embedding_coverage_percent(),
            status.vector_index().available(),
            vector_index_reason,
        ),
        Some(store_path.display().to_string()),
        Some(backend.schema_version()?),
    ))
}

fn enable(store_path: &Path, args: &VectorEnableArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let request = VectorEnableRequest::new(&args.model, args.dimensions)?
        .with_auto_embed_on_write(args.auto_embed_on_write);
    let settings = backend.enable_vectors(&request)?;

    Ok(status_result(
        "vectors enable",
        "ok",
        format!(
            "enabled vectors with model={} dimensions={} auto_embed_on_write={}",
            settings.embedding_model().unwrap_or("<none>"),
            settings.embedding_dimensions().unwrap_or_default(),
            settings.auto_embed_on_write(),
        ),
        Some(store_path.display().to_string()),
        Some(backend.schema_version()?),
    ))
}

fn backfill(store_path: &Path, args: &VectorBackfillArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let request = if args.apply {
        EmbeddingBackfillRequest::apply()
    } else {
        EmbeddingBackfillRequest::plan()
    };
    let result = backend.backfill_embeddings(&request)?;

    Ok(status_result(
        "vectors backfill",
        "ok",
        format!(
            "apply={} candidate_memories={} updated_memories={}",
            result.apply_writes(),
            result.candidate_memories(),
            result.updated_memories(),
        ),
        Some(store_path.display().to_string()),
        Some(backend.schema_version()?),
    ))
}
