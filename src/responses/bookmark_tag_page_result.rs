use serde::{Deserialize, Serialize};

use crate::models::BookmarkTag;

use super::PageList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarkTagPageResult {
    #[serde(rename = "bookmark_tags")]
    pub bookmark_tags: Vec<BookmarkTag>,
    #[serde(rename = "next_url", default, skip_serializing_if = "Option::is_none")]
    pub next_url: Option<String>,
}

impl PageList for BookmarkTagPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
