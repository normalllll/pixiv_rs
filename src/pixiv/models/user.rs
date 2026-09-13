use serde::{Deserialize, Serialize};

use super::ProfileImageUrls;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub account: String,
    #[serde(rename = "profile_image_urls")]
    pub profile_image_urls: ProfileImageUrls,
    #[serde(
        rename = "is_followed",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_followed: Option<bool>,
    #[serde(
        rename = "is_accept_request",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub is_accept_request: Option<bool>,
}
