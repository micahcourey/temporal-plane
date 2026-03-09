use std::path::Path;

use temporal_plane_core::{HistoryQuery, traits::HistoryBackend};

use crate::{
    cli::VersionsArgs,
    cmd::{open_store, query_limit, version_list_result},
    errors::CliError,
    output::CommandOutput,
};

/// Lists store-wide versions.
///
/// This command mirrors the backend's current unscoped version listing
/// behavior and does not accept a scope filter.
pub(crate) fn run(store_path: &Path, args: &VersionsArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let query = HistoryQuery::new(None, query_limit(args.limit)?);
    let versions = backend.history(&query)?;
    Ok(version_list_result("versions", None, &versions))
}
