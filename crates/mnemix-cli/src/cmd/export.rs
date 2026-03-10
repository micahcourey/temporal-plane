use std::path::Path;

use crate::{
    cli::ExportArgs,
    cmd::{open_store, status_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &ExportArgs) -> Result<CommandOutput, CliError> {
    let backend = open_store(store_path)?;
    backend.export_store(&args.destination)?;
    Ok(status_result(
        "export",
        "ok",
        format!("Exported store to {}", args.destination.display()),
        Some(store_path.display().to_string()),
        None,
    ))
}
