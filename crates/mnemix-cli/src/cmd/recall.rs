use std::path::Path;

use mnemix_core::{RecallQuery, RetrievalMode, traits::RecallBackend};

use crate::{
    cli::{RecallArgs, RetrievalModeArg},
    cmd::{
        ensure_provider_store_compatible, open_store, open_store_with_resolved_provider,
        query_limit, recall_result,
    },
    errors::CliError,
    output::CommandOutput,
    providers_runtime::resolve_named_provider,
};

pub(crate) fn run(store_path: &Path, args: &RecallArgs) -> Result<CommandOutput, CliError> {
    if args.mode.requires_provider() && args.provider.is_none() {
        return Err(CliError::ProviderRequired {
            command: match args.mode {
                RetrievalModeArg::Semantic => "mnemix recall --mode semantic",
                RetrievalModeArg::Hybrid => "mnemix recall --mode hybrid",
                RetrievalModeArg::Lexical => {
                    unreachable!("lexical recall does not require a provider")
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
                RetrievalModeArg::Semantic => "mnemix recall --mode semantic",
                RetrievalModeArg::Hybrid => "mnemix recall --mode hybrid",
                RetrievalModeArg::Lexical => "mnemix recall",
            },
        )?;
        open_store_with_resolved_provider(store_path, provider)?
    } else {
        open_store(store_path)?
    };
    let mut builder = RecallQuery::builder()
        .disclosure_depth(args.disclosure_depth.into())
        .retrieval_mode(retrieval_mode)
        .limit(query_limit(args.limit)?);

    if let Some(scope) = &args.scope {
        builder = builder.scope(scope.clone());
    }
    if let Some(text) = &args.text {
        builder = builder.text(text.clone())?;
    }

    let query = builder.build()?;
    let result = backend.recall(&query)?;
    Ok(recall_result(
        args.scope.as_ref().map(|value| value.as_str().to_owned()),
        args.text.clone(),
        retrieval_mode,
        args.provider.clone(),
        &result,
    ))
}
