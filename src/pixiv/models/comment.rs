use serde::{Deserialize, Serialize};

use super::{Stamp, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub comment: String,
    pub date: String,
    pub user: User,
    #[serde(rename = "has_replies")]
    pub has_replies: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stamp: Option<Stamp>,
}
