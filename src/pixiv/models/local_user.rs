use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalUser {
    #[serde(rename = "profile_image_urls")]
    pub profile_image_urls: LocalUserProfileImageUrls,
    pub id: String,
    pub name: String,
    pub account: String,
    #[serde(rename = "mail_address")]
    pub mail_address: String,
    #[serde(rename = "is_premium")]
    pub is_premium: bool,
    #[serde(rename = "x_restrict")]
    pub x_restrict: i32,
    #[serde(rename = "is_mail_authorized")]
    pub is_mail_authorized: bool,
    #[serde(rename = "require_policy_agreement")]
    pub require_policy_agreement: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalUserProfileImageUrls {
    #[serde(rename = "px_16x16")]
    pub px_16x16: String,
    #[serde(rename = "px_50x50")]
    pub px_50x50: String,
    #[serde(rename = "px_170x170")]
    pub px_170x170: String,
}
