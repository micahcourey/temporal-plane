use std::path::Path;

use crate::{
    cli::ImportArgs,
    cmd::{open_store, status_result},
    errors::CliError,
    output::CommandOutput,
};

pub(crate) fn run(store_path: &Path, args: &ImportArgs) -> Result<CommandOutput, CliError> {
    let mut backend = open_store(store_path)?;
    let staged = backend.import_store(&args.source)?;
    Ok(status_result(
        "import",
        "ok",
        format!(
            "Staged import from {} on branch {} ({} record{}); main store unchanged",
            args.source.display(),
            staged.branch_name().as_str(),
            staged.staged_records(),
            if staged.staged_records() == 1 {
                ""
            } else {
                "s"
            },
        ),
        Some(store_path.display().to_string()),
        None,
    ))
}
