mod cli;
mod commands;
pub mod import;
pub mod manifest;

use clap::Parser;
use cli::{Cli, Commands};
use std::process;

fn main() {
    let cli = Cli::parse();

    let result: anyhow::Result<()> = match cli.command {
        Commands::Info => commands::info::run(),
        Commands::Import(args) => commands::import::run(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
