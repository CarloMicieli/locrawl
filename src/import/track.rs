use serde::{Deserialize, Serialize};

use crate::manifest::{TrackInventory, TrackProduct};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackImport {
    #[serde(default)]
    pub products: Vec<TrackProduct>,
    #[serde(default)]
    pub inventories: Vec<TrackInventory>,
}
