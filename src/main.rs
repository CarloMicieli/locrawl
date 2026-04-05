pub mod cli;
pub mod commands;
pub mod import;
pub mod manifest;

use clap::Parser;
use cli::{Cli, Commands, LogLevel};
use env_logger::{Builder, Env, Target};
use log::error;
use std::process;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    init_logger(cli.log_level);

    // We wrap the match in an async block to unify the return type
    let run_command = async move {
        match cli.command {
            Commands::Info => commands::info::run().await,
            Commands::ImportCollection(args) => commands::import_collection::run(args).await,
            Commands::ImportDigitalRoster(args) => commands::import_digital_roster::run(args).await,
            Commands::ImportTrack(args) => commands::import_track::run(args).await,
            Commands::ImportWishlist(args) => commands::import_wishlist::run(args).await,
            Commands::Validate(args) => commands::validate::run(args).await,
        }
    };

    // Pinning the future is sometimes required for select! in complex scenarios
    tokio::pin!(run_command);

    let result: anyhow::Result<()> = tokio::select! {
        res = &mut run_command => res,
        _ = signal::ctrl_c() => {
            println!("\n[!] Interrupt received. Shutting down gracefully...");
            Ok(())
        }
    };

    if let Err(e) = result {
        error!("{:#}", e);
        process::exit(1);
    }

    Ok(())
}

fn init_logger(log_level: LogLevel) {
    let mut logger = Builder::from_env(Env::default());
    logger.target(Target::Stdout);
    logger.filter_level(log_level.as_level_filter());

    // Ignore double-init errors to keep startup resilient in test environments.
    let _ = logger.try_init();
}
