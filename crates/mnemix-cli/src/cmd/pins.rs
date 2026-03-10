use std::path::Path;

use crate::{
    cli::PinsArgs,
    cmd::{memory_list_result, open_store, query_limit},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &PinsArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let results = backend.list_pinned_memories(args.scope.as_ref(), query_limit(args.limit)?)?;
    Ok(memory_list_result(
        "pins",
        args.scope.as_ref().map(|value| value.as_str().to_owned()),
        None,
        &results,
    ))
}
