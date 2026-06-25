use serde::{Deserialize, Serialize};

use crate::models::ProfileImageUrls;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserDetailResult {
    pub user: UserInfo,
    pub profile: UserProfile,
    #[serde(rename = "profile_publicity")]
    pub profile_publicity: UserProfilePublicity,
    pub workspace: UserWorkspace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: u64,
    pub name: String,
    pub account: String,
    #[serde(rename = "profile_image_urls")]
    pub profile_image_urls: ProfileImageUrls,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "is_followed")]
    pub is_followed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webpage: Option<String>,
    pub gender: String,
    pub birth: String,
    #[serde(rename = "birth_day")]
    pub birth_day: String,
    #[serde(rename = "birth_year")]
    pub birth_year: u64,
    pub region: String,
    #[serde(rename = "address_id")]
    pub address_id: u64,
    #[serde(rename = "country_code")]
    pub country_code: String,
    pub job: String,
    #[serde(rename = "job_id")]
    pub job_id: u64,
    #[serde(rename = "total_follow_users")]
    pub total_follow_users: u64,
    #[serde(rename = "total_mypixiv_users")]
    pub total_mypixiv_users: u64,
    #[serde(rename = "total_illusts")]
    pub total_illusts: u64,
    #[serde(rename = "total_manga")]
    pub total_manga: u64,
    #[serde(rename = "total_novels")]
    pub total_novels: u64,
    #[serde(rename = "total_illust_bookmarks_public")]
    pub total_illust_bookmarks_public: u64,
    #[serde(rename = "total_illust_series")]
    pub total_illust_series: u64,
    #[serde(rename = "total_novel_series")]
    pub total_novel_series: u64,
    #[serde(
        rename = "background_image_url",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub background_image_url: Option<String>,
    #[serde(
        rename = "twitter_account",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub twitter_account: Option<String>,
    #[serde(
        rename = "twitter_url",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub twitter_url: Option<String>,
    #[serde(rename = "pawoo_url", default, skip_serializing_if = "Option::is_none")]
    pub pawoo_url: Option<String>,
    #[serde(rename = "is_premium")]
    pub is_premium: bool,
    #[serde(rename = "is_using_custom_profile_image")]
    pub is_using_custom_profile_image: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserProfilePublicity {
    pub gender: String,
    pub region: String,
    #[serde(rename = "birth_day")]
    pub birth_day: String,
    #[serde(rename = "birth_year")]
    pub birth_year: String,
    pub job: String,
    pub pawoo: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserWorkspace {
    pub pc: String,
    pub monitor: String,
    pub tool: String,
    pub scanner: String,
    pub tablet: String,
    pub mouse: String,
    pub printer: String,
    pub desktop: String,
    pub music: String,
    pub desk: String,
    pub chair: String,
    pub comment: String,
    #[serde(
        rename = "workspace_image_url",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub workspace_image_url: Option<String>,
}
