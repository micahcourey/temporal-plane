use std::path::Path;

use mnemix_core::{RecallQuery, traits::RecallBackend};

use crate::{
    cli::RecallArgs,
    cmd::{open_store, query_limit, recall_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &RecallArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let mut builder = RecallQuery::builder()
        .disclosure_depth(args.disclosure_depth.into())
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
        &result,
    ))
}
