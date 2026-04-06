use serde::{Deserialize, Serialize};

use super::railway_model::PurchaseInfo;
use super::railway_model::RailwayModel;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionItemImport {
    pub id: String,
    pub railway_model_id: String,
    pub purchase_info: PurchaseInfo,
    #[serde(default)]
    pub catalog_item_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub version: i64,
    pub description: Option<String>,
    pub modified_at: String,
    #[serde(default)]
    pub items: Vec<CollectionItemImport>,
    pub railway_models: Vec<RailwayModel>,
    #[serde(default)]
    pub owned_rolling_stocks: Vec<super::OwnedRollingStock>,
}
