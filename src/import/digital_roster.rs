use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DigitalRosterImport {
    pub items: Vec<DigitalRosterItem>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DigitalRosterItem {
    pub railway_model_id: String,
    pub decoder_id: String,
    pub address: i64,
    pub installation_date: String,
}
