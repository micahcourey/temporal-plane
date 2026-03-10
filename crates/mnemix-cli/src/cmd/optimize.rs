use std::path::Path;

use mnemix_core::{CleanupMode, OptimizeRequest, RetentionPolicy, traits::OptimizeBackend};

use crate::{
    cli::OptimizeArgs,
    cmd::{open_store, optimize_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &OptimizeArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let retention = RetentionPolicy::conservative()
        .with_minimum_age_days(args.older_than_days)
        .with_cleanup_mode(if args.prune {
            CleanupMode::AllowPrune
        } else {
            CleanupMode::ExplicitOnly
        });
    let request = OptimizeRequest::new(retention).with_prune_old_versions(args.prune);
    let result = backend.optimize(&request)?;
    Ok(optimize_result(&request, &result))
}
