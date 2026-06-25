use serde::{Deserialize, Serialize};

use crate::models::Illust;

use super::PageList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchIllustPageResult {
    pub illusts: Vec<Illust>,
    #[serde(rename = "next_url", default, skip_serializing_if = "Option::is_none")]
    pub next_url: Option<String>,
    #[serde(rename = "search_span_limit")]
    pub search_span_limit: u64,
}

impl PageList for SearchIllustPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
