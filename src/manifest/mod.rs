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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Category {
    #[serde(rename = "type")]
    pub category_type: CategoryType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CategoryType {
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
    pub last_maintenance_date: Option<NaiveDate>,
    pub next_maintenance_date: Option<NaiveDate>,
    #[serde(default)]
    pub events: Vec<MaintenanceEvent>,
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
    pub car_type: String,
    pub service_level: Option<String>,
    pub category: String,
    pub is_motorized: bool,
    pub is_custom: bool,
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
    pub owned_rolling_stock_id: Option<String>,
    pub position_order: i64,
    pub traction_override: TractionOverride,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TractionOverride {
    #[serde(rename = "-1")]
    MinusOne,
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
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
    pub owned_rolling_stock_id: String,
    pub dcc_address: i64,
    pub decoder_id: Option<String>,
}
