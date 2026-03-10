use std::path::Path;

use mnemix_core::{CheckpointRequest, traits::CheckpointBackend};

use crate::{
    cli::CheckpointArgs,
    cmd::{checkpoint_result, open_store},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &CheckpointArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let request = CheckpointRequest::new(args.name.clone(), args.description.clone());
    let checkpoint = backend.checkpoint(&request)?;
    Ok(checkpoint_result("created checkpoint", &checkpoint))
}
