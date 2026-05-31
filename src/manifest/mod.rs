use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    pub version: ManifestVersion,
    pub exported_at: Option<DateTime<Utc>>,
    pub source: Option<String>,
    pub data: DataContainer,
}

impl Manifest {
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        serde_json::from_str(json)
    }

    pub fn to_pretty_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ManifestVersion {
    #[serde(rename = "1.0")]
    V1_0,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataContainer {
    #[serde(default)]
    pub manufacturers: Vec<Manufacturer>,
    #[serde(default)]
    pub railway_companies: Vec<RailwayCompany>,
    #[serde(default)]
    pub railway_models: Vec<RailwayModel>,
    #[serde(default)]
    pub collection_items: Vec<CollectionItem>,
    #[serde(default)]
    pub owned_rolling_stocks: Vec<OwnedRollingStock>,
    #[serde(default)]
    pub sellers: Vec<Seller>,
    #[serde(default)]
    pub maintenance_cards: Vec<MaintenanceCard>,
    #[serde(default)]
    pub track_products: Vec<TrackProduct>,
    #[serde(default)]
    pub track_inventories: Vec<TrackInventory>,
    #[serde(default)]
    pub prototypes: Vec<Prototype>,
    #[serde(default)]
    pub formation_categories: Vec<FormationCategory>,
    #[serde(default)]
    pub train_formations: Vec<TrainFormation>,
    #[serde(default)]
    pub wishlists: Vec<Wishlist>,
    #[serde(default)]
    pub decoders: Vec<Decoder>,
    #[serde(default)]
    pub digital_rolling_stocks: Vec<DigitalRollingStock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Manufacturer {
    /// Expected format: trn:manufacturer:{slug}
    pub id: ManufacturerId,
    pub name: String,
    pub registered_company_name: Option<String>,
    pub country_code: Option<String>,
    pub status: Option<ManufacturerStatus>,
    pub website_url: Option<String>,
    pub street_address: Option<String>,
    pub extended_address: Option<String>,
    pub city: Option<String>,
    pub state_region: Option<String>,
    pub postal_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManufacturerStatus {
    Active,
    Merged,
    OutOfBusiness,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RailwayCompany {
    /// Expected format: trn:railway-company:{slug}
    pub id: RailwayCompanyId,
    pub name: String,
    pub country_code: Option<String>,
    pub status: Option<RailwayCompanyStatus>,
    pub operating_since: Option<NaiveDate>,
    pub operating_until: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RailwayCompanyStatus {
    Active,
    Inactive,
    Merged,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RailwayModel {
    /// Expected format: trn:railway-model:{manufacturer}:{product-code}
    pub id: RailwayModelId,
    /// Expected format: trn:manufacturer:{slug}
    pub manufacturer_id: ManufacturerId,
    pub product_code: String,
    pub description: LocalizedText,
    pub details: Option<LocalizedText>,
    pub scale: Scale,
    pub epoch: String,
    pub category: Category,
    pub power_method: PowerMethod,
    pub delivery_date: Option<String>,
    pub availability_status: Option<AvailabilityStatus>,
    pub image: Option<String>,
    #[serde(default)]
    pub rolling_stocks: Vec<RollingStock>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Scale {
    H0,
    H0m,
    H0e,
    N,
    TT,
    Z,
    G,
    #[serde(rename = "1")]
    Scale1,
    #[serde(rename = "0")]
    Scale0,
    #[serde(rename = "00")]
    Scale00,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PowerMethod {
    Ac,
    Dc,
    TrixExpress,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AvailabilityStatus {
    Available,
    Announced,
    Cancelled,
    Discontinued,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Category {
    Locomotives,
    PassengerCars,
    FreightCars,
    ElectricMultipleUnits,
    Railcars,
    TrainSets,
    StarterSets,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LocalizedText {
    pub en: Option<String>,
    pub it: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FeatureFlag {
    Yes,
    No,
    NotApplicable,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct ManufacturerId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct RailwayCompanyId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct RailwayModelId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct CollectionItemId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct OwnedRollingStockId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct SellerId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct MaintenanceCardId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct MaintenanceEventId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct TrackId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct TrackInventoryId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct TrackPurchaseId(pub String);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RollingStock {
    pub id: Option<String>,
    /// Expected format: trn:railway-company:{slug}
    pub railway_company_id: RailwayCompanyId,
    pub series_code: String,
    pub road_number: Option<String>,
    pub livery: Option<String>,
    pub friendly_name: Option<String>,
    pub series: Option<String>,
    pub depot: Option<String>,
    pub electric_multiple_unit_type: Option<ElectricMultipleUnitType>,
    pub freight_car_type: Option<FreightCarType>,
    pub locomotive_type: Option<LocomotiveType>,
    pub passenger_car_type: Option<PassengerCarType>,
    pub railcar_type: Option<RailcarType>,
    pub service_level: Option<ServiceLevel>,
    pub is_dummy: Option<bool>,
    pub length_inches: Option<f64>,
    pub length_millimeters: Option<f64>,
    pub technical_minimum_radius_mm: Option<f64>,
    pub technical_coupling_socket: Option<TechnicalCouplingSocket>,
    pub technical_coupling_close_couplers: Option<FeatureFlag>,
    pub technical_coupling_digital_shunting: Option<FeatureFlag>,
    pub technical_flywheel_fitted: Option<FeatureFlag>,
    pub technical_body_shell: Option<TechnicalMaterial>,
    pub technical_chassis: Option<TechnicalMaterial>,
    pub technical_interior_lights: Option<FeatureFlag>,
    pub technical_lights: Option<FeatureFlag>,
    pub technical_sprung_buffers: Option<FeatureFlag>,
    pub dcc_interface: Option<DccInterface>,
    pub control: Option<Control>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ElectricMultipleUnitType {
    DrivingCar,
    HighSpeedTrain,
    MotorCar,
    PowerCar,
    TrailerCar,
    TrainSet,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FreightCarType {
    AutoTransportCars,
    BrakeWagon,
    ClosedCargoVehicle,
    ContainerCars,
    CoveredFreightCars,
    DeepWellFlatCars,
    DumpCars,
    FlatWagon,
    Gondola,
    HeavyGoodsWagons,
    HingedCoverWagons,
    HopperWagon,
    RefrigeratorCars,
    SiloContainerCars,
    SlideTarpaulinWagon,
    SlidingWallBoxcars,
    SpecialTransport,
    StakeWagons,
    SwingRoofWagon,
    TankCars,
    TelescopeHoodWagons,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LocomotiveType {
    SteamLocomotive,
    DieselLocomotive,
    ElectricLocomotive,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PassengerCarType {
    BaggageCar,
    BuffetCar,
    CombineCar,
    CompartmentCoach,
    DiningCar,
    DoubleDecker,
    DomeCar,
    DrivingTrailer,
    Lounge,
    Observation,
    OpenCoach,
    RailwayPostOffice,
    SleepingCar,
    Sleeperette,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RailcarType {
    PowerCar,
    TrailerBaggageCar,
    TrailerCar,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceLevel {
    First,
    Second,
    Third,
    FirstSecond,
    SecondThird,
    FirstSecondThird,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TechnicalCouplingSocket {
    None,
    Nem355,
    Nem356,
    Nem357,
    Nem359,
    Nem360,
    Nem362,
    Nem365,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TechnicalMaterial {
    Plastic,
    MetalDieCast,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DccInterface {
    #[serde(rename = "NEM_651")]
    Nem651,
    #[serde(rename = "NEM_652")]
    Nem652,
    #[serde(rename = "NEM_654")]
    Nem654,
    #[serde(rename = "PLUX_8")]
    Plux8,
    #[serde(rename = "PLUX_12")]
    Plux12,
    #[serde(rename = "PLUX_16")]
    Plux16,
    #[serde(rename = "PLUX_22")]
    Plux22,
    #[serde(rename = "NEXT_18")]
    Next18,
    #[serde(rename = "NEXT_18_S")]
    Next18S,
    #[serde(rename = "MTC_21")]
    Mtc21,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Control {
    DccReady,
    DccFitted,
    DccSound,
    NoDcc,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CollectionItem {
    /// Expected format: trn:collection-item:{uuid}
    pub id: CollectionItemId,
    /// Expected format: trn:railway-model:{manufacturer}:{product-code}
    pub railway_model_id: RailwayModelId,
    pub added_date: NaiveDate,
    pub removed_date: Option<NaiveDate>,
    pub purchase_condition: Option<PurchaseCondition>,
    pub model_condition: Option<ModelCondition>,
    pub box_condition: Option<BoxCondition>,
    pub notes: Option<String>,
    pub image: Option<String>,
    pub purchase: Option<Purchase>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PurchaseCondition {
    New,
    PreOwned,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ModelCondition {
    Mint,
    NearMint,
    Excellent,
    VeryGood,
    Good,
    Fair,
    Poor,
    ForParts,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BoxCondition {
    OriginalMint,
    OriginalGood,
    OriginalWorn,
    ReplacementBox,
    NoBox,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Purchase {
    pub r#type: PurchaseType,
    pub purchase_date: Option<NaiveDate>,
    pub price: Option<Money>,
    /// Expected format: trn:seller:{slug}
    pub seller_id: Option<SellerId>,
    pub sale_date: Option<NaiveDate>,
    pub sale_price: Option<Money>,
    pub deposit_amount: Option<Money>,
    pub expected_delivery: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PurchaseType {
    Purchased,
    Sold,
    PreOrdered,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Money {
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Seller {
    /// Expected format: trn:seller:{slug}
    pub id: SellerId,
    pub name: String,
    pub seller_type: SellerType,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website_url: Option<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SellerType {
    Shop,
    Private,
    Marketplace,
    Distributor,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Address {
    pub street: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaintenanceCard {
    /// Expected format: trn:maintenance-card:{uuid}
    pub id: MaintenanceCardId,
    /// Expected format: trn:collection-item:{uuid}
    pub collection_item_id: CollectionItemId,
    /// Expected format: trn:owned-rolling-stock:{uuid}
    pub owned_rolling_stock_id: Option<OwnedRollingStockId>,
    pub last_maintenance_date: Option<NaiveDate>,
    pub next_maintenance_date: Option<NaiveDate>,
    #[serde(default)]
    pub events: Vec<MaintenanceEvent>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedRollingStock {
    /// Expected format: trn:owned-rolling-stock:{uuid}
    pub id: OwnedRollingStockId,
    /// Expected format: trn:collection-item:{uuid}
    pub collection_item_id: CollectionItemId,
    /// References the catalogue rolling stock entry; omitted when unknown
    pub rolling_stock_id: Option<String>,
    pub notes: Option<String>,
    pub dcc_address: Option<i64>,
    pub installed_decoder_id: Option<String>,
    pub current_coupler_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MaintenanceEvent {
    /// Expected format: trn:maintenance-event:{uuid}
    pub id: MaintenanceEventId,
    pub date: NaiveDate,
    pub r#type: MaintenanceEventType,
    pub description: Option<String>,
    pub cost: Option<Money>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MaintenanceEventType {
    Cleaning,
    Lubrication,
    Repair,
    Modification,
    Inspection,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackProduct {
    /// Expected format: trn:track:{manufacturer}:{product-code}
    pub track_id: TrackId,
    /// Expected format: trn:manufacturer:{slug}
    pub manufacturer_id: ManufacturerId,
    pub product_code: String,
    pub description: String,
    pub track_type: TrackType,
    pub track_code: TrackCode,
    pub with_roadbed: bool,
    pub length: Option<i64>,
    pub radius: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TrackType {
    Straight,
    Curve,
    Turnout,
    FlexTrack,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TrackCode {
    #[serde(rename = "CODE_70")]
    Code70,
    #[serde(rename = "CODE_75")]
    Code75,
    #[serde(rename = "CODE_83")]
    Code83,
    #[serde(rename = "CODE_100")]
    Code100,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackInventory {
    /// Expected format: trn:track-inventory:{slug-or-uuid}
    pub id: TrackInventoryId,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub items: Vec<TrackInventoryItem>,
    #[serde(default)]
    pub purchases: Vec<TrackPurchase>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackInventoryItem {
    /// Expected format: trn:track:{manufacturer}:{product-code}
    pub track_id: TrackId,
    pub quantity: i64,
    pub required: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackPurchase {
    /// Expected format: trn:track-purchase:{uuid}
    pub id: TrackPurchaseId,
    /// Expected format: trn:track:{manufacturer}:{product-code}
    pub track_id: TrackId,
    pub quantity: i64,
    pub price: Money,
    /// Expected format: trn:seller:{slug}
    pub seller_id: Option<SellerId>,
    pub purchase_date: NaiveDate,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Prototype {
    pub id: String,
    /// Expected format: trn:railway-company:{slug}
    pub railway_company_id: RailwayCompanyId,
    pub series_code: String,
    pub friendly_name: Option<String>,
    pub specification_type: SpecificationType,
    pub locomotive_type: Option<String>,
    pub locomotive_series: Option<String>,
    pub service_level: Option<String>,
    pub passenger_car_type: Option<String>,
    pub freight_car_type: Option<String>,
    pub railcar_type: Option<String>,
    pub electric_multiple_unit_type: Option<String>,
    pub elements_count: Option<i64>,
    pub is_permanently_coupled: Option<bool>,
    pub is_motorized: bool,
    pub is_custom: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpecificationType {
    Locomotive,
    PassengerCar,
    FreightCar,
    Railcar,
    ElectricMultipleUnit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FormationCategory {
    pub id: String,
    pub name: String,
    pub is_custom: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrainFormation {
    pub id: String,
    pub name: String,
    pub category_id: Option<String>,
    pub start_year: Option<i64>,
    pub end_year: Option<i64>,
    pub epoch: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub elements: Vec<FormationElement>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FormationElement {
    pub id: String,
    pub prototype_id: String,
    /// Expected format: trn:owned-rolling-stock:{uuid}
    pub owned_rolling_stock_id: OwnedRollingStockId,
    pub position_order: i64,
    pub traction_override: i8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Wishlist {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    pub is_default: bool,
    #[serde(default)]
    pub items: Vec<WishlistItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WishlistItem {
    pub id: String,
    pub railway_model_id: RailwayModelId,
    pub priority: WishlistPriority,
    pub status: WishlistStatus,
    pub added_date: NaiveDate,
    pub removed_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub desired_price: Option<Money>,
    pub purchased_price: Option<Money>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WishlistPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WishlistStatus {
    Wanted,
    OnOrder,
    Purchased,
    Ignored,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Decoder {
    pub id: String,
    pub manufacturer_id: ManufacturerId,
    pub product_code: String,
    pub decoder_type: DecoderType,
    pub protocol: DecoderProtocol,
    pub decoder_interface: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecoderType {
    Plain,
    Sound,
    Function,
    MultiProtocol,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DecoderProtocol {
    Dcc,
    Mfx,
    Selectrix,
    Motorola,
    Fmz,
    Next18,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DigitalRollingStock {
    pub id: String,
    /// Expected format: trn:owned-rolling-stock:{uuid}
    pub owned_rolling_stock_id: OwnedRollingStockId,
    pub dcc_address: i64,
    pub decoder_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ------- Manifest round-trip -------

    #[test]
    fn manifest_from_json_and_to_pretty_json_round_trips() {
        let original = Manifest {
            schema: Some("https://rusty-shed.app/schemas/manifest/v1.json".to_string()),
            version: ManifestVersion::V1_0,
            exported_at: None,
            source: Some("locrawl-test".to_string()),
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
        };

        let json = original
            .to_pretty_json()
            .expect("serialization should succeed");
        let restored = Manifest::from_json(&json).expect("deserialization should succeed");

        assert_eq!(
            restored.schema.as_deref(),
            Some("https://rusty-shed.app/schemas/manifest/v1.json")
        );
        assert!(matches!(restored.version, ManifestVersion::V1_0));
        assert_eq!(restored.source.as_deref(), Some("locrawl-test"));
    }

    #[test]
    fn manifest_from_json_fails_on_invalid_json() {
        let result = Manifest::from_json("{ not valid json }");
        assert!(result.is_err());
    }

    // ------- ManifestVersion serialization -------

    #[test]
    fn manifest_version_serializes_as_1_dot_0() {
        let v = serde_json::to_value(ManifestVersion::V1_0).unwrap();
        assert_eq!(v, json!("1.0"));
    }

    // ------- Scale serialization -------

    #[test]
    fn scale_serializes_special_variants_correctly() {
        assert_eq!(serde_json::to_value(Scale::H0).unwrap(), json!("H0"));
        assert_eq!(serde_json::to_value(Scale::N).unwrap(), json!("N"));
        assert_eq!(serde_json::to_value(Scale::TT).unwrap(), json!("TT"));
        assert_eq!(serde_json::to_value(Scale::Z).unwrap(), json!("Z"));
        assert_eq!(serde_json::to_value(Scale::Scale0).unwrap(), json!("0"));
        assert_eq!(serde_json::to_value(Scale::Scale1).unwrap(), json!("1"));
        assert_eq!(serde_json::to_value(Scale::Scale00).unwrap(), json!("00"));
        assert_eq!(serde_json::to_value(Scale::G).unwrap(), json!("G"));
    }

    #[test]
    fn scale_deserializes_special_variants_correctly() {
        assert!(matches!(
            serde_json::from_value::<Scale>(json!("0")).unwrap(),
            Scale::Scale0
        ));
        assert!(matches!(
            serde_json::from_value::<Scale>(json!("1")).unwrap(),
            Scale::Scale1
        ));
        assert!(matches!(
            serde_json::from_value::<Scale>(json!("00")).unwrap(),
            Scale::Scale00
        ));
    }

    // ------- PowerMethod serialization -------

    #[test]
    fn power_method_serializes_screaming_snake_case() {
        assert_eq!(serde_json::to_value(PowerMethod::Ac).unwrap(), json!("AC"));
        assert_eq!(serde_json::to_value(PowerMethod::Dc).unwrap(), json!("DC"));
        assert_eq!(
            serde_json::to_value(PowerMethod::TrixExpress).unwrap(),
            json!("TRIX_EXPRESS")
        );
    }

    // ------- DccInterface serialization -------

    #[test]
    fn dcc_interface_serializes_with_underscore_separators() {
        assert_eq!(
            serde_json::to_value(DccInterface::Nem651).unwrap(),
            json!("NEM_651")
        );
        assert_eq!(
            serde_json::to_value(DccInterface::Nem652).unwrap(),
            json!("NEM_652")
        );
        assert_eq!(
            serde_json::to_value(DccInterface::Plux8).unwrap(),
            json!("PLUX_8")
        );
        assert_eq!(
            serde_json::to_value(DccInterface::Next18).unwrap(),
            json!("NEXT_18")
        );
        assert_eq!(
            serde_json::to_value(DccInterface::Next18S).unwrap(),
            json!("NEXT_18_S")
        );
        assert_eq!(
            serde_json::to_value(DccInterface::Mtc21).unwrap(),
            json!("MTC_21")
        );
    }

    #[test]
    fn dcc_interface_deserializes_from_underscore_form() {
        assert!(matches!(
            serde_json::from_value::<DccInterface>(json!("NEM_651")).unwrap(),
            DccInterface::Nem651
        ));
        assert!(matches!(
            serde_json::from_value::<DccInterface>(json!("NEXT_18_S")).unwrap(),
            DccInterface::Next18S
        ));
    }

    // ------- TrackCode serialization -------

    #[test]
    fn track_code_serializes_with_underscore_separators() {
        assert_eq!(
            serde_json::to_value(TrackCode::Code70).unwrap(),
            json!("CODE_70")
        );
        assert_eq!(
            serde_json::to_value(TrackCode::Code75).unwrap(),
            json!("CODE_75")
        );
        assert_eq!(
            serde_json::to_value(TrackCode::Code83).unwrap(),
            json!("CODE_83")
        );
        assert_eq!(
            serde_json::to_value(TrackCode::Code100).unwrap(),
            json!("CODE_100")
        );
    }

    #[test]
    fn track_code_deserializes_from_underscore_form() {
        assert!(matches!(
            serde_json::from_value::<TrackCode>(json!("CODE_83")).unwrap(),
            TrackCode::Code83
        ));
    }

    // ------- Control serialization -------

    #[test]
    fn control_serializes_screaming_snake_case() {
        assert_eq!(
            serde_json::to_value(Control::DccReady).unwrap(),
            json!("DCC_READY")
        );
        assert_eq!(
            serde_json::to_value(Control::DccFitted).unwrap(),
            json!("DCC_FITTED")
        );
        assert_eq!(
            serde_json::to_value(Control::DccSound).unwrap(),
            json!("DCC_SOUND")
        );
        assert_eq!(
            serde_json::to_value(Control::NoDcc).unwrap(),
            json!("NO_DCC")
        );
    }

    #[test]
    fn owned_rolling_stock_id_round_trips() {
        let id = OwnedRollingStockId("trn:owned-rolling-stock:abc".to_string());
        let value = serde_json::to_value(id).unwrap();
        assert_eq!(value, json!("trn:owned-rolling-stock:abc"));
    }

    #[test]
    fn formation_element_requires_owned_rolling_stock_id() {
        let value = json!({
            "id": "elem-1",
            "prototypeId": "proto-1",
            "positionOrder": 0,
            "tractionOverride": 0
        });

        let result = serde_json::from_value::<FormationElement>(value);
        assert!(result.is_err());
    }

    #[test]
    fn maintenance_card_accepts_owned_rolling_stock_id() {
        let value = json!({
            "id": "trn:maintenance-card:card-1",
            "collectionItemId": "trn:collection-item:item-1",
            "ownedRollingStockId": "trn:owned-rolling-stock:ors-1",
            "events": []
        });

        let card = serde_json::from_value::<MaintenanceCard>(value).unwrap();
        assert_eq!(
            card.owned_rolling_stock_id.unwrap().0,
            "trn:owned-rolling-stock:ors-1"
        );
    }
}
