use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;
use log::info;
use serde_json::Value;

use crate::commands::import_collection::{
    ensure_parent_dir, load_existing_manifest_or_empty, strip_nulls, write_zip,
};
use crate::commands::validation::{manifest_schema_path, validate_value_with_schema};
use crate::import::TrackImport;
use crate::manifest::{Manifest, TrackInventory, TrackProduct};

#[derive(Debug, Args, Clone)]
pub struct ImportTrackArgs {
    /// Path to source track import JSON
    #[arg(short = 's', long = "source")]
    pub source: PathBuf,

    /// Path to zip archive to create or update
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,

    /// Overwrite conflicting existing entries
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

pub async fn run(args: ImportTrackArgs) -> Result<()> {
    let track_schema_path = track_import_schema_path();
    let manifest_schema_path = manifest_schema_path();

    let source_content = fs::read_to_string(&args.source)
        .with_context(|| format!("Failed to read source file '{}'.", args.source.display()))?;
    let source_json: Value = serde_json::from_str(&source_content)
        .with_context(|| format!("Failed to parse JSON from '{}'.", args.source.display()))?;

    validate_value_with_schema(&source_json, &track_schema_path, "track import input")
        .context("Failed to load schema/track_import_schema.json")?;

    let import_data: TrackImport = serde_json::from_value(source_json).with_context(|| {
        format!(
            "Failed to deserialize source data from '{}'.",
            args.source.display()
        )
    })?;

    ensure_no_duplicate_source_keys(&import_data)?;

    let existing_manifest = load_existing_manifest_or_empty(&args.output)?;
    let merged_manifest = merge_track_data(existing_manifest, import_data, args.force)?;

    let mut manifest_value = serde_json::to_value(&merged_manifest)
        .context("Failed to serialize manifest to JSON value")?;
    strip_nulls(&mut manifest_value);
    validate_value_with_schema(&manifest_value, &manifest_schema_path, "manifest output")
        .context("Failed to load schema/manifest_schema.json")?;

    let manifest_json = serde_json::to_string_pretty(&manifest_value)
        .context("Failed to serialize manifest JSON string")?;

    ensure_parent_dir(&args.output)?;
    write_zip(&args.output, &manifest_json)?;

    info!("Manifest successfully written to {}", args.output.display());
    Ok(())
}

fn track_import_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("track_import_schema.json")
}

fn ensure_no_duplicate_source_keys(import: &TrackImport) -> Result<()> {
    let mut seen_track_ids = BTreeSet::new();
    for product in &import.products {
        let track_id = product.track_id.0.clone();
        if !seen_track_ids.insert(track_id.clone()) {
            bail!(
                "Source import contains duplicate trackId '{}' in products.",
                track_id
            );
        }
    }

    let mut seen_inventory_ids = BTreeSet::new();
    for inventory in &import.inventories {
        let inventory_id = inventory.id.0.clone();
        if !seen_inventory_ids.insert(inventory_id.clone()) {
            bail!(
                "Source import contains duplicate inventory id '{}' in inventories.",
                inventory_id
            );
        }
    }

    Ok(())
}

fn merge_track_data(mut manifest: Manifest, import: TrackImport, force: bool) -> Result<Manifest> {
    for incoming_product in import.products {
        if let Some(existing_index) = manifest
            .data
            .track_products
            .iter()
            .position(|product| product.track_id.0 == incoming_product.track_id.0)
        {
            if !force {
                bail!(
                    "Track product with trackId '{}' already exists (use --force to overwrite).",
                    incoming_product.track_id.0
                );
            }

            manifest.data.track_products[existing_index] = incoming_product;
        } else {
            manifest.data.track_products.push(incoming_product);
        }
    }

    for incoming_inventory in import.inventories {
        if let Some(existing_index) = manifest
            .data
            .track_inventories
            .iter()
            .position(|inventory| inventory.id.0 == incoming_inventory.id.0)
        {
            if !force {
                bail!(
                    "Track inventory with id '{}' already exists (use --force to overwrite).",
                    incoming_inventory.id.0
                );
            }

            manifest.data.track_inventories[existing_index] = incoming_inventory;
        } else {
            manifest.data.track_inventories.push(incoming_inventory);
        }
    }

    validate_inventory_track_references(
        &manifest.data.track_products,
        &manifest.data.track_inventories,
    )?;

    Ok(manifest)
}

fn validate_inventory_track_references(
    track_products: &[TrackProduct],
    track_inventories: &[TrackInventory],
) -> Result<()> {
    let known_track_ids: BTreeSet<String> = track_products
        .iter()
        .map(|product| product.track_id.0.clone())
        .collect();

    let mut errors = Vec::new();

    for inventory in track_inventories {
        for item in &inventory.items {
            if !known_track_ids.contains(&item.track_id.0) {
                errors.push(format!(
                    "Track inventory '{}' references unknown trackId '{}' in items.",
                    inventory.id.0, item.track_id.0
                ));
            }
        }

        for purchase in &inventory.purchases {
            if !known_track_ids.contains(&purchase.track_id.0) {
                errors.push(format!(
                    "Track inventory '{}' references unknown trackId '{}' in purchases.",
                    inventory.id.0, purchase.track_id.0
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        bail!(errors.join("\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::import_collection::{
        empty_manifest, load_existing_manifest_or_empty, write_zip,
    };
    use crate::manifest::{
        ManufacturerId, TrackCode, TrackId, TrackInventoryId, TrackProduct, TrackType,
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("locrawl-{}-{}", name, nanos))
    }

    fn sample_product(track_id: &str, product_code: &str) -> TrackProduct {
        TrackProduct {
            track_id: TrackId(track_id.to_string()),
            manufacturer_id: ManufacturerId("trn:manufacturer:marklin".to_string()),
            product_code: product_code.to_string(),
            description: format!("Track {}", product_code),
            track_type: TrackType::Straight,
            track_code: TrackCode::Code83,
            with_roadbed: false,
            length: Some(188),
            radius: None,
        }
    }

    #[tokio::test]
    async fn import_track_appends_product_and_inventory() {
        let source_path = temp_path("track-source");
        let output_path = temp_path("track-output");

        let mut manifest = empty_manifest();
        manifest
            .data
            .track_products
            .push(sample_product("trn:track:marklin:24188", "24188"));

        let seed_json =
            serde_json::to_string_pretty(&manifest).expect("manifest JSON should serialize");
        write_zip(&output_path, &seed_json).expect("seed manifest should be written");

        let source_payload = serde_json::json!({
            "products": [
                {
                    "trackId": "trn:track:marklin:24172",
                    "manufacturerId": "trn:manufacturer:marklin",
                    "productCode": "24172",
                    "description": "Straight Track 171.7 mm",
                    "trackType": "STRAIGHT",
                    "trackCode": "CODE_83",
                    "withRoadbed": false,
                    "length": 172
                }
            ],
            "inventories": [
                {
                    "id": "trn:track-inventory:main-layout",
                    "name": "Main Layout",
                    "items": [
                        { "trackId": "trn:track:marklin:24188", "quantity": 12 },
                        { "trackId": "trn:track:marklin:24172", "quantity": 8 }
                    ]
                }
            ]
        });

        fs::write(
            &source_path,
            serde_json::to_string_pretty(&source_payload).expect("source JSON should serialize"),
        )
        .expect("source file should be written");

        let args = ImportTrackArgs {
            source: source_path.clone(),
            output: output_path.clone(),
            force: false,
        };

        let result = run(args).await;
        assert!(result.is_ok(), "track import should succeed: {:?}", result);

        let merged =
            load_existing_manifest_or_empty(&output_path).expect("output should be readable");

        assert_eq!(merged.data.track_products.len(), 2);
        assert_eq!(merged.data.track_inventories.len(), 1);

        let inventory_id = &merged.data.track_inventories[0].id;
        assert_eq!(
            inventory_id.0,
            TrackInventoryId("trn:track-inventory:main-layout".to_string()).0
        );

        let _ = fs::remove_file(source_path);
        let _ = fs::remove_file(output_path);
    }

    #[tokio::test]
    async fn duplicate_track_id_fails_without_force() {
        let source_path = temp_path("track-duplicate-source");
        let output_path = temp_path("track-duplicate-output");

        let mut manifest = empty_manifest();
        manifest
            .data
            .track_products
            .push(sample_product("trn:track:marklin:24188", "24188"));

        let seed_json =
            serde_json::to_string_pretty(&manifest).expect("manifest JSON should serialize");
        write_zip(&output_path, &seed_json).expect("seed manifest should be written");

        let source_payload = serde_json::json!({
            "products": [
                {
                    "trackId": "trn:track:marklin:24188",
                    "manufacturerId": "trn:manufacturer:marklin",
                    "productCode": "24188",
                    "description": "Straight Track 188 mm",
                    "trackType": "STRAIGHT",
                    "trackCode": "CODE_83",
                    "withRoadbed": false,
                    "length": 188
                }
            ],
            "inventories": []
        });

        fs::write(
            &source_path,
            serde_json::to_string_pretty(&source_payload).expect("source JSON should serialize"),
        )
        .expect("source file should be written");

        let args = ImportTrackArgs {
            source: source_path.clone(),
            output: output_path.clone(),
            force: false,
        };

        let result = run(args).await;
        assert!(
            result.is_err(),
            "import should fail when trackId already exists"
        );

        let error_message = format!("{}", result.expect_err("result should be error"));
        assert!(
            error_message.contains("already exists"),
            "error message should mention existing key, got: {}",
            error_message
        );

        let _ = fs::remove_file(source_path);
        let _ = fs::remove_file(output_path);
    }

    #[tokio::test]
    async fn duplicate_track_id_in_source_fails() {
        let source_path = temp_path("track-source-duplicates");
        let output_path = temp_path("track-source-duplicates-output");

        let source_payload = serde_json::json!({
            "products": [
                {
                    "trackId": "trn:track:marklin:24188",
                    "manufacturerId": "trn:manufacturer:marklin",
                    "productCode": "24188",
                    "description": "Straight Track 188 mm",
                    "trackType": "STRAIGHT",
                    "trackCode": "CODE_83",
                    "withRoadbed": false,
                    "length": 188
                },
                {
                    "trackId": "trn:track:marklin:24188",
                    "manufacturerId": "trn:manufacturer:marklin",
                    "productCode": "24188B",
                    "description": "Duplicate entry",
                    "trackType": "STRAIGHT",
                    "trackCode": "CODE_83",
                    "withRoadbed": false,
                    "length": 188
                }
            ],
            "inventories": []
        });

        fs::write(
            &source_path,
            serde_json::to_string_pretty(&source_payload).expect("source JSON should serialize"),
        )
        .expect("source file should be written");

        let args = ImportTrackArgs {
            source: source_path.clone(),
            output: output_path.clone(),
            force: false,
        };

        let result = run(args).await;
        assert!(result.is_err(), "import should fail for source duplicates");

        let error_message = format!("{}", result.expect_err("result should be error"));
        assert!(
            error_message.contains("duplicate trackId"),
            "error message should mention duplicate source trackId, got: {}",
            error_message
        );

        let _ = fs::remove_file(source_path);
        let _ = fs::remove_file(output_path);
    }
}
