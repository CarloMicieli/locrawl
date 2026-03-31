use clap::{Parser, Subcommand, ValueEnum};

use crate::commands::import_collection::ImportCollectionArgs;
use crate::commands::import_digital_roster::ImportDigitalRosterArgs;
use crate::commands::import_track::ImportTrackArgs;
use crate::commands::import_wishlist::ImportWishlistArgs;

#[derive(Parser)]
#[command(name = "locrawl")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "A CLI tool for retrieving railway model data from manufacturer websites and webshops"
)]
pub struct Cli {
    /// Log level for terminal output
    #[arg(long = "log-level", global = true, value_enum, default_value_t = LogLevel::Info)]
    pub log_level: LogLevel,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_level_filter(self) -> log::LevelFilter {
        match self {
            Self::Error => log::LevelFilter::Error,
            Self::Warn => log::LevelFilter::Warn,
            Self::Info => log::LevelFilter::Info,
            Self::Debug => log::LevelFilter::Debug,
            Self::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Display basic tool information
    Info,
    /// Import collection data into a manifest file
    ImportCollection(ImportCollectionArgs),
    /// Import digital roster data into a manifest file
    ImportDigitalRoster(ImportDigitalRosterArgs),
    /// Import track products and inventories into a manifest file
    ImportTrack(ImportTrackArgs),
    /// Import wishlist data into a manifest file
    ImportWishlist(ImportWishlistArgs),
}
