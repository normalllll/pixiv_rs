use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stamp {
    #[serde(rename = "stamp_id")]
    pub stamp_id: u64,
    #[serde(rename = "stamp_url")]
    pub stamp_url: String,
}
