use serde::{Deserialize, Serialize};

use super::rolling_stocks::RollingStock;

/// A railway modelling epoch, e.g. `"I"`, `"IIIa"`, `"III/IV"`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Epoch(pub String);

/// Monetary value with ISO 4217 currency code.
#[derive(Debug, Serialize, Deserialize)]
pub struct Price {
    pub amount: f64,
    pub currency: String,
}

/// Purchase information attached to a railway model.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseInfo {
    pub purchase_date: String,
    pub price: Price,
    pub seller: String,
}

/// Top-level category of a railway model product.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RailwayModelCategory {
    Locomotives,
    TrainSets,
    StarterSets,
    FreightCars,
    PassengerCars,
    ElectricMultipleUnits,
    Railcars,
}

/// Traction power method.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PowerMethod {
    Ac,
    Dc,
    TrixExpress,
}

/// Modelling scale.
#[derive(Debug, Serialize, Deserialize)]
pub enum Scale {
    Z,
    N,
    #[serde(rename = "TT")]
    Tt,
    H0,
    #[serde(rename = "0")]
    Zero,
    #[serde(rename = "1")]
    One,
    G,
}

/// A single railway model item (one product / set).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RailwayModel {
    pub id: String,
    pub manufacturer: String,
    pub product_code: String,
    pub description: String,
    pub power_method: PowerMethod,
    pub scale: Scale,
    pub epoch: Epoch,
    pub category: RailwayModelCategory,
    pub rolling_stocks: Vec<RollingStock>,
    pub delivery_date: Option<String>,
    pub purchase_info: Option<PurchaseInfo>,
}
