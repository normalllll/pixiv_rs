use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Japanese,
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Korean,
    Thai,
    Malay,
}

impl Language {
    pub fn path(self) -> &'static str {
        match self {
            Self::Japanese => "ja",
            Self::English => "en",
            Self::SimplifiedChinese => "zh",
            Self::TraditionalChinese => "zh-tw",
            Self::Korean => "ko",
            Self::Thai => "th",
            Self::Malay => "ms",
        }
    }

    pub(crate) fn from_path(path: &str) -> Option<Self> {
        Some(match path {
            "ja" => Self::Japanese,
            "en" => Self::English,
            "zh" => Self::SimplifiedChinese,
            "zh-tw" => Self::TraditionalChinese,
            "ko" => Self::Korean,
            "th" => Self::Thai,
            "ms" => Self::Malay,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
    Illustration,
    Manga,
    Novel,
    Tutorial,
    Making,
    Materials,
    Interview,
    Column,
    News,
}

impl Category {
    pub fn path(self) -> &'static str {
        match self {
            Self::Illustration => "illustration",
            Self::Manga => "manga",
            Self::Novel => "novels",
            Self::Tutorial => "how-to-draw",
            Self::Making => "draw-step-by-step",
            Self::Materials => "textures",
            Self::Interview => "interview",
            Self::Column => "column",
            Self::News => "news",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArticleFeed {
    Latest,
    Category { category: Category },
    Search { keyword: String },
    Tag { tag_id: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleLink {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleSummary {
    pub id: u64,
    pub title: String,
    pub url: String,
    pub thumbnail: Option<String>,
    pub publish_date: Option<String>,
    pub category: Option<ArticleLink>,
    pub tags: Vec<PixivisionTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticlePage {
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub articles: Vec<ArticleSummary>,
    pub next_url: Option<String>,
    pub previous_url: Option<String>,
    pub monthly_ranking: Vec<ArticleSummary>,
    pub recommended: Vec<ArticleSummary>,
    pub categories: Vec<ArticleLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixivisionTag {
    pub id: u64,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagDirectory {
    pub url: String,
    pub groups: Vec<TagGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagGroup {
    pub name: String,
    pub nodes: Vec<TagNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TagNode {
    pub name: String,
    pub tag: Option<PixivisionTag>,
    pub article_count: Option<u64>,
    pub children: Vec<TagNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Article {
    pub id: u64,
    pub url: String,
    pub language: Language,
    pub title: String,
    pub description: Option<String>,
    pub thumbnail: Option<String>,
    pub publish_date: String,
    pub category: Option<ArticleLink>,
    pub tags: Vec<PixivisionTag>,
    pub translations: Vec<ArticleLink>,
    /// Source order is retained, including unknown block types and article cards.
    pub blocks: Vec<ArticleBlock>,
    pub related: Vec<ArticleSection>,
    pub monthly_ranking: Vec<ArticleSummary>,
    pub recommended: Vec<ArticleSummary>,
    pub next_url: Option<String>,
    pub previous_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleSection {
    pub title: String,
    pub url: Option<String>,
    pub articles: Vec<ArticleSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockKind {
    Paragraph,
    Heading,
    Image,
    PixivWork,
    Video,
    Quote,
    List,
    Table,
    Code,
    TableOfContents,
    Profile,
    Question,
    Answer,
    ArticleCard,
    Divider,
    Unknown,
}

/// HTML is sanitized; links and media URLs are absolute. Embeds are data, not
/// executable HTML. The typed assets also support native cards and media viewers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleBlock {
    pub kind: BlockKind,
    pub anchor: Option<String>,
    pub heading_level: Option<u8>,
    pub html: String,
    pub text: String,
    pub images: Vec<ArticleImage>,
    pub links: Vec<ArticleLink>,
    pub works: Vec<FeaturedWork>,
    pub embeds: Vec<ArticleEmbed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleImage {
    pub url: String,
    pub alt: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeaturedWorkKind {
    Illustration,
    Novel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeaturedWork {
    pub id: u64,
    pub kind: FeaturedWorkKind,
    pub title: String,
    pub url: String,
    pub user_id: Option<u64>,
    pub user_name: Option<String>,
    pub preview: Option<String>,
    pub page_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmbedKind {
    Frame,
    Video,
    Audio,
    SocialPost,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArticleEmbed {
    pub kind: EmbedKind,
    pub url: String,
    pub title: Option<String>,
    pub poster: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotlightCategory {
    All,
    Manga,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpotlightPage {
    pub spotlight_articles: Vec<SpotlightArticle>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpotlightArticle {
    pub id: u64,
    pub title: String,
    pub pure_title: Option<String>,
    pub thumbnail: String,
    pub article_url: String,
    pub publish_date: String,
    pub category: String,
    pub subcategory_label: Option<String>,
}

impl crate::responses::PageList for SpotlightPage {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
