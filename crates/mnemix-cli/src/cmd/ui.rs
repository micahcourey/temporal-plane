use std::path::Path;

use crate::{
    cli::UiArgs,
    cmd::{open_store, query_limit},
    errors::CliError,
    tui,
};

pub(crate) fn run(store_path: &Path, args: &UiArgs) -> Result<(), CliError> {
    let backend = open_store(store_path)?;
    let limit = query_limit(args.limit)?;
    tui::run(&backend, limit)
}
