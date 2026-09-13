use super::PageList;
use crate::models::{Illust, Novel, User};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelSeriesPageResult {
    pub novel_series_detail: NovelSeriesDetail,
    pub novel_series_first_novel: Option<Novel>,
    pub novel_series_latest_novel: Option<Novel>,
    pub novels: Vec<Novel>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelSeriesDetail {
    pub id: u64,
    pub title: String,
    pub caption: String,
    pub is_original: bool,
    pub is_concluded: bool,
    pub content_count: u64,
    pub total_character_count: u64,
    pub display_text: String,
    pub novel_ai_type: i32,
    pub watchlist_added: bool,
    pub user: User,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IllustSeriesPageResult {
    pub illust_series_detail: IllustSeriesDetail,
    pub illust_series_first_illust: Option<Illust>,
    pub illusts: Vec<Illust>,
    pub next_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IllustSeriesDetail {
    pub create_date: String,
    pub series_work_count: u64,
    pub width: u64,
    pub height: u64,
    pub cover_image_urls: SeriesCoverImageUrls,
    pub watchlist_added: bool,
    pub id: u64,
    pub title: String,
    pub caption: String,
    pub user: User,
}

impl PageList for NovelSeriesPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}
impl PageList for IllustSeriesPageResult {
    fn next_url(&self) -> Option<&str> {
        self.next_url.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeriesCoverImageUrls {
    pub medium: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn novel_series_preserves_metadata_chapters_and_cursor() {
        let page: NovelSeriesPageResult =
            serde_json::from_str(include_str!("../../tests/fixtures/novel_series.json")).unwrap();
        assert_eq!(page.novel_series_detail.id, 1);
        assert_eq!(page.novels.len(), 1);
        assert!(page.novel_series_first_novel.is_some());
        assert!(page.novel_series_latest_novel.is_some());
        assert!(page.next_url().is_some());
        let mut value = serde_json::to_value(&page).unwrap();
        value["next_url"] = serde_json::Value::Null;
        value["novels"] = serde_json::json!([]);
        value["novel_series_first_novel"] = serde_json::Value::Null;
        value["novel_series_latest_novel"] = serde_json::Value::Null;
        let empty: NovelSeriesPageResult = serde_json::from_value(value).unwrap();
        assert!(empty.next_url().is_none());
        assert!(empty.novels.is_empty());
    }
    #[test]
    fn manga_series_preserves_cover_and_optional_comment_count() {
        let page: IllustSeriesPageResult =
            serde_json::from_str(include_str!("../../tests/fixtures/illust_series.json")).unwrap();
        assert_eq!(
            page.illust_series_detail.cover_image_urls.medium,
            "https://example.invalid/image.jpg"
        );
        assert_eq!(page.illusts.len(), 1);
        assert!(page.illusts[0].total_comments.is_none());
        assert_eq!(page.illusts[0].series.as_ref().unwrap().id, Some(1));
        assert!(page.illust_series_first_illust.is_some());
        assert!(page.next_url().is_some());
    }
}
