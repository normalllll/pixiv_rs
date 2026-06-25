use serde::{Deserialize, Serialize};

use crate::models::Comment;

use super::PageList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentPageResult {
    pub comments: Vec<Comment>,
    #[serde(rename = "next_url", default, skip_serializing_if = "Option::is_none")]
    pub next_url: Option<String>,
}

impl PageList for CommentPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
