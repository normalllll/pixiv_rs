use super::*;
use crate::pixivision::{SpotlightCategory, SpotlightPage};

impl PixivApi {
    pub async fn get_spotlight_article_page(
        &self,
        category: SpotlightCategory,
    ) -> Result<SpotlightPage, PixivError> {
        let category = match category {
            SpotlightCategory::All => "all",
            SpotlightCategory::Manga => "manga",
        };
        self.get_json(
            Endpoint::AppApi,
            "/v1/spotlight/articles",
            params!([("category", category), ("filter", "for_android")]),
        )
        .await
    }

    pub async fn get_next_spotlight_article_page(
        &self,
        url: String,
    ) -> Result<SpotlightPage, PixivError> {
        let parsed = reqwest::Url::parse(&url)?;
        if parsed.scheme() != "https"
            || parsed.host_str() != Some("app-api.pixiv.net")
            || parsed.path() != "/v1/spotlight/articles"
            || parsed.port_or_known_default() != Some(443)
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            return Err(PixivError::new(
                PixivErrorKind::InvalidEndpoint,
                "Invalid Spotlight pagination URL".into(),
            ));
        }
        self.get_next_page(url).await
    }
}
