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
        Commands::ImportCollection(args) => commands::import_collection::run(args),
        Commands::ImportWishlist(args) => commands::import_wishlist::run(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}
