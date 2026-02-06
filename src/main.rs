mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};
use std::process;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Info => commands::info::run(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
