use serde::{Deserialize, Serialize};

use super::{ImageUrls, MetaPage, MetaSinglePage, Tag, User};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Illust {
    pub id: u64,
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "image_urls")]
    pub image_urls: ImageUrls,
    pub caption: String,
    pub restrict: i32,
    pub user: User,
    pub tags: Vec<Tag>,
    pub tools: Vec<String>,
    #[serde(rename = "create_date")]
    pub create_date: String,
    #[serde(rename = "page_count")]
    pub page_count: u64,
    pub width: u64,
    pub height: u64,
    #[serde(rename = "sanity_level")]
    pub sanity_level: i32,
    #[serde(rename = "x_restrict")]
    pub x_restrict: i32,
    #[serde(rename = "meta_single_page")]
    pub meta_single_page: MetaSinglePage,
    #[serde(rename = "meta_pages")]
    pub meta_pages: Vec<MetaPage>,
    #[serde(rename = "total_view")]
    pub total_view: u64,
    #[serde(rename = "total_bookmarks")]
    pub total_bookmarks: u64,
    #[serde(rename = "is_bookmarked")]
    pub is_bookmarked: bool,
    pub visible: bool,
    #[serde(rename = "is_muted")]
    pub is_muted: bool,
    #[serde(
        rename = "total_comments",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub total_comments: Option<u64>,
    /** 0 = no AI, 1 = partial AI, 2 = fully AI */
    pub illust_ai_type: i32,
    #[serde(
        rename = "restriction_attributes",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub restriction_attributes: Option<Vec<String>>,
}

impl Illust {
    pub fn is_r18(&self) -> bool {
        self.tags.iter().any(|tag| tag.name == "R-18")
    }

    pub fn is_ugoira(&self) -> bool {
        self.kind == "ugoira"
    }
}
