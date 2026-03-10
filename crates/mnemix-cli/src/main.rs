#![cfg_attr(test, allow(unused_crate_dependencies))]

//! CLI entry point for Mnemix.

mod cli;
mod cmd;
mod errors;
mod output;

use std::process::ExitCode;

use clap::Parser;

use crate::{
    cli::Cli,
    output::{render_error, render_output},
};

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cmd::execute(&cli.command, &cli.store) {
        Ok(result) => match render_output(&result, cli.output_format()) {
            Ok(rendered) => {
                print!("{rendered}");
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprint!("{}", render_error(&error, cli.output_format()));
                ExitCode::FAILURE
            }
        },
        Err(error) => {
            eprint!("{}", render_error(&error, cli.output_format()));
            ExitCode::FAILURE
        }
    }
}
