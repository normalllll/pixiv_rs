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
}
