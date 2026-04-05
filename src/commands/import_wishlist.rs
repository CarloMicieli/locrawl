use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;
use clap::Args;
use log::info;
use serde_json::Value;

use crate::commands::import_collection::{
    SeedData, ensure_parent_dir, load_existing_manifest_or_empty, load_seed_data, make_model_id,
    map_railway_model, normalize_id_segment, parse_rfc3339_to_utc, strip_nulls, trn_slug,
    write_zip,
};
use crate::commands::validation::{manifest_schema_path, validate_value_with_schema};
use crate::import::{Wishlist as ImportWishlist, WishlistPriority as ImportWishlistPriority};
use crate::manifest::{
    self, DataContainer, Manifest, Manufacturer, ManufacturerId, RailwayCompany, RailwayModel,
    Wishlist, WishlistItem, WishlistPriority, WishlistStatus,
};

#[derive(Debug, Args, Clone)]
pub struct ImportWishlistArgs {
    /// Path to source wishlist JSON
    #[arg(short = 's', long = "source")]
    pub source: PathBuf,

    /// Path to zip archive to create or update
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,

    /// Overwrite conflicting wishlist entries by name
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

pub async fn run(args: ImportWishlistArgs) -> Result<()> {
    let wishlist_schema_path = wishlist_schema_path();
    let manifest_schema_path = manifest_schema_path();

    let source_content = std::fs::read_to_string(&args.source)
        .with_context(|| format!("Failed to read source file '{}'.", args.source.display()))?;
    let source_json: Value = serde_json::from_str(&source_content)
        .with_context(|| format!("Failed to parse JSON from '{}'.", args.source.display()))?;

    validate_value_with_schema(&source_json, &wishlist_schema_path, "wishlist import input")
        .context("Failed to load schema/wishlist_schema.json")?;

    let import_wishlist: ImportWishlist =
        serde_json::from_value(source_json).with_context(|| {
            format!(
                "Failed to deserialize source data from '{}'.",
                args.source.display()
            )
        })?;

    let seed_data = load_seed_data()?;
    let incoming_manifest = map_wishlist_to_manifest(&import_wishlist, &seed_data)?;
    let existing_manifest = load_existing_manifest_or_empty(&args.output)?;
    let merged_manifest =
        merge_wishlist_manifests(existing_manifest, incoming_manifest, args.force)?;

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

fn wishlist_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("wishlist_schema.json")
}

fn map_wishlist_to_manifest(import: &ImportWishlist, seeds: &SeedData) -> Result<Manifest> {
    let exported_at = parse_rfc3339_to_utc(&import.modified_at, "modifiedAt")?;
    let added_date = exported_at.date_naive();

    let mut manufacturers: BTreeMap<String, Manufacturer> = BTreeMap::new();
    let mut railway_companies: BTreeMap<String, RailwayCompany> = BTreeMap::new();
    let mut railway_models: Vec<RailwayModel> = Vec::with_capacity(import.railway_models.len());
    let mut wishlist_items: Vec<WishlistItem> = Vec::with_capacity(import.railway_models.len());

    for model in &import.railway_models {
        let manufacturer_slug = trn_slug(&model.manufacturer, "trn:manufacturer:");
        let manufacturer_id = ManufacturerId(format!("trn:manufacturer:{}", manufacturer_slug));

        let seed_manufacturer = seeds
            .manufacturers
            .get(&manufacturer_slug)
            .with_context(|| {
                format!(
                    "Manufacturer '{}' (slug: '{}') not found in seed/manufacturers.csv",
                    model.manufacturer, manufacturer_slug
                )
            })?;
        manufacturers
            .entry(manufacturer_id.0.clone())
            .or_insert_with(|| seed_manufacturer.clone());

        railway_models.push(map_railway_model(
            model,
            &manufacturer_id,
            &mut railway_companies,
            seeds,
        )?);

        let railway_model_id = make_model_id(&model.manufacturer, &model.product_code);
        let wishlist_slug = normalize_id_segment(&import.name);
        let item_id = format!(
            "trn:wishlist-item:{}:{}:{}",
            wishlist_slug,
            normalize_id_segment(&model.manufacturer),
            normalize_id_segment(&model.product_code)
        );

        let desired_price = model.wishlist_info.as_ref().and_then(|info| {
            info.wanted_price.as_ref().map(|price| manifest::Money {
                amount: price.amount.round() as i64,
                currency: price.currency.clone(),
            })
        });

        let notes = model
            .wishlist_info
            .as_ref()
            .and_then(|info| info.notes.clone());

        let priority = model
            .wishlist_info
            .as_ref()
            .and_then(|info| info.priority.as_ref())
            .map(map_priority)
            .unwrap_or(WishlistPriority::Normal);

        wishlist_items.push(WishlistItem {
            id: item_id,
            railway_model_id,
            priority,
            status: WishlistStatus::Wanted,
            added_date,
            removed_date: None,
            notes,
            desired_price,
            purchased_price: None,
        });
    }

    let wishlist = Wishlist {
        id: format!("trn:wishlist:{}", normalize_id_segment(&import.name)),
        name: import.name.clone(),
        notes: import.description.clone(),
        is_default: import.is_default.unwrap_or(false),
        items: wishlist_items,
    };

    Ok(Manifest {
        schema: Some("https://rusty-shed.app/schemas/manifest/v1.json".to_string()),
        version: crate::manifest::ManifestVersion::V1_0,
        exported_at: Some(exported_at),
        source: Some(format!("locrawl {}", env!("CARGO_PKG_VERSION"))),
        data: DataContainer {
            manufacturers: manufacturers.into_values().collect(),
            railway_companies: railway_companies.into_values().collect(),
            railway_models,
            collection_items: vec![],
            sellers: vec![],
            maintenance_cards: vec![],
            track_products: vec![],
            track_inventories: vec![],
            prototypes: vec![],
            formation_categories: vec![],
            train_formations: vec![],
            wishlists: vec![wishlist],
            decoders: vec![],
            digital_rolling_stocks: vec![],
        },
    })
}

fn map_priority(priority: &ImportWishlistPriority) -> WishlistPriority {
    match priority {
        ImportWishlistPriority::High => WishlistPriority::High,
        ImportWishlistPriority::Normal => WishlistPriority::Normal,
        ImportWishlistPriority::Low => WishlistPriority::Low,
    }
}

fn merge_wishlist_manifests(
    mut existing: Manifest,
    incoming: Manifest,
    force: bool,
) -> Result<Manifest> {
    let incoming_wishlist = incoming
        .data
        .wishlists
        .first()
        .cloned()
        .context("Incoming wishlist payload is empty")?;

    let conflict_idx = existing
        .data
        .wishlists
        .iter()
        .position(|wishlist| wishlist.name == incoming_wishlist.name);

    if let Some(index) = conflict_idx {
        if !force {
            bail!(
                "Wishlist conflict found for name '{}'. Re-run with --force to overwrite.",
                incoming_wishlist.name
            );
        }
        existing.data.wishlists.remove(index);
    }

    merge_models_by_id(
        &mut existing.data.manufacturers,
        incoming.data.manufacturers,
        force,
    );
    merge_models_by_id(
        &mut existing.data.railway_companies,
        incoming.data.railway_companies,
        force,
    );
    merge_models_by_id(
        &mut existing.data.railway_models,
        incoming.data.railway_models,
        force,
    );
    existing.data.wishlists.push(incoming_wishlist);

    existing.source = Some(format!("locrawl {}", env!("CARGO_PKG_VERSION")));
    if existing.exported_at.is_none() {
        existing.exported_at = Some(Utc::now());
    }

    Ok(existing)
}

fn merge_models_by_id<T>(existing: &mut Vec<T>, incoming: Vec<T>, force: bool)
where
    T: Clone,
    T: WishlistMergeId,
{
    for incoming_item in incoming {
        if let Some(index) = existing
            .iter()
            .position(|existing_item| existing_item.merge_id() == incoming_item.merge_id())
        {
            if force {
                existing[index] = incoming_item;
            }
        } else {
            existing.push(incoming_item);
        }
    }
}

trait WishlistMergeId {
    fn merge_id(&self) -> String;
}

impl WishlistMergeId for Manufacturer {
    fn merge_id(&self) -> String {
        self.id.0.clone()
    }
}

impl WishlistMergeId for RailwayCompany {
    fn merge_id(&self) -> String {
        self.id.0.clone()
    }
}

impl WishlistMergeId for RailwayModel {
    fn merge_id(&self) -> String {
        self.id.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::import_collection::load_existing_manifest_or_empty;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("locrawl-{}-{}", name, nanos))
    }

    fn sample_wishlist(name: &str, product_code: &str) -> Value {
        serde_json::json!({
            "version": 1,
            "modifiedAt": "2026-03-31T12:00:00Z",
            "name": name,
            "railwayModels": [
                {
                    "id": format!("rm-{}", product_code),
                    "manufacturer": "trn:manufacturer:acme",
                    "productCode": product_code,
                    "description": "Demo locomotive",
                    "powerMethod": "DC",
                    "scale": "H0",
                    "epoch": "IV",
                    "category": "LOCOMOTIVES",
                    "rollingStocks": [
                        {
                            "id": "stock-1",
                            "category": "LOCOMOTIVE",
                            "railway": "trn:railway-company:db"
                        }
                    ],
                    "wishlistInfo": {
                        "wantedPrice": {
                            "amount": 175.0,
                            "currency": "EUR"
                        },
                        "priority": "normal"
                    }
                }
            ]
        })
    }

    #[tokio::test]
    async fn append_wishlist_adds_second_named_wishlist() {
        let source_one = temp_path("wishlist-source-one.json");
        let source_two = temp_path("wishlist-source-two.json");
        let output = temp_path("wishlist-manifest.json");

        fs::write(
            &source_one,
            serde_json::to_string(&sample_wishlist("Primary", "A-123")).expect("json"),
        )
        .expect("source one write should succeed");
        fs::write(
            &source_two,
            serde_json::to_string(&sample_wishlist("Secondary", "B-456")).expect("json"),
        )
        .expect("source two write should succeed");

        run(ImportWishlistArgs {
            source: source_one.clone(),
            output: output.clone(),
            force: false,
        })
        .await
        .expect("first wishlist import should succeed");

        run(ImportWishlistArgs {
            source: source_two.clone(),
            output: output.clone(),
            force: false,
        })
        .await
        .expect("second wishlist import should append");

        let manifest = load_existing_manifest_or_empty(&output).expect("manifest should exist");
        assert_eq!(manifest.data.wishlists.len(), 2);

        let _ = fs::remove_file(source_one);
        let _ = fs::remove_file(source_two);
        let _ = fs::remove_file(output);
    }
}
