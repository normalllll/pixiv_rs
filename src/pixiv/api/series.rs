use super::*;

impl PixivApi {
    pub async fn get_novel_series_page(
        &self,
        series_id: u64,
    ) -> Result<NovelSeriesPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/novel/series",
            params!([("series_id", series_id), ("filter", "for_android")]),
        )
        .await
    }
    pub async fn get_next_novel_series_page(
        &self,
        url: String,
    ) -> Result<NovelSeriesPageResult, PixivError> {
        self.get_next_page(url).await
    }
    pub async fn get_illust_series_page(
        &self,
        series_id: u64,
    ) -> Result<IllustSeriesPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/series",
            params!([("illust_series_id", series_id), ("filter", "for_android")]),
        )
        .await
    }
    pub async fn get_next_illust_series_page(
        &self,
        url: String,
    ) -> Result<IllustSeriesPageResult, PixivError> {
        self.get_next_page(url).await
    }

    /// The manga or novel series the account follows up.
    pub async fn get_watchlist_page(
        &self,
        is_novel: bool,
    ) -> Result<WatchlistPageResult, PixivError> {
        let kind = if is_novel { "novel" } else { "manga" };
        self.get_json(
            Endpoint::AppApi,
            &format!("/v1/watchlist/{kind}"),
            params!([("filter", "for_android")]),
        )
        .await
    }

    pub async fn get_next_watchlist_page(
        &self,
        url: String,
    ) -> Result<WatchlistPageResult, PixivError> {
        self.get_next_page(url).await
    }

    fn watchlist_path(is_novel: bool, action: &str) -> String {
        let kind = if is_novel { "novel" } else { "manga" };
        format!("/v1/watchlist/{kind}/{action}")
    }

    /// Adds a manga or novel series to the account's follow-up list.
    pub async fn post_watchlist_add(
        &self,
        series_id: u64,
        is_novel: bool,
    ) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            &Self::watchlist_path(is_novel, "add"),
            Vec::new(),
            RequestBody::Form(params!([("series_id", series_id)])),
        )
        .await
    }

    pub async fn post_watchlist_delete(
        &self,
        series_id: u64,
        is_novel: bool,
    ) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            &Self::watchlist_path(is_novel, "delete"),
            Vec::new(),
            RequestBody::Form(params!([("series_id", series_id)])),
        )
        .await
    }
}
