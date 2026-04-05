use std::path::Path;

use mnemix_core::{RetrievalMode, SearchQuery};

use crate::{
    cli::{RetrievalModeArg, SearchArgs},
    cmd::{
        ensure_provider_store_compatible, open_store, open_store_with_resolved_provider,
        query_limit, search_result,
    },
    errors::CliError,
    output::{CommandOutput, MemorySummaryView, SearchMatchView, memory_summary_view},
    providers_runtime::resolve_named_provider,
};

pub(crate) fn run(store_path: &Path, args: &SearchArgs) -> Result<CommandOutput, CliError> {
    if args.mode.requires_provider() && args.provider.is_none() {
        return Err(CliError::ProviderRequired {
            command: match args.mode {
                RetrievalModeArg::Semantic => "mnemix search --mode semantic",
                RetrievalModeArg::Hybrid => "mnemix search --mode hybrid",
                RetrievalModeArg::Lexical => {
                    unreachable!("lexical search does not require a provider")
                }
            },
        });
    }

    let retrieval_mode: RetrievalMode = args.mode.into();
    let backend = if let Some(provider_name) = args.provider.as_deref() {
        let provider = resolve_named_provider(provider_name)?;
        ensure_provider_store_compatible(
            store_path,
            provider_name,
            &provider,
            match args.mode {
                RetrievalModeArg::Semantic => "mnemix search --mode semantic",
                RetrievalModeArg::Hybrid => "mnemix search --mode hybrid",
                RetrievalModeArg::Lexical => "mnemix search",
            },
        )?;
        open_store_with_resolved_provider(store_path, provider)?
    } else {
        open_store(store_path)?
    };
    let query = SearchQuery::new_with_mode(
        args.text.clone(),
        args.scope.clone(),
        query_limit(args.limit)?,
        retrieval_mode,
    )?;
    let results = backend.search_matches(&query)?;
    let memories = results.iter().map(search_memory_summary_view).collect();

    Ok(search_result(
        args.scope.as_ref().map(|value| value.as_str().to_owned()),
        args.text.clone(),
        retrieval_mode,
        args.provider.clone(),
        memories,
    ))
}

fn search_memory_summary_view(search_match: &mnemix_lancedb::SearchMatch) -> MemorySummaryView {
    let mut summary = memory_summary_view(search_match.record());
    summary.search_match = Some(SearchMatchView {
        kind: search_match_kind(search_match.lexical_match(), search_match.semantic_match()),
        lexical: search_match.lexical_match(),
        semantic: search_match.semantic_match(),
        semantic_score: search_match
            .semantic_score()
            .map(|score| format!("{score:.3}")),
    });
    summary
}

const fn search_match_kind(lexical: bool, semantic: bool) -> &'static str {
    match (lexical, semantic) {
        (true, true) => "hybrid",
        (true, false) => "lexical",
        (false, true) => "semantic",
        (false, false) => "unknown",
    }
}
