use serde::{Deserialize, Serialize};
use crate::models::Novel;

use super::PageList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelPageResult {
    pub novels: Vec<Novel>,
    #[serde(rename = "ranking_novels", default)]
    pub ranking_novels: Vec<Novel>,
    #[serde(
        rename = "nextUrl",
        alias = "next_url",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub next_url: Option<String>,
}

impl PageList for NovelPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
