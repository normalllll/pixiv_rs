use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub enum FanboxFeed {
    Home,
    Supporting,
    Creator {
        creator_id: String,
    },
    Tag {
        tag: String,
        creator_id: Option<String>,
        page: u32,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum FanboxCreatorList {
    Following,
    Recommended,
    Pixiv,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxUser {
    pub id: String,
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxCreator {
    pub creator_id: String,
    pub user: FanboxUser,
    pub description: String,
    pub cover_url: Option<String>,
    pub is_followed: bool,
    pub is_supported: bool,
    pub has_adult_content: bool,
    pub profile_links: Vec<String>,
    pub profile_images: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxPost {
    pub id: String,
    pub creator_id: String,
    pub user: FanboxUser,
    pub title: String,
    pub excerpt: String,
    pub cover_url: Option<String>,
    pub published_datetime: String,
    pub updated_datetime: String,
    pub fee_required: u64,
    pub is_restricted: bool,
    pub is_liked: bool,
    pub like_count: u64,
    pub comment_count: u64,
    pub tags: Vec<String>,
    pub blocks: Vec<FanboxBlock>,
    pub previous_post_id: Option<String>,
    pub next_post_id: Option<String>,
    /// Retained for future post types; never interpret it as executable content.
    pub unknown_body_json: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FanboxBlock {
    Paragraph {
        text: String,
        spans: Vec<FanboxTextSpan>,
    },
    Heading {
        text: String,
    },
    Image {
        image: FanboxImage,
    },
    File {
        file: FanboxFile,
    },
    Embed {
        url: Option<String>,
        html: Option<String>,
    },
    PostLink {
        post_id: String,
        creator_id: String,
        title: String,
    },
    Unknown {
        text: String,
        raw_json: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FanboxTextSpan {
    pub offset: u32,
    pub length: u32,
    pub bold: bool,
    pub italic: bool,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxImage {
    pub id: String,
    pub original_url: String,
    pub thumbnail_url: String,
    pub width: u32,
    pub height: u32,
    pub extension: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxFile {
    pub id: String,
    pub name: String,
    pub extension: String,
    pub size: u64,
    pub url: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxPostPage {
    pub posts: Vec<FanboxPost>,
    pub next_url: Option<String>,
    pub next_page: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxCreatorPage {
    pub creators: Vec<FanboxCreator>,
    pub next_page: Option<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxPlan {
    pub id: String,
    pub creator_id: String,
    pub user: FanboxUser,
    pub title: String,
    pub description: String,
    pub fee: u64,
    pub cover_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxTag {
    pub name: String,
    pub count: Option<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxComment {
    pub id: String,
    pub user: FanboxUser,
    pub body: String,
    pub created_datetime: String,
    pub is_liked: bool,
    pub is_own: bool,
    pub like_count: u64,
    pub replies: Vec<FanboxComment>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxCommentPage {
    pub comments: Vec<FanboxComment>,
    pub next_url: Option<String>,
    pub can_comment: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxNotice {
    pub id: String,
    pub kind: String,
    pub user_name: String,
    pub title: String,
    pub body: String,
    pub date: String,
    pub is_unread: bool,
    pub post_id: Option<String>,
    pub creator_id: Option<String>,
    pub icon_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxNoticePage {
    pub notices: Vec<FanboxNotice>,
    pub next_url: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FanboxSupport {
    pub creator_id: String,
    pub fan_card_url: Option<String>,
    pub started_datetime: Option<String>,
}
