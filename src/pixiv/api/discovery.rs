use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchAiMode {
    Hide,
    Show,
}

fn ranking_query(mode: String, date: String) -> Params {
    params!([("filter", "for_android"), ("mode", mode), ("date", date)])
}

impl PixivApi {
    /// Historical ranking date in YYYY-MM-DD format, subject to server availability.
    pub async fn get_illust_ranking_page_on_date(
        &self,
        mode: IllustRankingMode,
        date: String,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/ranking",
            ranking_query(mode.to_string(), date),
        )
        .await
    }
    pub async fn get_manga_ranking_page_on_date(
        &self,
        mode: MangaRankingMode,
        date: String,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/ranking",
            ranking_query(mode.to_string(), date),
        )
        .await
    }
    pub async fn get_novel_ranking_page_on_date(
        &self,
        mode: NovelRankingMode,
        date: String,
    ) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/novel/ranking",
            ranking_query(mode.to_string(), date),
        )
        .await
    }
    pub async fn get_mypixiv_user_page(&self, user_id: u64) -> Result<UserPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/mypixiv",
            params!([("user_id", user_id)]),
        )
        .await
    }
    pub async fn get_ai_show_settings(&self) -> Result<AiShowSettingsResult, PixivError> {
        self.get_json(Endpoint::AppApi, "/v1/user/ai-show-settings", Vec::new())
            .await
    }

    /// Updates the account-wide AI visibility preference on Pixiv.
    pub async fn post_ai_show_settings(&self, mode: SearchAiMode) -> Result<String, PixivError> {
        let show = match mode {
            SearchAiMode::Hide => "false",
            SearchAiMode::Show => "true",
        };
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/user/ai-show-settings/edit",
            Vec::new(),
            RequestBody::Form(params!([("show_ai", show)])),
        )
        .await
    }
}
