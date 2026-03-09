use std::path::Path;

use temporal_plane_core::{HistoryQuery, traits::HistoryBackend};

use crate::{
    cli::HistoryArgs,
    cmd::{open_store, query_limit, version_list_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &HistoryArgs) -> Result<CommandOutput, CliError> {
    if args.scope.is_some() {
        return Err(CliError::ScopedHistoryNotSupported);
    }

    let backend = open_store(store_path)?;
    let query = HistoryQuery::new(args.scope.clone(), query_limit(args.limit)?);
    let versions = backend.history(&query)?;
    Ok(version_list_result(
        "history",
        args.scope.as_ref().map(|value| value.as_str().to_owned()),
        &versions,
    ))
}
