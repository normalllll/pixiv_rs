use serde::{Deserialize, Serialize};

use super::User;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Live {
    pub id: String,
    #[serde(rename = "create_at", default, skip_serializing_if = "Option::is_none")]
    pub create_at: Option<String>,
    pub owner: NestedUser,
    pub performers: Vec<NestedUser>,
    pub name: String,
    #[serde(rename = "is_single")]
    pub is_single: bool,
    #[serde(rename = "is_adult")]
    pub is_adult: bool,
    #[serde(rename = "is_r15")]
    pub is_r15: bool,
    #[serde(rename = "is_r18")]
    pub is_r18: bool,
    pub publicity: String,
    #[serde(rename = "is_closed", default, skip_serializing_if = "Option::is_none")]
    pub is_closed: Option<bool>,
    pub mode: String,
    pub server: String,
    #[serde(rename = "channel_id")]
    pub channel_id: String,
    #[serde(rename = "is_enabled_mic_input")]
    pub is_enabled_mic_input: bool,
    #[serde(
        rename = "thumbnail_image_url",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub thumbnail_image_url: Option<String>,
    #[serde(rename = "member_count")]
    pub member_count: u64,
    #[serde(rename = "total_audience_count")]
    pub total_audience_count: u64,
    #[serde(rename = "performer_count")]
    pub performer_count: u64,
    #[serde(rename = "is_muted")]
    pub is_muted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NestedUser {
    pub user: User,
}
