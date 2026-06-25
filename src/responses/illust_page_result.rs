use serde::{Deserialize, Serialize};

use crate::models::Illust;

use super::PageList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IllustPageResult {
    pub illusts: Vec<Illust>,
    #[serde(rename = "ranking_illusts", default)]
    pub ranking_illusts: Vec<Illust>,
    #[serde(rename = "next_url", default, skip_serializing_if = "Option::is_none")]
    pub next_url: Option<String>,
}

impl PageList for IllustPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
