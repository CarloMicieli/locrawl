use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedRollingStock {
    pub id: String,
    pub collection_item_id: String,
    pub rolling_stock_id: Option<String>,
    pub notes: Option<String>,
    pub dcc_address: Option<i64>,
    pub installed_decoder_id: Option<String>,
    pub current_coupler_id: Option<String>,
}
