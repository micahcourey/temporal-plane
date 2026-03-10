use std::path::Path;

use mnemix_core::traits::MemoryRepository;

use crate::{
    cli::ShowArgs,
    cmd::{memory_result, open_store},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &ShowArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    let record = backend
        .get(&args.id)?
        .ok_or_else(|| CliError::MemoryNotFound {
            id: args.id.as_str().to_owned(),
        })?;
    Ok(memory_result("show", "displayed memory", &record))
}
