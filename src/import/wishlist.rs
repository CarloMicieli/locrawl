use serde::{Deserialize, Serialize};

use super::railway_model::RailwayModel;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Wishlist {
    pub version: i64,
    pub name: String,
    pub description: Option<String>,
    pub modified_at: String,
    pub is_default: Option<bool>,
    pub railway_models: Vec<RailwayModel>,
}
