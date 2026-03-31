use clap::{Parser, Subcommand};

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
    #[command(subcommand)]
    pub command: Commands,
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
