use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BookmarkDetailResult {
    #[serde(rename = "is_bookmarked")]
    pub is_bookmarked: bool,
    pub tags: Vec<String>,
    pub restrict: String,
}
