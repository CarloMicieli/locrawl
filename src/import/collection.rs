use serde::{Deserialize, Serialize};

use super::railway_model::RailwayModel;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub version: i64,
    pub description: Option<String>,
    pub modified_at: String,
    pub railway_models: Vec<RailwayModel>,
}
