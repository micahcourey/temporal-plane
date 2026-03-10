use std::path::Path;

use mnemix_core::{StatsQuery, traits::StatsBackend};

use crate::{
    cli::StatsArgs,
    cmd::{open_store, stats_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &StatsArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let stats = backend.stats(&StatsQuery::new(args.scope.clone()))?;
    Ok(stats_result(&stats))
}
