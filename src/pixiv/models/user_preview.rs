use serde::{Deserialize, Serialize};

use super::{Illust, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserPreview {
    pub user: User,
    pub illusts: Vec<Illust>,
    #[serde(rename = "is_muted")]
    pub is_muted: bool,
}
