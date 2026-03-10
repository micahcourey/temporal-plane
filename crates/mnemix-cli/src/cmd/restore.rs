use std::path::Path;

use mnemix_core::{CheckpointSelector, RestoreRequest, VersionNumber, traits::RestoreBackend};

use crate::{
    cli::RestoreArgs,
    cmd::{open_store, restore_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &RestoreArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let target = if let Some(name) = &args.checkpoint {
        CheckpointSelector::Named(name.clone())
    } else if let Some(version) = args.version {
        CheckpointSelector::Version(VersionNumber::new(version))
    } else {
        unreachable!("run() must be called with either --checkpoint or --version")
    };
    let result = backend.restore(&RestoreRequest::new(target))?;
    Ok(restore_result(&result))
}
