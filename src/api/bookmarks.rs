use super::*;

/// Filters for an account's saved works. Private bookmarks require the owning account.
#[derive(Debug, Clone)]
pub struct BookmarkPageOptions {
    pub restrict: Restrict,
    pub tag: Option<String>,
    pub max_bookmark_id: Option<u64>,
}

impl Default for BookmarkPageOptions {
    fn default() -> Self {
        Self {
            restrict: Restrict::Public,
            tag: None,
            max_bookmark_id: None,
        }
    }
}

impl BookmarkPageOptions {
    fn into_query(self, user_id: u64) -> Params {
        let mut query = params!([("user_id", user_id), ("restrict", self.restrict)]);
        push_optional(&mut query, "tag", self.tag);
        push_optional(
            &mut query,
            "max_bookmark_id",
            self.max_bookmark_id.map(|id| id.to_string()),
        );
        query
    }
}

impl PixivApi {
    pub async fn get_user_illust_bookmark_page_with_options(
        &self,
        user_id: u64,
        options: BookmarkPageOptions,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/bookmarks/illust",
            options.into_query(user_id),
        )
        .await
    }

    pub async fn get_user_novel_bookmark_page_with_options(
        &self,
        user_id: u64,
        options: BookmarkPageOptions,
    ) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/bookmarks/novel",
            options.into_query(user_id),
        )
        .await
    }

    pub async fn get_illust_bookmark_detail(
        &self,
        illust_id: u64,
    ) -> Result<WorkBookmarkDetailResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/illust/bookmark/detail",
            params!([("illust_id", illust_id)]),
        )
        .await
    }

    pub async fn get_novel_bookmark_detail(
        &self,
        novel_id: u64,
    ) -> Result<WorkBookmarkDetailResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/novel/bookmark/detail",
            params!([("novel_id", novel_id)]),
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bookmark_filters_preserve_private_scope_and_unicode_tags() {
        let query = BookmarkPageOptions {
            restrict: Restrict::Private,
            tag: Some("猫 & 空白".into()),
            max_bookmark_id: Some(42),
        }
        .into_query(7);
        let url =
            Url::parse_with_params("https://app-api.pixiv.net/v1/user/bookmarks/illust", &query)
                .unwrap();
        let decoded: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(decoded["tag"], "猫 & 空白");
        assert_eq!(decoded["restrict"], "private");
        assert_eq!(decoded["max_bookmark_id"], "42");
        assert_eq!(BookmarkPageOptions::default().into_query(7).len(), 2);
    }
}
