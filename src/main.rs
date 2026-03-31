mod cli;
mod commands;
pub mod import;
pub mod manifest;

use clap::Parser;
use cli::{Cli, Commands, LogLevel};
use env_logger::{Builder, Env, Target};
use log::error;
use std::process;

fn main() {
    let cli = Cli::parse();
    init_logger(cli.log_level);

    let result: anyhow::Result<()> = match cli.command {
        Commands::Info => commands::info::run(),
        Commands::ImportCollection(args) => commands::import_collection::run(args),
        Commands::ImportDigitalRoster(args) => commands::import_digital_roster::run(args),
        Commands::ImportTrack(args) => commands::import_track::run(args),
        Commands::ImportWishlist(args) => commands::import_wishlist::run(args),
        Commands::Validate(args) => commands::validate::run(args),
    };

    if let Err(e) = result {
        error!("{}", e);
        process::exit(1);
    }
}

fn init_logger(log_level: LogLevel) {
    let mut logger = Builder::from_env(Env::default());
    logger.target(Target::Stdout);
    logger.filter_level(log_level.as_level_filter());

    // Ignore double-init errors to keep startup resilient in test environments.
    let _ = logger.try_init();
}
