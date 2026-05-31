use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Args;
use log::info;
use serde_json::Value;
use uuid::Uuid;

use crate::commands::import_collection::{
    ensure_parent_dir, load_existing_manifest_or_empty, normalize_id_segment, strip_nulls,
    write_zip,
};
use crate::commands::validation::{manifest_schema_path, validate_value_with_schema};
use crate::import::DigitalRosterImport;
use crate::manifest::{
    CollectionItemId, Control, DccInterface, DecoderType, DigitalRollingStock, Manifest,
    OwnedRollingStock, OwnedRollingStockId,
};

#[derive(Debug, Args, Clone)]
pub struct ImportDigitalRosterArgs {
    /// Path to source digital roster JSON
    #[arg(short = 's', long = "source")]
    pub source: PathBuf,

    /// Path to zip archive to create or update
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,

    /// Overwrite conflicting existing digital assignments
    #[arg(short = 'f', long = "force")]
    pub force: bool,
}

pub async fn run(args: ImportDigitalRosterArgs) -> Result<()> {
    let digital_schema_path = digital_roster_schema_path();
    let manifest_schema_path = manifest_schema_path();

    let source_content = fs::read_to_string(&args.source)
        .with_context(|| format!("Failed to read source file '{}'.", args.source.display()))?;
    let source_json: Value = serde_json::from_str(&source_content)
        .with_context(|| format!("Failed to parse JSON from '{}'.", args.source.display()))?;

    validate_value_with_schema(
        &source_json,
        &digital_schema_path,
        "digital roster import input",
    )
    .context("Failed to load schema/digital_roster_schema.json")?;

    let import_roster: DigitalRosterImport =
        serde_json::from_value(source_json).with_context(|| {
            format!(
                "Failed to deserialize source data from '{}'.",
                args.source.display()
            )
        })?;

    let existing_manifest = load_existing_manifest_or_empty(&args.output)?;
    let merged_manifest = merge_digital_roster(existing_manifest, import_roster, args.force)?;

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

fn digital_roster_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("digital_roster_schema.json")
}

fn merge_digital_roster(
    mut manifest: Manifest,
    import_roster: DigitalRosterImport,
    force: bool,
) -> Result<Manifest> {
    let mut railway_model_index: BTreeMap<String, usize> = BTreeMap::new();
    for (index, model) in manifest.data.railway_models.iter().enumerate() {
        railway_model_index.insert(model.id.0.clone(), index);
    }

    let mut collection_item_to_model: BTreeMap<String, String> = BTreeMap::new();
    let mut collection_items_by_model: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in &manifest.data.collection_items {
        collection_item_to_model.insert(item.id.0.clone(), item.railway_model_id.0.clone());
        collection_items_by_model
            .entry(item.railway_model_id.0.clone())
            .or_default()
            .push(item.id.0.clone());
    }

    let mut decoder_by_id: BTreeMap<String, (DecoderType, String)> = BTreeMap::new();
    for decoder in &manifest.data.decoders {
        decoder_by_id.insert(
            decoder.id.clone(),
            (
                decoder.decoder_type.clone(),
                decoder.decoder_interface.clone(),
            ),
        );
    }

    // Map owned rolling stock ids to their railway model via the collection item
    let mut owned_to_model: BTreeMap<String, String> = BTreeMap::new();
    let mut owned_ids_by_model: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for ors in &manifest.data.owned_rolling_stocks {
        let cid = &ors.collection_item_id.0;
        if let Some(model_id) = collection_item_to_model.get(cid) {
            owned_to_model.insert(ors.id.0.clone(), model_id.clone());
            owned_ids_by_model
                .entry(model_id.clone())
                .or_default()
                .push(ors.id.0.clone());
        }
    }

    let existing_digital_models = existing_digital_model_ids(
        &manifest,
        &collection_item_to_model,
        &collection_items_by_model,
        &owned_to_model,
    );

    let mut errors = Vec::new();
    for item in &import_roster.items {
        let Some(model_index) = railway_model_index.get(&item.railway_model_id) else {
            errors.push(format!(
                "Digital roster item references unknown railwayModelId '{}'.",
                item.railway_model_id
            ));
            continue;
        };

        let has_existing_digital_data = {
            let model = &manifest.data.railway_models[*model_index];
            let rolling_stock_is_digital = model.rolling_stocks.iter().any(|stock| {
                stock.dcc_interface.is_some()
                    || matches!(stock.control, Some(Control::DccFitted | Control::DccSound))
            });
            rolling_stock_is_digital || existing_digital_models.contains(&item.railway_model_id)
        };

        if has_existing_digital_data && !force {
            errors.push(format!(
                "Digital data already exists for railwayModelId '{}' (use --force to overwrite).",
                item.railway_model_id
            ));
        }
    }

    if !errors.is_empty() {
        bail!(errors.join("\n"));
    }

    for item in import_roster.items {
        let Some(model_index) = railway_model_index.get(&item.railway_model_id).copied() else {
            continue;
        };

        let model = &mut manifest.data.railway_models[model_index];
        if model.rolling_stocks.is_empty() {
            bail!(
                "railwayModelId '{}' has no rollingStocks to update.",
                item.railway_model_id
            );
        }

        let (control, dcc_interface) = resolve_digital_state(&item.decoder_id, &decoder_by_id)
            .with_context(|| {
                format!(
                    "Failed to map decoder '{}' for railwayModelId '{}'.",
                    item.decoder_id, item.railway_model_id
                )
            })?;

        for stock in &mut model.rolling_stocks {
            stock.control = Some(control.clone());
            if let Some(interface) = dcc_interface.clone() {
                stock.dcc_interface = Some(interface);
            }
        }

        let mut owned_rolling_stock_id = owned_ids_by_model
            .get(&item.railway_model_id)
            .and_then(|ids| ids.first())
            .cloned();

        if owned_rolling_stock_id.is_none()
            && let Some(collection_item_id) = collection_items_by_model
                .get(&item.railway_model_id)
                .and_then(|ids| ids.first())
                .cloned()
        {
            let generated_id = format!("trn:owned-rolling-stock:{}", Uuid::new_v4());
            manifest.data.owned_rolling_stocks.push(OwnedRollingStock {
                id: OwnedRollingStockId(generated_id.clone()),
                collection_item_id: CollectionItemId(collection_item_id),
                rolling_stock_id: None,
                notes: None,
                dcc_address: None,
                installed_decoder_id: None,
                current_coupler_id: None,
            });
            owned_to_model.insert(generated_id.clone(), item.railway_model_id.clone());
            owned_ids_by_model
                .entry(item.railway_model_id.clone())
                .or_default()
                .push(generated_id.clone());
            owned_rolling_stock_id = Some(generated_id);
        }

        let owned_rolling_stock_id = owned_rolling_stock_id.with_context(|| {
            format!(
                "No ownedRollingStocks found for railwayModelId '{}'; import collection data first.",
                item.railway_model_id
            )
        })?;

        let entry_id = format!(
            "trn:digital-rolling-stock:{}",
            normalize_id_segment(&item.railway_model_id)
        );

        let updated_entry = DigitalRollingStock {
            id: entry_id,
            owned_rolling_stock_id: OwnedRollingStockId(owned_rolling_stock_id),
            dcc_address: item.address,
            decoder_id: Some(item.decoder_id),
        };

        if let Some(existing_index) =
            manifest
                .data
                .digital_rolling_stocks
                .iter()
                .position(|entry| {
                    digital_entry_matches_model(
                        entry,
                        &item.railway_model_id,
                        &collection_item_to_model,
                        &collection_items_by_model,
                        &owned_to_model,
                    )
                })
        {
            manifest.data.digital_rolling_stocks[existing_index] = updated_entry;
        } else {
            manifest.data.digital_rolling_stocks.push(updated_entry);
        }
    }

    Ok(manifest)
}

fn existing_digital_model_ids(
    manifest: &Manifest,
    collection_item_to_model: &BTreeMap<String, String>,
    collection_items_by_model: &BTreeMap<String, Vec<String>>,
    owned_to_model: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    manifest
        .data
        .digital_rolling_stocks
        .iter()
        .filter_map(|entry| {
            resolve_model_id_for_digital_entry(
                &entry.owned_rolling_stock_id.0,
                collection_item_to_model,
                collection_items_by_model,
                owned_to_model,
            )
        })
        .collect()
}

fn resolve_model_id_for_digital_entry(
    owned_rolling_stock_id: &str,
    collection_item_to_model: &BTreeMap<String, String>,
    collection_items_by_model: &BTreeMap<String, Vec<String>>,
    owned_to_model: &BTreeMap<String, String>,
) -> Option<String> {
    // If the id directly references a collection item id
    if let Some(model_id) = collection_item_to_model.get(owned_rolling_stock_id) {
        return Some(model_id.clone());
    }

    // If the id references an owned rolling stock entry, map it to the model
    if let Some(model_id) = owned_to_model.get(owned_rolling_stock_id) {
        return Some(model_id.clone());
    }

    // If it's already a railway model id, accept it
    if owned_rolling_stock_id.starts_with("trn:railway-model:") {
        return Some(owned_rolling_stock_id.to_string());
    }

    collection_items_by_model
        .iter()
        .find(|(_, item_ids)| {
            item_ids
                .iter()
                .any(|item_id| item_id == owned_rolling_stock_id)
        })
        .map(|(model_id, _)| model_id.clone())
}

fn digital_entry_matches_model(
    entry: &DigitalRollingStock,
    railway_model_id: &str,
    collection_item_to_model: &BTreeMap<String, String>,
    collection_items_by_model: &BTreeMap<String, Vec<String>>,
    owned_to_model: &BTreeMap<String, String>,
) -> bool {
    resolve_model_id_for_digital_entry(
        &entry.owned_rolling_stock_id.0,
        collection_item_to_model,
        collection_items_by_model,
        owned_to_model,
    )
    .is_some_and(|id| id == railway_model_id)
}

fn resolve_digital_state(
    decoder_id: &str,
    decoder_by_id: &BTreeMap<String, (DecoderType, String)>,
) -> Result<(Control, Option<DccInterface>)> {
    if let Some((decoder_type, decoder_interface)) = decoder_by_id.get(decoder_id) {
        let control = if matches!(decoder_type, DecoderType::Sound) {
            Control::DccSound
        } else {
            Control::DccFitted
        };

        return Ok((control, Some(parse_dcc_interface(decoder_interface)?)));
    }

    if let Some(inferred) = infer_dcc_interface_from_decoder_id(decoder_id) {
        return Ok((Control::DccFitted, Some(inferred)));
    }

    Ok((Control::DccFitted, None))
}

fn infer_dcc_interface_from_decoder_id(decoder_id: &str) -> Option<DccInterface> {
    let suffix = decoder_id.rsplit(':').next().unwrap_or(decoder_id);
    parse_dcc_interface_token(suffix)
}

fn parse_dcc_interface(raw: &str) -> Result<DccInterface> {
    parse_dcc_interface_token(raw)
        .with_context(|| format!("Unsupported decoder interface '{}'.", raw))
}

fn parse_dcc_interface_token(raw: &str) -> Option<DccInterface> {
    let token = normalize_token(raw);
    match token.as_str() {
        "NEM651" | "NEM_651" => Some(DccInterface::Nem651),
        "NEM652" | "NEM_652" => Some(DccInterface::Nem652),
        "NEM654" | "NEM_654" => Some(DccInterface::Nem654),
        "PLUX8" | "PLUX_8" => Some(DccInterface::Plux8),
        "PLUX12" | "PLUX_12" => Some(DccInterface::Plux12),
        "PLUX16" | "PLUX_16" => Some(DccInterface::Plux16),
        "PLUX22" | "PLUX_22" => Some(DccInterface::Plux22),
        "NEXT18" | "NEXT_18" => Some(DccInterface::Next18),
        "NEXT18S" | "NEXT_18_S" => Some(DccInterface::Next18S),
        "MTC21" | "MTC_21" => Some(DccInterface::Mtc21),
        _ => None,
    }
}

fn normalize_token(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    let mut last_was_separator = false;

    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_uppercase());
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('_');
            last_was_separator = true;
        }
    }

    normalized.trim_matches('_').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::import_collection::{load_existing_manifest_or_empty, write_zip};
    use crate::manifest::{Control, DccInterface};
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be monotonic")
            .as_nanos();
        std::env::temp_dir().join(format!("locrawl-{}-{}", name, nanos))
    }

    #[tokio::test]
    async fn import_digital_roster_sets_fitted_and_dcc_address() {
        let source = temp_path("digital-roster-source.json");
        let output = temp_path("digital-roster-manifest.json");

        let source_payload = json!({
            "items": [
                {
                    "railwayModelId": "trn:railway-model:acme:br101",
                    "decoderId": "trn:decoder:esu:lokpilot5",
                    "address": 77,
                    "installationDate": "2026-03-31"
                }
            ]
        });

        let manifest_payload = json!({
            "$schema": "https://rusty-shed.app/schemas/manifest/v1.json",
            "version": "1.0",
            "data": {
                "manufacturers": [
                    {"id": "trn:manufacturer:acme", "name": "ACME"},
                    {"id": "trn:manufacturer:esu", "name": "ESU"}
                ],
                "railwayCompanies": [
                    {"id": "trn:railway-company:db", "name": "Deutsche Bahn"}
                ],
                "railwayModels": [
                    {
                        "id": "trn:railway-model:acme:br101",
                        "manufacturerId": "trn:manufacturer:acme",
                        "productCode": "BR101",
                        "description": {"en": "Demo locomotive"},
                        "scale": "H0",
                        "epoch": "V",
                        "category": "LOCOMOTIVES",
                        "powerMethod": "DC",
                        "rollingStocks": [
                            {
                                "id": "rs-1",
                                "railwayCompanyId": "trn:railway-company:db",
                                "seriesCode": "BR 101",
                                "control": "DCC_READY"
                            }
                        ]
                    }
                ],
                "collectionItems": [
                    {
                        "id": "trn:collection-item:acme-br101",
                        "railwayModelId": "trn:railway-model:acme:br101",
                        "addedDate": "2026-03-31"
                    }
                ],
                "decoders": [
                    {
                        "id": "trn:decoder:esu:lokpilot5",
                        "manufacturerId": "trn:manufacturer:esu",
                        "productCode": "LokPilot5",
                        "decoderType": "PLAIN",
                        "protocol": "DCC",
                        "decoderInterface": "NEM_652"
                    }
                ],
                "digitalRollingStocks": []
            }
        });

        fs::write(
            &source,
            serde_json::to_string(&source_payload).expect("source payload should serialize"),
        )
        .expect("source file should be written");
        let manifest_seed =
            serde_json::to_string(&manifest_payload).expect("manifest payload should serialize");
        write_zip(&output, &manifest_seed).expect("manifest file should be written");

        run(ImportDigitalRosterArgs {
            source: source.clone(),
            output: output.clone(),
            force: false,
        })
        .await
        .expect("digital roster import should succeed");

        let merged_manifest =
            load_existing_manifest_or_empty(&output).expect("manifest should exist");

        let rolling_stock = &merged_manifest.data.railway_models[0].rolling_stocks[0];
        assert!(matches!(rolling_stock.control, Some(Control::DccFitted)));
        assert!(matches!(
            rolling_stock.dcc_interface,
            Some(DccInterface::Nem652)
        ));

        assert_eq!(merged_manifest.data.digital_rolling_stocks.len(), 1);
        assert_eq!(
            merged_manifest.data.digital_rolling_stocks[0].dcc_address,
            77
        );

        let _ = fs::remove_file(source);
        let _ = fs::remove_file(output);
    }
}
