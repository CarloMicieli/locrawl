use clap::{Parser, Subcommand};

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
}
