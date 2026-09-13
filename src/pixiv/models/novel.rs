use serde::{Deserialize, Serialize};

use super::{ImageUrls, Tag, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Novel {
    pub id: u64,
    pub title: String,
    pub caption: String,
    pub restrict: i32,
    #[serde(rename = "x_restrict")]
    pub x_restrict: i32,
    #[serde(rename = "is_original")]
    pub is_original: bool,
    #[serde(rename = "image_urls")]
    pub image_urls: ImageUrls,
    #[serde(rename = "create_date")]
    pub create_date: String,
    pub tags: Vec<Tag>,
    #[serde(rename = "page_count")]
    pub page_count: u64,
    #[serde(rename = "text_length")]
    pub text_length: u64,
    pub user: User,
    pub series: Series,
    #[serde(rename = "total_bookmarks")]
    pub total_bookmarks: u64,
    #[serde(rename = "is_bookmarked")]
    pub is_bookmarked: bool,
    #[serde(rename = "total_view")]
    pub total_view: u64,
    pub visible: bool,
    #[serde(rename = "total_comments")]
    pub total_comments: u64,
    #[serde(rename = "is_muted")]
    pub is_muted: bool,
    #[serde(rename = "is_mypixiv_only")]
    pub is_mypixiv_only: bool,
    #[serde(rename = "is_x_restricted")]
    pub is_x_restricted: bool,
    /** 0 = no AI, 1 = partial AI, 2 = fully AI */
    #[serde(rename = "novel_ai_type")]
    pub novel_ai_type: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Series {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}
