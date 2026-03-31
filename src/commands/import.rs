use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, NaiveDate, Utc};
use clap::Args;
use jsonschema::Validator;
use serde_json::Value;
use slug::slugify;
use uuid::Uuid;

use crate::import::{
    Category as ImportRollingStockCategory, Collection, PowerMethod as ImportPowerMethod,
    RailwayModel as ImportRailwayModel, RailwayModelCategory as ImportRailwayModelCategory,
    RollingStock as ImportRollingStock, Scale as ImportScale, ServiceLevel as ImportServiceLevel,
    SubCategory as ImportSubCategory,
};
use crate::manifest::{
    self, Category, CategoryType, CollectionItem, CollectionItemId, Control, DataContainer,
    ElectricMultipleUnitType, FreightCarType, LocalizedText, LocomotiveType, Manifest,
    ManifestVersion, Manufacturer, ManufacturerId, PassengerCarType, PowerMethod, Purchase,
    PurchaseType, RailcarType, RailwayCompany, RailwayCompanyId, RailwayCompanyStatus,
    RailwayModel, RailwayModelId, RollingStock, Scale, Seller, SellerId, SellerType, ServiceLevel,
};

#[derive(Debug, Args, Clone)]
pub struct ImportArgs {
    /// Path to source collection JSON (validated against schema/import_schema.json)
    #[arg(short = 's', long = "source")]
    pub source: PathBuf,

    /// Path where the generated manifest JSON will be written
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,
}

pub fn run(args: ImportArgs) -> Result<()> {
    let import_schema =
        load_schema(&import_schema_path()).context("Failed to load schema/import_schema.json")?;
    let manifest_schema = load_schema(&manifest_schema_path())
        .context("Failed to load schema/manifest_schema.json")?;

    let import_validator = jsonschema::validator_for(&import_schema)
        .context("Failed to compile import schema validator")?;
    let manifest_validator = jsonschema::validator_for(&manifest_schema)
        .context("Failed to compile manifest schema validator")?;

    let source_content = fs::read_to_string(&args.source)
        .with_context(|| format!("Failed to read source file '{}'.", args.source.display()))?;

    let source_json: Value = serde_json::from_str(&source_content)
        .with_context(|| format!("Failed to parse JSON from '{}'.", args.source.display()))?;

    validate_payload(&import_validator, &source_json, "import input")?;

    let import_collection: Collection = serde_json::from_value(source_json).with_context(|| {
        format!(
            "Failed to deserialize source data from '{}'.",
            args.source.display()
        )
    })?;

    let manifest = map_import_to_manifest(&import_collection)?;

    let mut manifest_value =
        serde_json::to_value(&manifest).context("Failed to serialize manifest to JSON value")?;
    strip_nulls(&mut manifest_value);
    validate_payload(&manifest_validator, &manifest_value, "manifest output")?;

    let manifest_json = serde_json::to_string_pretty(&manifest_value)
        .context("Failed to serialize manifest JSON string")?;

    if let Some(parent) = args.output.parent()
        && !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent).with_context(|| {
            format!("Failed to create output directory '{}'.", parent.display())
        })?;
    }

    fs::write(&args.output, manifest_json)
        .with_context(|| format!("Failed to write output file '{}'.", args.output.display()))?;

    println!("Manifest successfully written to {}", args.output.display());
    Ok(())
}

fn strip_nulls(value: &mut Value) {
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

pub fn map_import_to_manifest(import: &Collection) -> Result<Manifest> {
    let exported_at = parse_rfc3339_to_utc(&import.modified_at, "modifiedAt")?;

    let mut manufacturers: BTreeMap<String, Manufacturer> = BTreeMap::new();
    let mut railway_companies: BTreeMap<String, RailwayCompany> = BTreeMap::new();
    let mut sellers: BTreeMap<String, Seller> = BTreeMap::new();

    let mut railway_models = Vec::with_capacity(import.railway_models.len());
    let mut collection_items = Vec::with_capacity(import.railway_models.len());

    for model in &import.railway_models {
        let manufacturer_slug = normalize_id_segment(&model.manufacturer);
        let manufacturer_id = ManufacturerId(format!("trn:manufacturer:{}", manufacturer_slug));

        manufacturers
            .entry(manufacturer_id.0.clone())
            .or_insert_with(|| Manufacturer {
                id: manufacturer_id.clone(),
                name: model.manufacturer.clone(),
                registered_company_name: None,
                country_code: None,
                status: None,
                website_url: None,
                street_address: None,
                extended_address: None,
                city: None,
                state_region: None,
                postal_code: None,
            });

        railway_models.push(map_railway_model(
            model,
            &manufacturer_id,
            &mut railway_companies,
        ));

        let railway_model_id = RailwayModelId(format!(
            "trn:railway-model:{}:{}",
            manufacturer_slug,
            normalize_id_segment(&model.product_code)
        ));

        let mut purchase = None;
        let mut added_date = exported_at.date_naive();

        if let Some(import_purchase) = &model.purchase_info {
            added_date = parse_date(&import_purchase.purchase_date, "purchaseDate")?;

            let seller_slug = normalize_id_segment(&import_purchase.seller);
            let seller_id = SellerId(format!("trn:seller:{}", seller_slug));

            sellers
                .entry(seller_id.0.clone())
                .or_insert_with(|| Seller {
                    id: seller_id.clone(),
                    name: import_purchase.seller.clone(),
                    seller_type: SellerType::Shop,
                    email: None,
                    phone: None,
                    website_url: None,
                    address: None,
                });

            purchase = Some(Purchase {
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
            });
        }

        collection_items.push(CollectionItem {
            id: CollectionItemId(format!("trn:collection-item:{}", Uuid::new_v4())),
            railway_model_id,
            added_date,
            removed_date: None,
            purchase_condition: None,
            model_condition: None,
            box_condition: None,
            notes: None,
            image: None,
            purchase,
        });
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
            sellers: sellers.into_values().collect(),
            maintenance_cards: vec![],
            track_products: vec![],
            track_inventories: vec![],
            prototypes: vec![],
            formation_categories: vec![],
            train_formations: vec![],
        },
    })
}

fn map_railway_model(
    model: &ImportRailwayModel,
    manufacturer_id: &ManufacturerId,
    railway_companies: &mut BTreeMap<String, RailwayCompany>,
) -> RailwayModel {
    let manufacturer_slug = normalize_id_segment(&model.manufacturer);
    let product_slug = normalize_id_segment(&model.product_code);

    RailwayModel {
        id: RailwayModelId(format!(
            "trn:railway-model:{}:{}",
            manufacturer_slug, product_slug
        )),
        manufacturer_id: manufacturer_id.clone(),
        product_code: model.product_code.clone(),
        description: LocalizedText {
            en: Some(model.description.clone()),
            it: None,
        },
        details: None,
        scale: map_scale(&model.scale),
        epoch: model.epoch.0.clone(),
        category: Category {
            category_type: map_model_category(&model.category),
        },
        power_method: map_power_method(&model.power_method),
        delivery_date: model.delivery_date.clone(),
        availability_status: None,
        image: None,
        rolling_stocks: model
            .rolling_stocks
            .iter()
            .map(|stock| map_rolling_stock(stock, railway_companies))
            .collect(),
    }
}

fn map_rolling_stock(
    stock: &ImportRollingStock,
    railway_companies: &mut BTreeMap<String, RailwayCompany>,
) -> RollingStock {
    let railway_company_id = railway_company_id_for(&stock.railway, railway_companies);

    RollingStock {
        id: Some(stock.id.clone()),
        railway_company_id,
        series_code: stock
            .type_name
            .clone()
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
    }
}

fn railway_company_id_for(
    railway_name: &str,
    railway_companies: &mut BTreeMap<String, RailwayCompany>,
) -> RailwayCompanyId {
    let railway_slug = normalize_id_segment(railway_name);
    let company_id = RailwayCompanyId(format!("trn:railway-company:{}", railway_slug));

    railway_companies
        .entry(company_id.0.clone())
        .or_insert_with(|| RailwayCompany {
            id: company_id.clone(),
            name: railway_name.to_string(),
            country_code: None,
            status: Some(RailwayCompanyStatus::Active),
            operating_since: None,
            operating_until: None,
        });

    company_id
}

fn map_model_category(category: &ImportRailwayModelCategory) -> CategoryType {
    match category {
        ImportRailwayModelCategory::Locomotives => CategoryType::Locomotives,
        ImportRailwayModelCategory::TrainSets => CategoryType::TrainSets,
        ImportRailwayModelCategory::StarterSets => CategoryType::StarterSets,
        ImportRailwayModelCategory::FreightCars => CategoryType::FreightCars,
        ImportRailwayModelCategory::PassengerCars => CategoryType::PassengerCars,
        ImportRailwayModelCategory::ElectricMultipleUnits => CategoryType::ElectricMultipleUnits,
        ImportRailwayModelCategory::Railcars => CategoryType::Railcars,
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

fn validate_payload(validator: &Validator, payload: &Value, label: &str) -> Result<()> {
    let errors: Vec<String> = validator
        .iter_errors(payload)
        .map(|error| {
            let location = if error.instance_path.to_string().is_empty() {
                "$".to_string()
            } else {
                format!("${}", error.instance_path)
            };
            format!("Validation Error ({} at {}): {}", label, location, error)
        })
        .collect();

    if errors.is_empty() {
        return Ok(());
    }

    bail!(errors.join("\n"));
}

fn load_schema(path: &Path) -> Result<Value> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read schema '{}'.", path.display()))?;

    let schema = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse schema '{}'.", path.display()))?;

    Ok(schema)
}

fn import_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("import_schema.json")
}

fn manifest_schema_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("manifest_schema.json")
}

fn parse_rfc3339_to_utc(raw: &str, field_name: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .with_context(|| format!("Invalid date-time '{}' for field '{}'.", raw, field_name))
}

fn parse_date(raw: &str, field_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(raw, "%Y-%m-%d")
        .with_context(|| format!("Invalid date '{}' for field '{}'.", raw, field_name))
}

fn normalize_id_segment(raw: &str) -> String {
    let candidate = slugify(raw);
    if candidate.is_empty() {
        "unknown".to_string()
    } else {
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::import::{
        Epoch, PowerMethod as ImportPowerMethod, Price, PurchaseInfo,
        RailwayModel as ImportRailwayModel, RailwayModelCategory as ImportRailwayModelCategory,
        Scale as ImportScale,
    };

    #[test]
    fn map_import_to_manifest_produces_schema_valid_json() {
        let import = Collection {
            version: 1,
            description: Some("Test collection".to_string()),
            modified_at: "2026-03-31T12:00:00Z".to_string(),
            railway_models: vec![ImportRailwayModel {
                id: "rm-1".to_string(),
                manufacturer: "ACME Rail".to_string(),
                product_code: "A-123".to_string(),
                description: "Demo locomotive".to_string(),
                power_method: ImportPowerMethod::Dc,
                scale: ImportScale::H0,
                epoch: Epoch("IV".to_string()),
                category: ImportRailwayModelCategory::Locomotives,
                rolling_stocks: vec![ImportRollingStock {
                    id: "stock-1".to_string(),
                    category: ImportRollingStockCategory::Locomotive,
                    railway: "Deutsche Bahn".to_string(),
                    road_number: Some("101 001-6".to_string()),
                    type_name: Some("BR 101".to_string()),
                    sub_category: Some(ImportSubCategory::ElectricLocomotive),
                    depot: None,
                    length: Some(218),
                    livery: Some("Traffic Red".to_string()),
                    service_level: None,
                    series: Some("101".to_string()),
                    is_dummy: Some(false),
                }],
                delivery_date: Some("2026/03".to_string()),
                purchase_info: Some(PurchaseInfo {
                    purchase_date: "2026-03-01".to_string(),
                    price: Price {
                        amount: 199.0,
                        currency: "EUR".to_string(),
                    },
                    seller: "Model Bahn Shop".to_string(),
                }),
            }],
        };

        let manifest = map_import_to_manifest(&import).expect("mapping should succeed");
        let mut manifest_value =
            serde_json::to_value(&manifest).expect("manifest should serialize");
        strip_nulls(&mut manifest_value);

        let schema = load_schema(&manifest_schema_path()).expect("schema should load");
        let validator = jsonschema::validator_for(&schema).expect("schema should compile");
        let errors: Vec<String> = validator
            .iter_errors(&manifest_value)
            .map(|e| e.to_string())
            .collect();

        assert!(
            errors.is_empty(),
            "manifest should satisfy schema, got: {:?}",
            errors
        );

        assert_eq!(manifest.data.railway_models.len(), 1);
        assert_eq!(manifest.data.manufacturers.len(), 1);
        assert_eq!(manifest.data.railway_companies.len(), 1);
    }
}
