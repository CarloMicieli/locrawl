use serde::{Deserialize, Serialize};

/// Rolling-stock category.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Category {
    ElectricMultipleUnit,
    FreightCar,
    Locomotive,
    PassengerCar,
    Railcar,
}

/// Rolling-stock sub-category.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubCategory {
    AutoTransportCars,
    BaggageCar,
    BrakeWagon,
    BuffetCar,
    ClosedCargoVehicle,
    CombineCar,
    CompartmentCoach,
    ContainerCars,
    CoveredFreightCars,
    DeepWellFlatCars,
    DieselLocomotive,
    DiningCar,
    DomeCar,
    DoubleDecker,
    DrivingCar,
    DrivingTrailer,
    DumpCars,
    ElectricLocomotive,
    Gondola,
    HeavyGoodsWagons,
    HighSpeedTrain,
    HingedCoverWagons,
    HopperWagon,
    Lounge,
    MotorCar,
    Observation,
    OpenCoach,
    PowerCar,
    RailwayPostOffice,
    RefrigeratorCars,
    SiloContainerCars,
    Sleeperette,
    SleepingCar,
    SlideTarpaulinWagon,
    SlidingWallBoxcars,
    SpecialTransport,
    StakeWagons,
    SteamLocomotive,
    SwingRoofWagon,
    TankCars,
    TelescopeHoodWagons,
    TrailerCar,
    TrainSet,
}

/// Passenger service level.
#[derive(Debug, Serialize, Deserialize)]
pub enum ServiceLevel {
    #[serde(rename = "1cl")]
    OneCl,
    #[serde(rename = "2cl")]
    TwoCl,
    #[serde(rename = "3cl")]
    ThreeCl,
    #[serde(rename = "1cl/2cl")]
    OneClTwoCl,
    #[serde(rename = "1cl/2cl/3cl")]
    OneClTwoClThreeCl,
    #[serde(rename = "2cl/3cl")]
    TwoClThreeCl,
}

/// An individual rolling-stock vehicle within a railway model product.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollingStock {
    pub id: String,
    pub category: Category,
    pub railway: String,
    pub road_number: Option<String>,
    pub type_name: Option<String>,
    pub sub_category: Option<SubCategory>,
    pub depot: Option<String>,
    /// Length of the rolling stock in millimetres.
    pub length: Option<u32>,
    pub livery: Option<String>,
    pub service_level: Option<ServiceLevel>,
    pub series: Option<String>,
    pub is_dummy: Option<bool>,
}
