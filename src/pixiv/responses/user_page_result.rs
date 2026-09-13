use serde::{Deserialize, Serialize};

use crate::models::UserPreview;

use super::PageList;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPageResult {
    #[serde(rename = "user_previews")]
    pub user_previews: Vec<UserPreview>,
    #[serde(rename = "next_url", default, skip_serializing_if = "Option::is_none")]
    pub next_url: Option<String>,
}

impl PageList for UserPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
