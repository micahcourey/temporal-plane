use std::path::Path;

use crate::{cmd::open_or_init_store, errors::CliError, output::CommandOutput};

use super::status_result;

pub(crate) fn run(store_path: &Path) -> Result<CommandOutput, CliError> {
    let backend = open_or_init_store(store_path)?;
    let schema_version = backend.schema_version()?;
    Ok(status_result(
        "init",
        "ok",
        "Initialized Mnemix store".to_owned(),
        Some(store_path.display().to_string()),
        Some(schema_version),
    ))
}
