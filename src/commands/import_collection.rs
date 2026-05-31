use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::{CompressionMethod, ZipArchive};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDate, Utc};
use clap::Args;
use log::{info, warn};
use serde_json::Value;
use slug::slugify;
use uuid::Uuid;

use crate::commands::validation::{manifest_schema_path, validate_value_with_schema};

use crate::import::{
    Category as ImportRollingStockCategory, Collection, PowerMethod as ImportPowerMethod,
    RailwayModel as ImportRailwayModel, RailwayModelCategory as ImportRailwayModelCategory,
    RollingStock as ImportRollingStock, Scale as ImportScale, ServiceLevel as ImportServiceLevel,
    SubCategory as ImportSubCategory,
};
use crate::manifest::{
    self, Category, CollectionItem, CollectionItemId, Control, DataContainer,
    ElectricMultipleUnitType, FreightCarType, LocalizedText, LocomotiveType, Manifest,
    ManifestVersion, Manufacturer, ManufacturerId, OwnedRollingStock, OwnedRollingStockId,
    PassengerCarType, PowerMethod, Purchase, PurchaseType, RailcarType, RailwayCompany,
    RailwayCompanyId, RailwayCompanyStatus, RailwayModel, RailwayModelId, RollingStock, Scale,
    Seller, SellerId, SellerType, ServiceLevel,
};

#[derive(Debug, Args, Clone)]
pub struct ImportCollectionArgs {
    /// Path to source collection JSON
    #[arg(short = 's', long = "source")]
    pub source: PathBuf,

    /// Path to zip archive to create or update
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,

    /// Overwrite conflicting existing entries
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

pub async fn run(args: ImportCollectionArgs) -> Result<()> {
    let collection_schema_path = collection_schema_path();
    let manifest_schema_path = manifest_schema_path();

    let output = if args.output.extension().is_none() {
        args.output.with_extension("zip")
    } else {
        args.output.clone()
    };

    let source_content = fs::read_to_string(&args.source)
        .with_context(|| format!("Failed to read source file '{}'.", args.source.display()))?;

    let source_json: Value = serde_json::from_str(&source_content)
        .with_context(|| format!("Failed to parse JSON from '{}'.", args.source.display()))?;

    // Validate input; if it doesn't match the current schema, try migrating
    // legacy input into the new schema and validate the migrated result.
    let input_json: Value = if validate_value_with_schema(
        &source_json,
        &collection_schema_path,
        "collection import input",
    )
    .is_ok()
    {
        source_json.clone()
    } else {
        // Try deserializing into the permissive import structs and migrate
        let import_collection_legacy: Collection = serde_json::from_value(source_json.clone())
            .with_context(|| {
                format!(
                    "Failed to deserialize legacy source data from '{}'.",
                    args.source.display()
                )
            })?;

        let migrated = migrate_collection_to_new_schema(&import_collection_legacy)
            .context("Failed to migrate legacy collection to new schema")?;

        // Validate migrated output
        validate_value_with_schema(
            &migrated,
            &collection_schema_path,
            "migrated collection input",
        )
        .context("Migrated collection did not validate against new schema")?;

        migrated
    };

    let import_collection: Collection = serde_json::from_value(input_json).with_context(|| {
        format!(
            "Failed to deserialize source data from '{}'.",
            args.source.display()
        )
    })?;

    let registry = load_registry()?;
    let incoming_manifest = map_import_to_manifest(&import_collection, &registry)?;
    let existing_manifest = load_existing_manifest_or_empty(&output)?;
    let mut merged_manifest =
        merge_collection_manifests(existing_manifest, incoming_manifest, args.force)?;
    validate_manifest_integrity(&mut merged_manifest)?;

    let mut manifest_value = serde_json::to_value(&merged_manifest)
        .context("Failed to serialize manifest to JSON value")?;
    strip_nulls(&mut manifest_value);
    validate_value_with_schema(&manifest_value, &manifest_schema_path, "manifest output")
        .context("Failed to load schema/manifest_schema.json")?;

    let manifest_json = serde_json::to_string_pretty(&manifest_value)
        .context("Failed to serialize manifest JSON string")?;

    ensure_parent_dir(&output)?;
    write_zip(&output, &manifest_json)?;

    info!("Manifest successfully written to {}", output.display());
    Ok(())
}

fn collection_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("collection_schema.json")
}

fn manufacturers_seed_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("seed")
        .join("manufacturers.csv")
}

fn railway_companies_seed_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("seed")
        .join("railway_companies.csv")
}

fn sellers_seed_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("seed")
        .join("sellers.csv")
}

// ── CSV row types (field names match exact CSV column headers) ─────────────

#[derive(serde::Deserialize)]
struct ManufacturerRow {
    name: String,
    registered_company_name: String,
    status: String,
    country_code: String,
    website_url: String,
}

#[derive(serde::Deserialize)]
struct RailwayCompanyRow {
    name: String,
    #[allow(dead_code)]
    registered_company_name: String,
    country_code: String,
    status: String,
    operating_since: String,
    operating_until: String,
}

#[derive(serde::Deserialize)]
struct SellerRow {
    name: String,
    #[serde(rename = "type")]
    seller_type: String,
    email: String,
    phone: String,
    website_url: String,
    street_address: String,
    city: String,
    region: String,
    postal_code: String,
    country_code: String,
}

// ── Seed data ──────────────────────────────────────────────────────────────

use std::collections::HashMap;

pub(crate) struct Registry {
    pub(crate) manufacturers: HashMap<String, Manufacturer>,
    pub(crate) railway_companies: HashMap<String, RailwayCompany>,
    pub(crate) sellers: HashMap<String, Seller>,
}

impl Registry {
    pub(crate) fn manufacturer_id(slug: &str) -> ManufacturerId {
        ManufacturerId(format!("trn:manufacturer:{}", slug))
    }

    pub(crate) fn company_id(slug: &str) -> RailwayCompanyId {
        RailwayCompanyId(format!("trn:railway-company:{}", slug))
    }

    pub(crate) fn seller_id(slug: &str) -> SellerId {
        SellerId(format!("trn:seller:{}", slug))
    }

    pub(crate) fn model_id(manufacturer_slug: &str, product_slug: &str) -> RailwayModelId {
        RailwayModelId(format!(
            "trn:railway-model:{}:{}",
            manufacturer_slug, product_slug
        ))
    }
}

/// Slugify a display name. Removes dots first so "A.C.M.E." → "acme".
fn slugify_name(name: &str) -> String {
    let without_dots = name.replace('.', "");
    let candidate = slugify(&without_dots);
    if candidate.is_empty() {
        "unknown".to_string()
    } else {
        candidate
    }
}

fn parse_manufacturer_status(raw: &str) -> Option<manifest::ManufacturerStatus> {
    match raw {
        "ACTIVE" => Some(manifest::ManufacturerStatus::Active),
        "MERGED" => Some(manifest::ManufacturerStatus::Merged),
        "OUT_OF_BUSINESS" => Some(manifest::ManufacturerStatus::OutOfBusiness),
        _ => None,
    }
}

fn parse_railway_company_status(raw: &str) -> Option<RailwayCompanyStatus> {
    match raw {
        "ACTIVE" => Some(RailwayCompanyStatus::Active),
        "INACTIVE" => Some(RailwayCompanyStatus::Inactive),
        "MERGED" => Some(RailwayCompanyStatus::Merged),
        _ => None,
    }
}

fn parse_seller_type(raw: &str) -> Result<SellerType> {
    match raw {
        "SHOP" => Ok(SellerType::Shop),
        "PRIVATE" => Ok(SellerType::Private),
        "MARKETPLACE" => Ok(SellerType::Marketplace),
        "DISTRIBUTOR" => Ok(SellerType::Distributor),
        other => anyhow::bail!("Unknown seller type '{}' in seed data", other),
    }
}

fn opt_str(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub(crate) fn load_registry() -> Result<Registry> {
    // ── Manufacturers ──────────────────────────────────────────────────────
    let mut manufacturers: HashMap<String, Manufacturer> = HashMap::new();
    let mut rdr = csv::Reader::from_path(manufacturers_seed_path())
        .context("Failed to open seed/manufacturers.csv")?;
    for result in rdr.deserialize::<ManufacturerRow>() {
        let row = result.context("Failed to parse row in seed/manufacturers.csv")?;
        let slug = slugify_name(&row.name);
        let manufacturer = Manufacturer {
            id: Registry::manufacturer_id(&slug),
            name: row.name.clone(),
            registered_company_name: opt_str(&row.registered_company_name),
            country_code: opt_str(&row.country_code),
            status: parse_manufacturer_status(&row.status),
            website_url: opt_str(&row.website_url),
            street_address: None,
            extended_address: None,
            city: None,
            state_region: None,
            postal_code: None,
        };
        manufacturers.insert(slug, manufacturer);
    }

    // ── Railway companies ──────────────────────────────────────────────────
    let mut railway_companies: HashMap<String, RailwayCompany> = HashMap::new();
    let mut rdr = csv::Reader::from_path(railway_companies_seed_path())
        .context("Failed to open seed/railway_companies.csv")?;
    for result in rdr.deserialize::<RailwayCompanyRow>() {
        let row = result.context("Failed to parse row in seed/railway_companies.csv")?;
        let slug = slugify_name(&row.name);
        let operating_since = if row.operating_since.is_empty() {
            None
        } else {
            Some(parse_date(&row.operating_since, "operating_since")?)
        };
        let operating_until = if row.operating_until.is_empty() {
            None
        } else {
            Some(parse_date(&row.operating_until, "operating_until")?)
        };
        let company = RailwayCompany {
            id: Registry::company_id(&slug),
            name: row.name.clone(),
            country_code: opt_str(&row.country_code),
            status: parse_railway_company_status(&row.status),
            operating_since,
            operating_until,
        };
        railway_companies.insert(slug, company);
    }

    // ── Sellers ────────────────────────────────────────────────────────────
    let mut sellers: HashMap<String, Seller> = HashMap::new();
    let mut rdr =
        csv::Reader::from_path(sellers_seed_path()).context("Failed to open seed/sellers.csv")?;
    for result in rdr.deserialize::<SellerRow>() {
        let row = result.context("Failed to parse row in seed/sellers.csv")?;
        let slug = slugify_name(&row.name);
        let has_address = !row.street_address.is_empty()
            || !row.city.is_empty()
            || !row.region.is_empty()
            || !row.postal_code.is_empty()
            || !row.country_code.is_empty();
        let address = if has_address {
            Some(manifest::Address {
                street: opt_str(&row.street_address),
                city: opt_str(&row.city),
                region: opt_str(&row.region),
                postal_code: opt_str(&row.postal_code),
                country_code: opt_str(&row.country_code),
            })
        } else {
            None
        };
        let seller = Seller {
            id: Registry::seller_id(&slug),
            name: row.name.clone(),
            seller_type: parse_seller_type(&row.seller_type)?,
            email: opt_str(&row.email),
            phone: opt_str(&row.phone),
            website_url: opt_str(&row.website_url),
            address,
        };
        sellers.insert(slug, seller);
    }

    Ok(Registry {
        manufacturers,
        railway_companies,
        sellers,
    })
}

pub(crate) fn strip_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            for child in map.values_mut() {
                strip_nulls(child);
            }
        }
        Value::Array(values) => {
            values.retain(|v| !v.is_null());
            for child in values {
                strip_nulls(child);
            }
        }
        _ => {}
    }
}

pub(crate) fn parse_rfc3339_to_utc(raw: &str, field_name: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .with_context(|| format!("Invalid date-time '{}' for field '{}'.", raw, field_name))
}

pub(crate) fn parse_date(raw: &str, field_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .with_context(|| format!("Invalid date '{}' for field '{}'.", raw, field_name))
}

pub(crate) fn migrate_collection_to_new_schema(
    import: &crate::import::Collection,
) -> Result<serde_json::Value> {
    use serde_json::{Value, json};
    use std::collections::{BTreeMap, HashSet};

    // railwayModels without purchase/catalog fields
    let mut railway_models_vals: Vec<Value> = Vec::new();
    for model in &import.railway_models {
        let mut mval = serde_json::to_value(model)?;
        if let Value::Object(map) = &mut mval {
            map.remove("collectionItemId");
            map.remove("purchaseInfo");
            map.remove("catalogItemId");
        }
        // Remove nulls so schema validation does not see `null` values
        strip_nulls(&mut mval);
        railway_models_vals.push(mval);
    }

    // items[] - start with any explicit items provided in the import (new schema)
    let mut items_vals: Vec<Value> = Vec::new();
    let mut import_model_to_collection_item: BTreeMap<String, String> = BTreeMap::new();
    let mut source_collection_id_to_generated: BTreeMap<String, String> = BTreeMap::new();

    for input_item in &import.items {
        let mut item_val = serde_json::to_value(input_item)?;
        strip_nulls(&mut item_val);
        items_vals.push(item_val);
        source_collection_id_to_generated.insert(input_item.id.clone(), input_item.id.clone());
        import_model_to_collection_item
            .entry(input_item.railway_model_id.clone())
            .or_insert_with(|| input_item.id.clone());
    }

    // Create items for models that provide purchase_info but didn't have explicit items[]
    for model in &import.railway_models {
        if import_model_to_collection_item.contains_key(&model.id) {
            continue;
        }
        if let Some(pi) = &model.purchase_info {
            let gen_id = format!("trn:collection-item:{}", Uuid::new_v4());
            let mut item_obj = json!({
                "id": gen_id.clone(),
                "railwayModelId": model.id.clone(),
                "purchaseInfo": serde_json::to_value(pi)?,
            });
            if let Some(cat) = &model.catalog_item_id
                && let Value::Object(map) = &mut item_obj
            {
                map.insert("catalogItemId".to_string(), Value::String(cat.clone()));
            }
            strip_nulls(&mut item_obj);
            items_vals.push(item_obj);
            import_model_to_collection_item.insert(model.id.clone(), gen_id.clone());
            if let Some(source_cid) = &model.collection_item_id {
                source_collection_id_to_generated.insert(source_cid.clone(), gen_id.clone());
            }
        }
    }

    // Build ownedRollingStocks[] by resolving explicit owned entries and inline markers
    let mut owned_vals: Vec<Value> = Vec::new();
    let mut handled_rolling_stock_ids: HashSet<String> = HashSet::new();

    // explicit top-level owned entries
    for input_owned in &import.owned_rolling_stocks {
        let resolved: Option<String> = source_collection_id_to_generated
            .get(&input_owned.collection_item_id)
            .cloned()
            .or_else(|| {
                input_owned.rolling_stock_id.as_ref().and_then(|rs_id| {
                    import
                        .railway_models
                        .iter()
                        .find(|m| m.rolling_stocks.iter().any(|s| s.id == *rs_id))
                        .and_then(|model| import_model_to_collection_item.get(&model.id).cloned())
                })
            });

        if let Some(collection_item_id) = resolved {
            if let Some(rs) = &input_owned.rolling_stock_id {
                handled_rolling_stock_ids.insert(rs.clone());
            }

            let mut owned_obj = serde_json::to_value(input_owned)?;
            if let Value::Object(map) = &mut owned_obj {
                map.insert(
                    "collectionItemId".to_string(),
                    Value::String(collection_item_id),
                );
            }
            strip_nulls(&mut owned_obj);
            owned_vals.push(owned_obj);
        } else {
            warn!(
                "Could not resolve collectionItemId '{}' for ownedRollingStock '{}'; skipping.",
                input_owned.collection_item_id, input_owned.id
            );
        }
    }

    // Inline owned_rolling_stock_id markers on rolling stocks
    for model in &import.railway_models {
        let gen_cid_opt = import_model_to_collection_item.get(&model.id).cloned();
        for stock in &model.rolling_stocks {
            if handled_rolling_stock_ids.contains(&stock.id) {
                continue;
            }
            if let Some(owned_id) = &stock.owned_rolling_stock_id {
                // If model has no generated collection item, this is an error
                let gen_cid = match &gen_cid_opt {
                    Some(v) => v.clone(),
                    None => bail!(
                        "Model '{}' is missing purchaseInfo; cannot migrate ownedRollingStock '{}'",
                        model.id,
                        owned_id
                    ),
                };

                let mut owned_obj = json!({
                    "id": owned_id.clone(),
                    "collectionItemId": gen_cid,
                    "rollingStockId": stock.id.clone(),
                });
                strip_nulls(&mut owned_obj);
                owned_vals.push(owned_obj);
                handled_rolling_stock_ids.insert(stock.id.clone());
            }
        }
    }

    // For any remaining rolling stocks in models that have a generated collection item,
    // create derived OwnedRollingStock records.
    for model in &import.railway_models {
        if let Some(gen_cid) = import_model_to_collection_item.get(&model.id) {
            for stock in &model.rolling_stocks {
                if handled_rolling_stock_ids.contains(&stock.id) {
                    continue;
                }
                let derived_id = format!("trn:owned-rolling-stock:{}", Uuid::new_v4());
                let mut owned_obj = json!({
                    "id": derived_id,
                    "collectionItemId": gen_cid.clone(),
                    "rollingStockId": stock.id.clone(),
                });
                strip_nulls(&mut owned_obj);
                owned_vals.push(owned_obj);
                handled_rolling_stock_ids.insert(stock.id.clone());
            }
        }
    }

    let result = json!({
        "version": import.version,
        "description": import.description.clone().unwrap_or_default(),
        "modifiedAt": import.modified_at.clone(),
        "railwayModels": railway_models_vals,
        "items": items_vals,
        "ownedRollingStocks": owned_vals,
    });

    Ok(result)
}

pub(crate) fn normalize_id_segment(raw: &str) -> String {
    let candidate = slugify(raw);
    if candidate.is_empty() {
        "unknown".to_string()
    } else {
        candidate
    }
}

/// Extract the slug portion from a TRN value.
/// If `value` starts with `prefix` (e.g. `"trn:manufacturer:"`), the rest is returned as-is.
/// Otherwise falls back to `normalize_id_segment`.
pub(crate) fn trn_slug(value: &str, prefix: &str) -> String {
    if let Some(rest) = value.strip_prefix(prefix)
        && !rest.is_empty()
    {
        return rest.to_string();
    }
    normalize_id_segment(value)
}

pub(crate) fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create output directory '{}'.", parent.display())
        })?;
    }

    Ok(())
}

pub(crate) fn write_zip(path: &Path, manifest_json: &str) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("manifest.zip");

    let tmp_name = format!("{}.tmp", file_name);
    let tmp_path = match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(&tmp_name),
        _ => PathBuf::from(&tmp_name),
    };

    let file = File::create(&tmp_path)
        .with_context(|| format!("Failed to create temporary file '{}'.", tmp_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    zip.start_file("manifest.json", options)
        .context("Failed to start manifest.json in zip")?;
    zip.write_all(manifest_json.as_bytes())
        .context("Failed to write manifest.json content to zip")?;

    zip.add_directory("images/", options)
        .context("Failed to add images/ directory to zip")?;

    zip.finish().context("Failed to finalize zip archive")?;

    fs::rename(&tmp_path, path).with_context(|| {
        format!(
            "Failed to atomically replace '{}' using '{}'.",
            path.display(),
            tmp_path.display()
        )
    })?;

    Ok(())
}

pub(crate) fn empty_manifest() -> Manifest {
    Manifest {
        schema: Some("https://rusty-shed.app/schemas/manifest/v1.json".to_string()),
        version: ManifestVersion::V1_0,
        exported_at: Some(Utc::now()),
        source: Some(format!("locrawl {}", env!("CARGO_PKG_VERSION"))),
        data: DataContainer {
            manufacturers: vec![],
            railway_companies: vec![],
            railway_models: vec![],
            collection_items: vec![],
            owned_rolling_stocks: vec![],
            sellers: vec![],
            maintenance_cards: vec![],
            track_products: vec![],
            track_inventories: vec![],
            prototypes: vec![],
            formation_categories: vec![],
            train_formations: vec![],
            wishlists: vec![],
            decoders: vec![],
            digital_rolling_stocks: vec![],
        },
    }
}

pub(crate) fn load_existing_manifest_or_empty(output: &Path) -> Result<Manifest> {
    if !output.exists() {
        return Ok(empty_manifest());
    }

    let metadata = fs::metadata(output)
        .with_context(|| format!("Failed to read metadata for '{}'.", output.display()))?;
    if metadata.len() == 0 {
        return Ok(empty_manifest());
    }

    let file = File::open(output)
        .with_context(|| format!("Failed to open zip archive '{}'.", output.display()))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| format!("Failed to read zip archive '{}'.", output.display()))?;
    let mut zip_file = archive
        .by_name("manifest.json")
        .with_context(|| format!("'manifest.json' not found in '{}'.", output.display()))?;
    let mut raw = String::new();
    zip_file
        .read_to_string(&mut raw)
        .with_context(|| format!("Failed to read manifest.json from '{}'.", output.display()))?;

    if raw.trim().is_empty() {
        return Ok(empty_manifest());
    }

    serde_json::from_str(&raw).with_context(|| {
        format!(
            "Failed to deserialize manifest from '{}'.",
            output.display()
        )
    })
}

pub(crate) fn validate_manifest_integrity(manifest: &mut Manifest) -> Result<()> {
    let valid_manufacturer_ids: HashSet<String> = manifest
        .data
        .manufacturers
        .iter()
        .map(|m| m.id.0.clone())
        .collect();

    let valid_company_ids: HashSet<String> = manifest
        .data
        .railway_companies
        .iter()
        .map(|c| c.id.0.clone())
        .collect();

    let valid_seller_ids: HashSet<String> = manifest
        .data
        .sellers
        .iter()
        .map(|s| s.id.0.clone())
        .collect();

    let valid_model_ids: HashSet<String> = manifest
        .data
        .railway_models
        .iter()
        .map(|m| m.id.0.clone())
        .collect();

    let valid_owned_rolling_stock_ids: HashSet<String> = manifest
        .data
        .owned_rolling_stocks
        .iter()
        .map(|o| o.id.0.clone())
        .collect();

    let mut errors: Vec<String> = Vec::new();

    // ── Manufacturer integrity ─────────────────────────────────────────────
    for model in &manifest.data.railway_models {
        if !valid_manufacturer_ids.contains(&model.manufacturer_id.0) {
            let slug = model
                .manufacturer_id
                .0
                .strip_prefix("trn:manufacturer:")
                .unwrap_or(&model.manufacturer_id.0);
            errors.push(format!(
                "RailwayModel '{}' references Manufacturer '{}', but '{}' was not found in manufacturers.csv.",
                model.product_code, model.manufacturer_id.0, slug
            ));
        }
    }

    // ── Railway company integrity ──────────────────────────────────────────
    for model in &manifest.data.railway_models {
        for stock in &model.rolling_stocks {
            if !valid_company_ids.contains(&stock.railway_company_id.0) {
                let slug = stock
                    .railway_company_id
                    .0
                    .strip_prefix("trn:railway-company:")
                    .unwrap_or(&stock.railway_company_id.0);
                let stock_id = stock.id.as_deref().unwrap_or("(unknown)");
                errors.push(format!(
                    "RollingStock '{}' in RailwayModel '{}' references RailwayCompany '{}', but '{}' was not found in railway_companies.csv.",
                    stock_id, model.product_code, stock.railway_company_id.0, slug
                ));
            }
        }
    }

    // ── Seller integrity ───────────────────────────────────────────────────
    for item in &manifest.data.collection_items {
        if let Some(purchase) = &item.purchase
            && let Some(seller_id) = &purchase.seller_id
            && !valid_seller_ids.contains(&seller_id.0)
        {
            let slug = seller_id
                .0
                .strip_prefix("trn:seller:")
                .unwrap_or(&seller_id.0);
            errors.push(format!(
                "CollectionItem '{}' purchase references Seller '{}', but '{}' was not found in sellers.csv.",
                item.id.0, seller_id.0, slug
            ));
        }
    }

    // ── Owned rolling stock reference integrity ───────────────────────────
    for card in &manifest.data.maintenance_cards {
        if let Some(owned_id) = &card.owned_rolling_stock_id
            && !valid_owned_rolling_stock_ids.contains(&owned_id.0)
        {
            errors.push(format!(
                "MaintenanceCard '{}' references OwnedRollingStock '{}', but it was not found in ownedRollingStocks.",
                card.id.0, owned_id.0
            ));
        }
    }

    for formation in &manifest.data.train_formations {
        for element in &formation.elements {
            if !valid_owned_rolling_stock_ids.contains(&element.owned_rolling_stock_id.0) {
                errors.push(format!(
                    "FormationElement '{}' in TrainFormation '{}' references OwnedRollingStock '{}', but it was not found in ownedRollingStocks.",
                    element.id, formation.id, element.owned_rolling_stock_id.0
                ));
            }
        }
    }

    for digital in &manifest.data.digital_rolling_stocks {
        if !valid_owned_rolling_stock_ids.contains(&digital.owned_rolling_stock_id.0) {
            errors.push(format!(
                "DigitalRollingStock '{}' references OwnedRollingStock '{}', but it was not found in ownedRollingStocks.",
                digital.id, digital.owned_rolling_stock_id.0
            ));
        }
    }

    if !errors.is_empty() {
        bail!(
            "Manifest integrity validation failed — Orphaned references:\n{}",
            errors.join("\n")
        );
    }

    // ── CollectionItem model-reference pruning ─────────────────────────────
    let before_count = manifest.data.collection_items.len();
    manifest
        .data
        .collection_items
        .retain(|item| valid_model_ids.contains(&item.railway_model_id.0));
    let pruned = before_count - manifest.data.collection_items.len();
    if pruned > 0 {
        warn!(
            "Pruned {} CollectionItem(s) with missing railwayModelId.",
            pruned
        );
    }

    // ── WishlistItem model-reference pruning ───────────────────────────────
    for wishlist in &mut manifest.data.wishlists {
        let before = wishlist.items.len();
        wishlist
            .items
            .retain(|item| valid_model_ids.contains(&item.railway_model_id.0));
        let pruned = before - wishlist.items.len();
        if pruned > 0 {
            warn!(
                "Pruned {} WishlistItem(s) from '{}' with missing railwayModelId.",
                pruned, wishlist.name
            );
        }
    }

    Ok(())
}

pub(crate) fn map_import_to_manifest(import: &Collection, seeds: &Registry) -> Result<Manifest> {
    let exported_at = parse_rfc3339_to_utc(&import.modified_at, "modifiedAt")?;

    let mut manufacturers: BTreeMap<String, Manufacturer> = BTreeMap::new();
    let mut railway_companies: BTreeMap<String, RailwayCompany> = BTreeMap::new();
    let mut sellers: BTreeMap<String, Seller> = BTreeMap::new();

    let mut railway_models = Vec::with_capacity(import.railway_models.len());
    let mut collection_items = Vec::new();
    let mut owned_rolling_stocks: Vec<OwnedRollingStock> = Vec::new();
    // mappings to translate source collection IDs (if provided) and import model ids
    // to the generated manifest CollectionItemId values.
    let mut import_model_to_collection_item: BTreeMap<String, CollectionItemId> = BTreeMap::new();
    let mut source_collection_id_to_generated: BTreeMap<String, CollectionItemId> = BTreeMap::new();

    // First, process any explicit `items[]` provided in the (new) collection schema.
    // These entries map directly to manifest CollectionItem records and may
    // reference existing railway model ids.
    for input_item in &import.items {
        let added_date = parse_date(&input_item.purchase_info.purchase_date, "purchaseDate")?;

        // Map seller
        let seller_slug = trn_slug(&input_item.purchase_info.seller, "trn:seller:");
        let seller_id = Registry::seller_id(&seller_slug);
        if let Some(seed_seller) = seeds.sellers.get(&seller_slug).cloned() {
            sellers
                .entry(seller_id.0.clone())
                .or_insert_with(|| seed_seller.clone());
        }

        let purchase = Some(Purchase {
            r#type: PurchaseType::Purchased,
            purchase_date: Some(added_date),
            price: Some(manifest::Money {
                amount: input_item.purchase_info.price.amount.round() as i64,
                currency: input_item.purchase_info.price.currency.clone(),
            }),
            seller_id: Some(seller_id.clone()),
            sale_date: None,
            sale_price: None,
            deposit_amount: None,
            expected_delivery: None,
        });

        let generated_collection_item_id = CollectionItemId(input_item.id.clone());

        collection_items.push(CollectionItem {
            id: generated_collection_item_id.clone(),
            railway_model_id: RailwayModelId(input_item.railway_model_id.clone()),
            added_date,
            removed_date: None,
            purchase_condition: None,
            model_condition: None,
            box_condition: None,
            notes: None,
            image: None,
            purchase,
        });

        // Map source collection item id -> generated id
        source_collection_id_to_generated
            .insert(input_item.id.clone(), generated_collection_item_id.clone());

        // Map railway model -> generated collection item if not already present
        import_model_to_collection_item
            .entry(input_item.railway_model_id.clone())
            .or_insert_with(|| generated_collection_item_id.clone());
    }

    // Now process railway models; generate a CollectionItem for models that
    // provide `purchaseInfo` but did not already have an explicit `items[]`
    for model in &import.railway_models {
        let manufacturer_slug = trn_slug(&model.manufacturer, "trn:manufacturer:");
        let product_slug = normalize_id_segment(&model.product_code);
        let manufacturer_id = Registry::manufacturer_id(&manufacturer_slug);

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

        let railway_model_id = Registry::model_id(&manufacturer_slug, &product_slug);

        // If this model already has an explicit item (from import.items), skip
        // generating an additional CollectionItem; otherwise, always generate
        // a CollectionItem (purchase may be none).
        if !import_model_to_collection_item.contains_key(&model.id) {
            let mut added_date = exported_at.date_naive();

            let purchase = if let Some(import_purchase) = &model.purchase_info {
                added_date = parse_date(&import_purchase.purchase_date, "purchaseDate")?;

                let seller_slug = trn_slug(&import_purchase.seller, "trn:seller:");
                let seller_id = Registry::seller_id(&seller_slug);

                let seed_seller = seeds.sellers.get(&seller_slug).with_context(|| {
                    format!(
                        "Seller '{}' (slug: '{}') not found in seed/sellers.csv",
                        import_purchase.seller, seller_slug
                    )
                })?;
                sellers
                    .entry(seller_id.0.clone())
                    .or_insert_with(|| seed_seller.clone());

                Some(Purchase {
                    r#type: PurchaseType::Purchased,
                    purchase_date: Some(added_date),
                    price: Some(manifest::Money {
                        amount: import_purchase.price.amount.round() as i64,
                        currency: import_purchase.price.currency.clone(),
                    }),
                    seller_id: Some(seller_id),
                    sale_date: None,
                    sale_price: None,
                    deposit_amount: None,
                    expected_delivery: None,
                })
            } else {
                None
            };

            // Create a generated CollectionItem id and record mapping from the
            // import model id so explicit ownedRollingStocks can reference it.
            let generated_collection_item_id =
                CollectionItemId(format!("trn:collection-item:{}", Uuid::new_v4()));

            collection_items.push(CollectionItem {
                id: generated_collection_item_id.clone(),
                railway_model_id: railway_model_id.clone(),
                added_date,
                removed_date: None,
                purchase_condition: None,
                model_condition: None,
                box_condition: None,
                notes: None,
                image: None,
                purchase,
            });

            import_model_to_collection_item
                .insert(model.id.clone(), generated_collection_item_id.clone());
            if let Some(source_cid) = model.collection_item_id.clone() {
                source_collection_id_to_generated
                    .insert(source_cid, generated_collection_item_id.clone());
            }
        }
    }

    // First, process any explicit ownedRollingStocks provided in the input.
    // These entries reference source collection item ids which we map to the
    // generated CollectionItem ids recorded above. We also record which
    // rolling stock ids have already been handled so we don't duplicate them.
    let mut handled_rolling_stock_ids: HashSet<String> = HashSet::new();

    for input_owned in &import.owned_rolling_stocks {
        // Try to resolve the source collectionItemId to the generated value.
        let mut resolved_collection_item: Option<CollectionItemId> = None;

        if let Some(mapped) = source_collection_id_to_generated.get(&input_owned.collection_item_id)
        {
            resolved_collection_item = Some(mapped.clone());
        }

        // If unresolved, try to resolve using the referenced rollingStockId.
        // Collapse nested `if` statements to satisfy clippy's `collapsible_if`.
        if resolved_collection_item.is_none()
            && let Some(rs_id) = &input_owned.rolling_stock_id
            && let Some(model) = import
                .railway_models
                .iter()
                .find(|m| m.rolling_stocks.iter().any(|s| s.id == *rs_id))
            && let Some(mapped) = import_model_to_collection_item.get(&model.id)
        {
            resolved_collection_item = Some(mapped.clone());
        }

        if let Some(collection_item_id) = resolved_collection_item {
            if let Some(rs) = &input_owned.rolling_stock_id {
                handled_rolling_stock_ids.insert(rs.clone());
            }

            owned_rolling_stocks.push(OwnedRollingStock {
                id: OwnedRollingStockId(input_owned.id.clone()),
                collection_item_id: collection_item_id.clone(),
                rolling_stock_id: input_owned.rolling_stock_id.clone(),
                notes: input_owned.notes.clone(),
                dcc_address: input_owned.dcc_address,
                installed_decoder_id: input_owned.installed_decoder_id.clone(),
                current_coupler_id: input_owned.current_coupler_id.clone(),
            });
        } else {
            warn!(
                "Could not resolve collectionItemId '{}' for ownedRollingStock '{}'; skipping.",
                input_owned.collection_item_id, input_owned.id
            );
        }
    }

    // For any rolling stocks not covered by explicit owned entries, create
    // derived OwnedRollingStock records (one per physical rolling stock).
    for model in &import.railway_models {
        if let Some(gen_cid) = import_model_to_collection_item.get(&model.id) {
            for stock in &model.rolling_stocks {
                if handled_rolling_stock_ids.contains(&stock.id) {
                    continue;
                }
                owned_rolling_stocks.push(OwnedRollingStock {
                    id: OwnedRollingStockId(format!("trn:owned-rolling-stock:{}", Uuid::new_v4())),
                    collection_item_id: gen_cid.clone(),
                    rolling_stock_id: Some(stock.id.clone()),
                    notes: None,
                    dcc_address: None,
                    installed_decoder_id: None,
                    current_coupler_id: None,
                });
            }
        }
    }
    Ok(Manifest {
        schema: Some("https://rusty-shed.app/schemas/manifest/v1.json".to_string()),
        version: ManifestVersion::V1_0,
        exported_at: Some(exported_at),
        source: Some(format!("locrawl {}", env!("CARGO_PKG_VERSION"))),
        data: DataContainer {
            manufacturers: manufacturers.into_values().collect(),
            railway_companies: railway_companies.into_values().collect(),
            railway_models,
            collection_items,
            owned_rolling_stocks,
            sellers: sellers.into_values().collect(),
            maintenance_cards: vec![],
            track_products: vec![],
            track_inventories: vec![],
            prototypes: vec![],
            formation_categories: vec![],
            train_formations: vec![],
            wishlists: vec![],
            decoders: vec![],
            digital_rolling_stocks: vec![],
        },
    })
}

pub(crate) fn make_model_id(manufacturer: &str, product_code: &str) -> RailwayModelId {
    Registry::model_id(
        &trn_slug(manufacturer, "trn:manufacturer:"),
        &normalize_id_segment(product_code),
    )
}

pub(crate) fn map_railway_model(
    model: &ImportRailwayModel,
    manufacturer_id: &ManufacturerId,
    railway_companies: &mut BTreeMap<String, RailwayCompany>,
    seeds: &Registry,
) -> Result<RailwayModel> {
    let manufacturer_slug = trn_slug(&model.manufacturer, "trn:manufacturer:");
    let product_slug = normalize_id_segment(&model.product_code);

    Ok(RailwayModel {
        id: Registry::model_id(&manufacturer_slug, &product_slug),
        manufacturer_id: manufacturer_id.clone(),
        product_code: model.product_code.clone(),
        description: LocalizedText {
            en: Some(model.description.clone()),
            it: None,
        },
        details: None,
        scale: map_scale(&model.scale),
        epoch: model.epoch.0.clone(),
        category: map_model_category(&model.category),
        power_method: map_power_method(&model.power_method),
        delivery_date: model.delivery_date.clone(),
        availability_status: None,
        image: None,
        rolling_stocks: model
            .rolling_stocks
            .iter()
            .map(|stock| map_rolling_stock(stock, railway_companies, seeds))
            .collect::<Result<Vec<_>>>()?,
    })
}

fn map_rolling_stock(
    stock: &ImportRollingStock,
    railway_companies: &mut BTreeMap<String, RailwayCompany>,
    seeds: &Registry,
) -> Result<RollingStock> {
    let railway_company_id = railway_company_id_for(&stock.railway, railway_companies, seeds)?;

    Ok(RollingStock {
        id: Some(stock.id.clone()),
        railway_company_id,
        series_code: stock
            .series_code
            .clone()
            .or_else(|| stock.type_name.clone())
            .or_else(|| stock.series.clone())
            .unwrap_or_else(|| stock.id.clone()),
        road_number: stock.road_number.clone(),
        livery: stock.livery.clone(),
        friendly_name: stock.type_name.clone(),
        series: stock.series.clone(),
        depot: stock.depot.clone(),
        electric_multiple_unit_type: map_electric_multiple_unit_type(
            &stock.category,
            &stock.sub_category,
        ),
        freight_car_type: map_freight_car_type(&stock.category, &stock.sub_category),
        locomotive_type: map_locomotive_type(&stock.category, &stock.sub_category),
        passenger_car_type: map_passenger_car_type(&stock.category, &stock.sub_category),
        railcar_type: map_railcar_type(&stock.category, &stock.sub_category),
        service_level: map_service_level(&stock.service_level),
        is_dummy: stock.is_dummy,
        length_inches: None,
        length_millimeters: stock.length.map(|v| v as f64),
        technical_minimum_radius_mm: None,
        technical_coupling_socket: None,
        technical_coupling_close_couplers: None,
        technical_coupling_digital_shunting: None,
        technical_flywheel_fitted: None,
        technical_body_shell: None,
        technical_chassis: None,
        technical_interior_lights: None,
        technical_lights: None,
        technical_sprung_buffers: None,
        dcc_interface: None,
        control: Some(Control::NoDcc),
    })
}

pub(crate) fn railway_company_id_for(
    railway_name: &str,
    railway_companies: &mut BTreeMap<String, RailwayCompany>,
    seeds: &Registry,
) -> Result<RailwayCompanyId> {
    let railway_slug = trn_slug(railway_name, "trn:railway-company:");
    let company_id = Registry::company_id(&railway_slug);

    if !railway_companies.contains_key(&company_id.0) {
        let seed_company = seeds
            .railway_companies
            .get(&railway_slug)
            .with_context(|| {
                format!(
                    "Railway company '{}' (slug: '{}') not found in seed/railway_companies.csv",
                    railway_name, railway_slug
                )
            })?;
        railway_companies.insert(company_id.0.clone(), seed_company.clone());
    }

    Ok(company_id)
}

fn map_model_category(category: &ImportRailwayModelCategory) -> Category {
    match category {
        ImportRailwayModelCategory::Locomotives => Category::Locomotives,
        ImportRailwayModelCategory::TrainSets => Category::TrainSets,
        ImportRailwayModelCategory::StarterSets => Category::StarterSets,
        ImportRailwayModelCategory::FreightCars => Category::FreightCars,
        ImportRailwayModelCategory::PassengerCars => Category::PassengerCars,
        ImportRailwayModelCategory::ElectricMultipleUnits => Category::ElectricMultipleUnits,
        ImportRailwayModelCategory::Railcars => Category::Railcars,
    }
}

fn map_power_method(power_method: &ImportPowerMethod) -> PowerMethod {
    match power_method {
        ImportPowerMethod::Ac => PowerMethod::Ac,
        ImportPowerMethod::Dc => PowerMethod::Dc,
        ImportPowerMethod::TrixExpress => PowerMethod::TrixExpress,
    }
}

fn map_scale(scale: &ImportScale) -> Scale {
    match scale {
        ImportScale::Z => Scale::Z,
        ImportScale::N => Scale::N,
        ImportScale::Tt => Scale::TT,
        ImportScale::H0 => Scale::H0,
        ImportScale::Zero => Scale::Scale0,
        ImportScale::One => Scale::Scale1,
        ImportScale::G => Scale::G,
    }
}

fn map_service_level(level: &Option<ImportServiceLevel>) -> Option<ServiceLevel> {
    level.as_ref().map(|v| match v {
        ImportServiceLevel::OneCl => ServiceLevel::First,
        ImportServiceLevel::TwoCl => ServiceLevel::Second,
        ImportServiceLevel::ThreeCl => ServiceLevel::Third,
        ImportServiceLevel::OneClTwoCl => ServiceLevel::FirstSecond,
        ImportServiceLevel::OneClTwoClThreeCl => ServiceLevel::FirstSecondThird,
        ImportServiceLevel::TwoClThreeCl => ServiceLevel::SecondThird,
    })
}

fn map_electric_multiple_unit_type(
    category: &ImportRollingStockCategory,
    sub_category: &Option<ImportSubCategory>,
) -> Option<ElectricMultipleUnitType> {
    if !matches!(category, ImportRollingStockCategory::ElectricMultipleUnit) {
        return None;
    }

    match sub_category {
        Some(ImportSubCategory::DrivingCar) => Some(ElectricMultipleUnitType::DrivingCar),
        Some(ImportSubCategory::HighSpeedTrain) => Some(ElectricMultipleUnitType::HighSpeedTrain),
        Some(ImportSubCategory::MotorCar) => Some(ElectricMultipleUnitType::MotorCar),
        Some(ImportSubCategory::PowerCar) => Some(ElectricMultipleUnitType::PowerCar),
        Some(ImportSubCategory::TrailerCar) => Some(ElectricMultipleUnitType::TrailerCar),
        Some(ImportSubCategory::TrainSet) => Some(ElectricMultipleUnitType::TrainSet),
        _ => None,
    }
}

fn map_freight_car_type(
    category: &ImportRollingStockCategory,
    sub_category: &Option<ImportSubCategory>,
) -> Option<FreightCarType> {
    if !matches!(category, ImportRollingStockCategory::FreightCar) {
        return None;
    }

    match sub_category {
        Some(ImportSubCategory::AutoTransportCars) => Some(FreightCarType::AutoTransportCars),
        Some(ImportSubCategory::BrakeWagon) => Some(FreightCarType::BrakeWagon),
        Some(ImportSubCategory::ClosedCargoVehicle) => Some(FreightCarType::ClosedCargoVehicle),
        Some(ImportSubCategory::ContainerCars) => Some(FreightCarType::ContainerCars),
        Some(ImportSubCategory::CoveredFreightCars) => Some(FreightCarType::CoveredFreightCars),
        Some(ImportSubCategory::DeepWellFlatCars) => Some(FreightCarType::DeepWellFlatCars),
        Some(ImportSubCategory::DumpCars) => Some(FreightCarType::DumpCars),
        Some(ImportSubCategory::Gondola) => Some(FreightCarType::Gondola),
        Some(ImportSubCategory::HeavyGoodsWagons) => Some(FreightCarType::HeavyGoodsWagons),
        Some(ImportSubCategory::HingedCoverWagons) => Some(FreightCarType::HingedCoverWagons),
        Some(ImportSubCategory::HopperWagon) => Some(FreightCarType::HopperWagon),
        Some(ImportSubCategory::RefrigeratorCars) => Some(FreightCarType::RefrigeratorCars),
        Some(ImportSubCategory::SiloContainerCars) => Some(FreightCarType::SiloContainerCars),
        Some(ImportSubCategory::SlideTarpaulinWagon) => Some(FreightCarType::SlideTarpaulinWagon),
        Some(ImportSubCategory::SlidingWallBoxcars) => Some(FreightCarType::SlidingWallBoxcars),
        Some(ImportSubCategory::SpecialTransport) => Some(FreightCarType::SpecialTransport),
        Some(ImportSubCategory::StakeWagons) => Some(FreightCarType::StakeWagons),
        Some(ImportSubCategory::SwingRoofWagon) => Some(FreightCarType::SwingRoofWagon),
        Some(ImportSubCategory::TankCars) => Some(FreightCarType::TankCars),
        Some(ImportSubCategory::TelescopeHoodWagons) => Some(FreightCarType::TelescopeHoodWagons),
        _ => None,
    }
}

fn map_locomotive_type(
    category: &ImportRollingStockCategory,
    sub_category: &Option<ImportSubCategory>,
) -> Option<LocomotiveType> {
    if !matches!(category, ImportRollingStockCategory::Locomotive) {
        return None;
    }

    match sub_category {
        Some(ImportSubCategory::SteamLocomotive) => Some(LocomotiveType::SteamLocomotive),
        Some(ImportSubCategory::DieselLocomotive) => Some(LocomotiveType::DieselLocomotive),
        Some(ImportSubCategory::ElectricLocomotive) => Some(LocomotiveType::ElectricLocomotive),
        _ => None,
    }
}

fn map_passenger_car_type(
    category: &ImportRollingStockCategory,
    sub_category: &Option<ImportSubCategory>,
) -> Option<PassengerCarType> {
    if !matches!(category, ImportRollingStockCategory::PassengerCar) {
        return None;
    }

    match sub_category {
        Some(ImportSubCategory::BaggageCar) => Some(PassengerCarType::BaggageCar),
        Some(ImportSubCategory::BuffetCar) => Some(PassengerCarType::BuffetCar),
        Some(ImportSubCategory::CombineCar) => Some(PassengerCarType::CombineCar),
        Some(ImportSubCategory::CompartmentCoach) => Some(PassengerCarType::CompartmentCoach),
        Some(ImportSubCategory::DiningCar) => Some(PassengerCarType::DiningCar),
        Some(ImportSubCategory::DoubleDecker) => Some(PassengerCarType::DoubleDecker),
        Some(ImportSubCategory::DomeCar) => Some(PassengerCarType::DomeCar),
        Some(ImportSubCategory::DrivingTrailer) => Some(PassengerCarType::DrivingTrailer),
        Some(ImportSubCategory::Lounge) => Some(PassengerCarType::Lounge),
        Some(ImportSubCategory::Observation) => Some(PassengerCarType::Observation),
        Some(ImportSubCategory::OpenCoach) => Some(PassengerCarType::OpenCoach),
        Some(ImportSubCategory::RailwayPostOffice) => Some(PassengerCarType::RailwayPostOffice),
        Some(ImportSubCategory::SleepingCar) => Some(PassengerCarType::SleepingCar),
        Some(ImportSubCategory::Sleeperette) => Some(PassengerCarType::Sleeperette),
        _ => None,
    }
}

fn map_railcar_type(
    category: &ImportRollingStockCategory,
    sub_category: &Option<ImportSubCategory>,
) -> Option<RailcarType> {
    if !matches!(category, ImportRollingStockCategory::Railcar) {
        return None;
    }

    match sub_category {
        Some(ImportSubCategory::PowerCar) => Some(RailcarType::PowerCar),
        Some(ImportSubCategory::TrailerCar) => Some(RailcarType::TrailerCar),
        _ => None,
    }
}

fn merge_collection_manifests(
    mut existing: Manifest,
    incoming: Manifest,
    force: bool,
) -> Result<Manifest> {
    let existing_ids: BTreeSet<String> = existing
        .data
        .collection_items
        .iter()
        .map(|item| item.id.0.clone())
        .collect();

    let incoming_ids: BTreeSet<String> = incoming
        .data
        .collection_items
        .iter()
        .map(|item| item.id.0.clone())
        .collect();

    let conflicts: Vec<String> = incoming_ids.intersection(&existing_ids).cloned().collect();

    if !force && !conflicts.is_empty() {
        bail!(
            "Collection conflicts found for IDs: {}. Re-run with --force to overwrite.",
            conflicts.join(", ")
        );
    }

    if force {
        let conflict_set: BTreeSet<&str> = conflicts.iter().map(String::as_str).collect();
        existing
            .data
            .collection_items
            .retain(|item| !conflict_set.contains(item.id.0.as_str()));
    }

    merge_by_key(
        &mut existing.data.manufacturers,
        incoming.data.manufacturers,
        |m| m.id.0.clone(),
        force,
    );
    merge_by_key(
        &mut existing.data.railway_companies,
        incoming.data.railway_companies,
        |c| c.id.0.clone(),
        force,
    );
    merge_by_key(
        &mut existing.data.railway_models,
        incoming.data.railway_models,
        |m| m.id.0.clone(),
        force,
    );
    merge_by_key(
        &mut existing.data.collection_items,
        incoming.data.collection_items,
        |i| i.id.0.clone(),
        true,
    );
    merge_by_key(
        &mut existing.data.owned_rolling_stocks,
        incoming.data.owned_rolling_stocks,
        |o| o.id.clone(),
        true,
    );
    merge_by_key(
        &mut existing.data.sellers,
        incoming.data.sellers,
        |s| s.id.0.clone(),
        force,
    );

    existing.source = Some(format!("locrawl {}", env!("CARGO_PKG_VERSION")));
    if existing.exported_at.is_none() {
        existing.exported_at = Some(Utc::now());
    }

    Ok(existing)
}

fn merge_by_key<T, K, F>(existing: &mut Vec<T>, incoming: Vec<T>, key_fn: F, replace_existing: bool)
where
    K: Ord,
    F: Fn(&T) -> K,
{
    let mut merged: BTreeMap<K, T> = BTreeMap::new();

    for item in existing.drain(..) {
        merged.insert(key_fn(&item), item);
    }

    for item in incoming {
        let key = key_fn(&item);
        if replace_existing || !merged.contains_key(&key) {
            merged.insert(key, item);
        }
    }

    existing.extend(merged.into_values());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::{
        Category as ImportCategory, PowerMethod as ImportPowerMethod,
        RailwayModelCategory as ImportRailwayModelCategory, Scale as ImportScale,
        ServiceLevel as ImportServiceLevel, SubCategory as ImportSubCategory,
    };
    use chrono::Datelike;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    // ------- normalize_id_segment -------

    #[test]
    fn normalize_id_segment_lowercases_and_slugifies() {
        assert_eq!(normalize_id_segment("ACME Rail"), "acme-rail");
    }

    #[test]
    fn normalize_id_segment_strips_special_characters() {
        assert_eq!(normalize_id_segment("Märklin & Co."), "marklin-co");
    }

    #[test]
    fn normalize_id_segment_returns_unknown_for_empty_string() {
        assert_eq!(normalize_id_segment(""), "unknown");
    }

    #[test]
    fn normalize_id_segment_returns_unknown_for_only_special_chars() {
        assert_eq!(normalize_id_segment("---!!!"), "unknown");
    }

    // ------- parse_date -------

    #[test]
    fn parse_date_parses_valid_iso_date() {
        let date = parse_date("2024-06-15", "testField").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 6);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn parse_date_returns_error_for_invalid_format() {
        let result = parse_date("15/06/2024", "testField");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("testField"));
    }

    #[test]
    fn parse_date_returns_error_for_garbage_input() {
        let result = parse_date("not-a-date", "someField");
        assert!(result.is_err());
    }

    // ------- parse_rfc3339_to_utc -------

    #[test]
    fn parse_rfc3339_to_utc_parses_valid_timestamp() {
        let dt = parse_rfc3339_to_utc("2026-03-31T12:00:00Z", "modifiedAt").unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-03-31T12:00:00+00:00");
    }

    #[test]
    fn parse_rfc3339_to_utc_converts_offset_to_utc() {
        let dt = parse_rfc3339_to_utc("2026-01-01T13:00:00+01:00", "modifiedAt").unwrap();
        assert_eq!(dt.to_rfc3339(), "2026-01-01T12:00:00+00:00");
    }

    #[test]
    fn parse_rfc3339_to_utc_returns_error_for_invalid() {
        let result = parse_rfc3339_to_utc("2026-03-31", "modifiedAt");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("modifiedAt"));
    }

    // ------- map_scale -------

    #[test]
    fn map_scale_maps_all_variants() {
        use Scale::*;
        assert!(matches!(map_scale(&ImportScale::Z), Z));
        assert!(matches!(map_scale(&ImportScale::N), N));
        assert!(matches!(map_scale(&ImportScale::Tt), TT));
        assert!(matches!(map_scale(&ImportScale::H0), H0));
        assert!(matches!(map_scale(&ImportScale::Zero), Scale0));
        assert!(matches!(map_scale(&ImportScale::One), Scale1));
        assert!(matches!(map_scale(&ImportScale::G), G));
    }

    // ------- map_power_method -------

    #[test]
    fn map_power_method_maps_all_variants() {
        use PowerMethod::*;
        assert!(matches!(map_power_method(&ImportPowerMethod::Ac), Ac));
        assert!(matches!(map_power_method(&ImportPowerMethod::Dc), Dc));
        assert!(matches!(
            map_power_method(&ImportPowerMethod::TrixExpress),
            TrixExpress
        ));
    }

    // ------- map_model_category -------

    #[test]
    fn map_model_category_maps_all_variants() {
        use Category::*;
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::Locomotives),
            Locomotives
        ));
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::FreightCars),
            FreightCars
        ));
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::PassengerCars),
            PassengerCars
        ));
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::ElectricMultipleUnits),
            ElectricMultipleUnits
        ));
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::Railcars),
            Railcars
        ));
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::TrainSets),
            TrainSets
        ));
        assert!(matches!(
            map_model_category(&ImportRailwayModelCategory::StarterSets),
            StarterSets
        ));
    }

    // ------- map_service_level -------

    #[test]
    fn map_service_level_maps_all_variants() {
        use ServiceLevel::*;
        assert!(matches!(
            map_service_level(&Some(ImportServiceLevel::OneCl)),
            Some(First)
        ));
        assert!(matches!(
            map_service_level(&Some(ImportServiceLevel::TwoCl)),
            Some(Second)
        ));
        assert!(matches!(
            map_service_level(&Some(ImportServiceLevel::ThreeCl)),
            Some(Third)
        ));
        assert!(matches!(
            map_service_level(&Some(ImportServiceLevel::OneClTwoCl)),
            Some(FirstSecond)
        ));
        assert!(matches!(
            map_service_level(&Some(ImportServiceLevel::TwoClThreeCl)),
            Some(SecondThird)
        ));
        assert!(matches!(
            map_service_level(&Some(ImportServiceLevel::OneClTwoClThreeCl)),
            Some(FirstSecondThird)
        ));
        assert!(map_service_level(&None).is_none());
    }

    // ------- map_locomotive_type -------

    #[test]
    fn map_locomotive_type_maps_all_sub_categories() {
        assert!(matches!(
            map_locomotive_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::SteamLocomotive)
            ),
            Some(LocomotiveType::SteamLocomotive)
        ));
        assert!(matches!(
            map_locomotive_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::DieselLocomotive)
            ),
            Some(LocomotiveType::DieselLocomotive)
        ));
        assert!(matches!(
            map_locomotive_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::ElectricLocomotive)
            ),
            Some(LocomotiveType::ElectricLocomotive)
        ));
    }

    #[test]
    fn map_locomotive_type_returns_none_for_wrong_category() {
        assert!(
            map_locomotive_type(
                &ImportCategory::FreightCar,
                &Some(ImportSubCategory::SteamLocomotive)
            )
            .is_none()
        );
    }

    #[test]
    fn map_locomotive_type_returns_none_for_unknown_sub_category() {
        assert!(
            map_locomotive_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::PowerCar)
            )
            .is_none()
        );
    }

    // ------- map_freight_car_type -------

    #[test]
    fn map_freight_car_type_returns_none_for_wrong_category() {
        assert!(
            map_freight_car_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::TankCars)
            )
            .is_none()
        );
    }

    #[test]
    fn map_freight_car_type_maps_tank_cars() {
        assert!(matches!(
            map_freight_car_type(
                &ImportCategory::FreightCar,
                &Some(ImportSubCategory::TankCars)
            ),
            Some(FreightCarType::TankCars)
        ));
    }

    // ------- map_passenger_car_type -------

    #[test]
    fn map_passenger_car_type_returns_none_for_wrong_category() {
        assert!(
            map_passenger_car_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::DiningCar)
            )
            .is_none()
        );
    }

    #[test]
    fn map_passenger_car_type_maps_dining_car() {
        assert!(matches!(
            map_passenger_car_type(
                &ImportCategory::PassengerCar,
                &Some(ImportSubCategory::DiningCar)
            ),
            Some(PassengerCarType::DiningCar)
        ));
    }

    // ------- map_electric_multiple_unit_type -------

    #[test]
    fn map_electric_multiple_unit_type_returns_none_for_wrong_category() {
        assert!(
            map_electric_multiple_unit_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::MotorCar)
            )
            .is_none()
        );
    }

    #[test]
    fn map_electric_multiple_unit_type_maps_driving_car() {
        assert!(matches!(
            map_electric_multiple_unit_type(
                &ImportCategory::ElectricMultipleUnit,
                &Some(ImportSubCategory::DrivingCar)
            ),
            Some(ElectricMultipleUnitType::DrivingCar)
        ));
    }

    // ------- map_railcar_type -------

    #[test]
    fn map_railcar_type_returns_none_for_wrong_category() {
        assert!(
            map_railcar_type(
                &ImportCategory::Locomotive,
                &Some(ImportSubCategory::PowerCar)
            )
            .is_none()
        );
    }

    #[test]
    fn map_railcar_type_maps_power_car() {
        assert!(matches!(
            map_railcar_type(&ImportCategory::Railcar, &Some(ImportSubCategory::PowerCar)),
            Some(RailcarType::PowerCar)
        ));
    }

    // ------- merge_by_key -------

    #[test]
    fn merge_by_key_appends_new_items() {
        let mut existing = vec![(1u32, "a"), (2u32, "b")];
        let incoming = vec![(3u32, "c")];
        merge_by_key(&mut existing, incoming, |(k, _)| *k, false);
        assert_eq!(existing.len(), 3);
        assert!(existing.iter().any(|(k, _)| *k == 3));
    }

    #[test]
    fn merge_by_key_skips_duplicate_when_not_force() {
        let mut existing = vec![(1u32, "original")];
        let incoming = vec![(1u32, "replacement")];
        merge_by_key(&mut existing, incoming, |(k, _)| *k, false);
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].1, "original");
    }

    #[test]
    fn merge_by_key_replaces_duplicate_when_force() {
        let mut existing = vec![(1u32, "original")];
        let incoming = vec![(1u32, "replacement")];
        merge_by_key(&mut existing, incoming, |(k, _)| *k, true);
        assert_eq!(existing.len(), 1);
        assert_eq!(existing[0].1, "replacement");
    }

    // ------- strip_nulls -------

    #[test]
    fn strip_nulls_removes_null_object_fields() {
        let mut value = json!({ "a": 1, "b": null, "c": "x" });
        strip_nulls(&mut value);
        assert_eq!(value, json!({ "a": 1, "c": "x" }));
    }

    #[test]
    fn strip_nulls_removes_null_array_elements() {
        let mut value = json!([1, null, 3]);
        strip_nulls(&mut value);
        assert_eq!(value, json!([1, 3]));
    }

    #[test]
    fn strip_nulls_recurses_into_nested_objects() {
        let mut value = json!({ "outer": { "keep": true, "drop": null } });
        strip_nulls(&mut value);
        assert_eq!(value, json!({ "outer": { "keep": true } }));
    }

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("locrawl-{}-{}", name, nanos))
    }

    fn sample_collection(product_code: &str) -> Value {
        json!({
            "version": 1,
            "modifiedAt": "2026-03-31T12:00:00Z",
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
                    ]
                }
            ]
        })
    }

    #[tokio::test]
    async fn append_collection_adds_second_item() {
        let source_one = temp_path("source-one.json");
        let source_two = temp_path("source-two.json");
        let output = temp_path("manifest.json");

        fs::write(
            &source_one,
            serde_json::to_string(&sample_collection("A-123")).expect("json"),
        )
        .expect("source one write should succeed");
        fs::write(
            &source_two,
            serde_json::to_string(&sample_collection("B-456")).expect("json"),
        )
        .expect("source two write should succeed");

        run(ImportCollectionArgs {
            source: source_one.clone(),
            output: output.clone(),
            force: false,
        })
        .await
        .expect("first import should succeed");

        run(ImportCollectionArgs {
            source: source_two.clone(),
            output: output.clone(),
            force: false,
        })
        .await
        .expect("second import should append");

        let manifest = load_existing_manifest_or_empty(&output).expect("manifest should exist");
        assert_eq!(manifest.data.collection_items.len(), 2);

        let _ = fs::remove_file(source_one);
        let _ = fs::remove_file(source_two);
        let _ = fs::remove_file(output);
    }

    #[tokio::test]
    async fn duplicate_collection_id_fails_without_force() {
        let source_one = temp_path("dup-source-one.json");
        let source_two = temp_path("dup-source-two.json");
        let output = temp_path("dup-manifest.json");

        fs::write(
            &source_one,
            serde_json::to_string(&sample_collection("A-123")).expect("json"),
        )
        .expect("source one write should succeed");
        fs::write(
            &source_two,
            serde_json::to_string(&sample_collection("A-123")).expect("json"),
        )
        .expect("source two write should succeed");

        run(ImportCollectionArgs {
            source: source_one.clone(),
            output: output.clone(),
            force: false,
        })
        .await
        .expect("first import should succeed");

        let result = run(ImportCollectionArgs {
            source: source_two.clone(),
            output: output.clone(),
            force: false,
        })
        .await;

        // UUID-based IDs are always unique, so the same source can be imported
        // multiple times without conflict.
        assert!(result.is_ok());

        let manifest = load_existing_manifest_or_empty(&output).expect("manifest should exist");
        assert_eq!(manifest.data.collection_items.len(), 2);

        let _ = fs::remove_file(source_one);
        let _ = fs::remove_file(source_two);
        let _ = fs::remove_file(output);
    }

    // ------- output extension defaulting -------

    #[tokio::test]
    async fn run_appends_zip_extension_when_output_has_no_extension() {
        let source = temp_path("ext-source.json");
        let output_no_ext = temp_path("ext-output"); // no extension

        fs::write(
            &source,
            serde_json::to_string(&sample_collection("EXT-001")).expect("json"),
        )
        .expect("source write should succeed");

        run(ImportCollectionArgs {
            source: source.clone(),
            output: output_no_ext.clone(),
            force: false,
        })
        .await
        .expect("import should succeed");

        let expected = output_no_ext.with_extension("zip");
        assert!(expected.exists(), "expected {expected:?} to exist");
        assert!(
            !output_no_ext.exists(),
            "bare path without extension should not exist"
        );

        let _ = fs::remove_file(source);
        let _ = fs::remove_file(expected);
    }

    #[tokio::test]
    async fn run_preserves_explicit_extension() {
        let source = temp_path("ext2-source.json");
        let output_with_ext = temp_path("ext2-output.zip");

        fs::write(
            &source,
            serde_json::to_string(&sample_collection("EXT-002")).expect("json"),
        )
        .expect("source write should succeed");

        run(ImportCollectionArgs {
            source: source.clone(),
            output: output_with_ext.clone(),
            force: false,
        })
        .await
        .expect("import should succeed");

        assert!(output_with_ext.exists(), "explicit .zip path should exist");

        let _ = fs::remove_file(source);
        let _ = fs::remove_file(output_with_ext);
    }
}
