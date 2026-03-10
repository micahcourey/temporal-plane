use std::path::Path;

use mnemix_core::traits::MemoryRepository;

use crate::{
    cli::RememberArgs,
    cmd::{build_memory_record, memory_result, open_store},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &RememberArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let record = build_memory_record(args)?;
    let stored = backend.remember(record)?;
    Ok(memory_result("remember", "stored memory", &stored))
}
