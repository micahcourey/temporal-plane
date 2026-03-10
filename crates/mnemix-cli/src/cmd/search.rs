use std::path::Path;

use mnemix_core::{SearchQuery, traits::RecallBackend};

use crate::{
    cli::SearchArgs,
    cmd::{memory_list_result, open_store, query_limit},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &SearchArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let query = SearchQuery::new(
        args.text.clone(),
        args.scope.clone(),
        query_limit(args.limit)?,
    )?;
    let results = backend.search(&query)?;
    Ok(memory_list_result(
        "search",
        args.scope.as_ref().map(|value| value.as_str().to_owned()),
        Some(args.text.clone()),
        &results,
    ))
}
