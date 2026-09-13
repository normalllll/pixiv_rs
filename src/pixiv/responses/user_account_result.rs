use serde::{Deserialize, Serialize};

use crate::models::LocalUser;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAccountResult {
    #[serde(rename = "access_token")]
    pub access_token: String,
    #[serde(rename = "expires_in")]
    pub expires_in: u64,
    #[serde(rename = "token_type")]
    pub token_type: String,
    pub scope: String,
    #[serde(rename = "refresh_token")]
    pub refresh_token: String,
    pub user: LocalUser,
}
