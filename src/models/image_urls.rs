use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageUrls {
    #[serde(rename = "square_medium")]
    pub square_medium: String,
    pub medium: String,
    pub large: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
}
