use std::path::PathBuf;

use anyhow::{Result, bail};
use clap::Args;
use colored::Colorize;

use crate::commands::SchemaType;
use crate::commands::validation::{schema_path_for_type, validate_file};

#[derive(Debug, Args, Clone)]
pub struct ValidateArgs {
    /// Schema type used for validation
    #[arg(value_enum, ignore_case = true)]
    pub schema_type: SchemaType,

    /// Path to source JSON to validate
    #[arg(short = 's', long = "source")]
    pub source: PathBuf,
}

pub async fn run(args: ValidateArgs) -> Result<()> {
    let schema_path = schema_path_for_type(args.schema_type);

    match validate_file(&args.source, &schema_path) {
        Ok(()) => {
            println!(
                "{}",
                format!(
                    "✅ File is valid according to the {} schema.",
                    schema_type_name(args.schema_type)
                )
                .green()
            );
            Ok(())
        }
        Err(error) => {
            println!(
                "{}",
                format!(
                    "Validation failed for {} schema:",
                    schema_type_name(args.schema_type)
                )
                .red()
            );

            for line in error.to_string().lines() {
                println!("{}", line.red());
            }

            bail!("validation failed")
        }
    }
}

fn schema_type_name(schema_type: SchemaType) -> &'static str {
    match schema_type {
        SchemaType::Collection => "Collection",
        SchemaType::Wishlist => "Wishlist",
        SchemaType::DigitalRoster => "DigitalRoster",
        SchemaType::Track => "Track",
        SchemaType::Manifest => "Manifest",
    }
}
