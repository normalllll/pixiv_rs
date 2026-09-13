use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchAiMode {
    Hide,
    Show,
}

pub(super) fn push_search_ai(query: &mut Params, mode: Option<SearchAiMode>) {
    if let Some(mode) = mode {
        let value = match mode {
            SearchAiMode::Hide => "0",
            SearchAiMode::Show => "1",
        };
        query.push(("search_ai_type".into(), value.into()));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hiding_ai_sends_zero_instead_of_omitting_filter() {
        let mut query = Vec::new();
        push_search_ai(&mut query, None);
        assert!(query.is_empty());
        push_search_ai(&mut query, Some(SearchAiMode::Hide));
        assert_eq!(query, vec![("search_ai_type".into(), "0".into())]);
        query.clear();
        push_search_ai(&mut query, Some(SearchAiMode::Show));
        assert_eq!(query[0].1, "1");
    }
}
