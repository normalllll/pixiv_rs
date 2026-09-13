use serde::{Deserialize, Serialize};

use crate::models::Illust;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendingTagListResult {
    #[serde(rename = "trend_tags")]
    pub trend_tags: Vec<TrendTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrendTag {
    pub tag: String,
    #[serde(
        rename = "translated_name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub translated_name: Option<String>,
    pub illust: Illust,
}
