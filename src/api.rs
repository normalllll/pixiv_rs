use reqwest::header::{ACCEPT_LANGUAGE, AUTHORIZATION, HOST, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Method, Proxy, StatusCode, Url};
use serde::de::DeserializeOwned;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::auth::{PixivAuth, PixivAuthConfig};
use crate::enums::{
    IllustRankingMode, IllustType, PixivEnumParam, Restrict, SearchSort, SearchTarget,
};
use crate::error::{PixivError, PixivErrorKind};
use crate::responses::*;
use crate::{MangaRankingMode, NovelRankingMode};

type Params = Vec<(String, String)>;

macro_rules! params {
    ([$(($key:expr, $value:expr)),* $(,)?]) => {
        vec![$(($key.to_string(), $value.to_string())),*]
    };
}

const APP_API_HOST: &str = "app-api.pixiv.net";
const SKETCH_HOST: &str = "sketch.pixiv.net";
const SKETCH_IP: &str = "210.140.170.179";
const API_USER_AGENT: &str = "PixivAndroidApp/6.184.0 (Android 15.0; {device})";
const REFRESH_TOKEN_MIN_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PixivApiConfig {
    pub device_name: String,
    pub target_ip: String,
    pub language: String,
    pub account: Option<UserAccountResult>,
    pub accept_invalid_certs: bool,
    pub proxy: Option<String>,
}

impl PixivApiConfig {
    pub fn new(
        target_ip: String,
        language: String,
        device_name: String,
        account: Option<UserAccountResult>,
        accept_invalid_certs: bool,
    ) -> Self {
        Self {
            device_name,
            target_ip,
            language,
            account,
            accept_invalid_certs,
            proxy: None,
        }
    }
}

#[derive(Clone)]
pub struct PixivApi {
    config: PixivApiConfig,
    auth: PixivAuth,
    account: Arc<RwLock<Option<UserAccountResult>>>,
    refresh_token_last_time: Arc<Mutex<Option<Instant>>>,
}

impl PixivApi {
    pub fn new(config: PixivApiConfig) -> Self {
        let auth = PixivAuth::new(PixivAuthConfig {
            target_ip: config.target_ip.clone(),
            language: config.language.clone(),
            device_name: config.device_name.clone(),
            accept_invalid_certs: config.accept_invalid_certs,
            proxy: config.proxy.clone(),
        });
        let account = Arc::new(RwLock::new(config.account.clone()));

        Self {
            config,
            auth,
            account,
            refresh_token_last_time: Arc::new(Mutex::new(None)),
        }
    }

    pub fn account(&self) -> Option<UserAccountResult> {
        self.account
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn set_account(&self, account: Option<UserAccountResult>) {
        *self
            .account
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = account;
    }

    pub fn set_proxy(&mut self, proxy: String) {
        self.config.proxy = Some(proxy.clone());
        self.auth.set_proxy(proxy);
    }

    pub async fn get_next_comment_page(
        &self,
        url: String,
    ) -> Result<CommentPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_next_illust_page(&self, url: String) -> Result<IllustPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_next_novel_page(&self, url: String) -> Result<NovelPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_next_user_page(&self, url: String) -> Result<UserPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_next_search_illust_page(
        &self,
        url: String,
    ) -> Result<SearchIllustPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_next_search_novel_page(
        &self,
        url: String,
    ) -> Result<SearchNovelPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_next_bookmark_tag_page(
        &self,
        url: String,
    ) -> Result<BookmarkTagPageResult, PixivError> {
        self.get_next_page(url).await
    }

    pub async fn get_user_detail(&self, user_id: u64) -> Result<UserDetailResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/detail",
            params!([("filter", "for_android"), ("user_id", user_id.to_string())]),
        )
        .await
    }

    pub async fn get_user_illust_bookmark_page(
        &self,
        user_id: u64,
        restrict: Restrict,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/bookmarks/illust",
            params!([
                ("user_id", user_id.to_string()),
                ("restrict", restrict.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_user_novel_bookmark_page(
        &self,
        user_id: u64,
        restrict: Restrict,
    ) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/bookmarks/novel",
            params!([
                ("user_id", user_id.to_string()),
                ("restrict", restrict.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_user_illust_page(
        &self,
        user_id: u64,
        illust_type: IllustType,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/illusts",
            params!([
                ("filter", "for_android"),
                ("user_id", user_id.to_string()),
                ("type", illust_type.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_user_novel_page(&self, user_id: u64) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/novels",
            params!([("user_id", user_id.to_string())]),
        )
        .await
    }

    pub async fn get_recommended_illust_page(
        &self,
        illust_type: IllustType,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            &format!("/v1/{}/recommended", illust_type.as_pixiv_param()),
            params!([
                ("filter", "for_ios"),
                ("include_ranking_illusts", true.to_string()),
                ("include_privacy_policy", true.to_string()),
            ]),
        )
        .await
    }
    pub async fn get_recommended_novel_page(&self) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/novel/recommended",
            params!([
                ("include_ranking_novels", true.to_string()),
                ("include_privacy_policy", true.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_illust_ranking_page(
        &self,
        mode: IllustRankingMode,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/ranking",
            params!([("filter", "for_android"), ("mode", mode.to_string())]),
        )
        .await
    }

    pub async fn get_manga_ranking_page(
        &self,
        mode: MangaRankingMode,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/ranking",
            params!([("filter", "for_android"), ("mode", mode.to_string())]),
        )
        .await
    }

    pub async fn get_novel_ranking_page(
        &self,
        mode: NovelRankingMode,
    ) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/novel/ranking",
            params!([("filter", "for_android"), ("mode", mode.to_string())]),
        )
        .await
    }

    pub async fn get_trending_tag_list(&self) -> Result<TrendingTagListResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/trending-tags/illust",
            params!([("filter", "for_android")]),
        )
        .await
    }

    pub async fn get_recommended_user_page(&self) -> Result<UserPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/recommended",
            params!([("filter", "for_android")]),
        )
        .await
    }

    pub async fn get_follower_page(&self, user_id: u64) -> Result<UserPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/follower",
            params!([("filter", "for_android"), ("user_id", user_id.to_string())]),
        )
        .await
    }

    pub async fn get_following_user_page(
        &self,
        user_id: u64,
        restrict: Restrict,
    ) -> Result<UserPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/following",
            params!([
                ("filter", "for_android"),
                ("user_id", user_id.to_string()),
                ("restrict", restrict.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_follow_new_illust_page(
        &self,
        restrict: Option<Restrict>,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/illust/follow",
            params!([
                ("filter", "for_android"),
                ("restrict", restrict_param(restrict)),
            ]),
        )
        .await
    }

    pub async fn get_follow_new_novel_page(
        &self,
        restrict: Option<Restrict>,
    ) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/novel/follow",
            params!([
                ("filter", "for_android"),
                ("restrict", restrict_param(restrict)),
            ]),
        )
        .await
    }

    pub async fn get_mypixiv_new_illust_page(&self) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/illust/mypixiv",
            params!([("filter", "for_android"),]),
        )
        .await
    }

    pub async fn get_mypixiv_new_novel_page(&self) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/novel/mypixiv",
            params!([("filter", "for_android"),]),
        )
        .await
    }

    pub async fn get_new_illust_page(
        &self,
        illust_type: IllustType,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/new",
            params!([
                ("filter", "for_android"),
                ("content_type", illust_type.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_new_novel_page(&self) -> Result<NovelPageResult, PixivError> {
        self.get_json(Endpoint::AppApi, "/v1/novel/new", Vec::new())
            .await
    }

    pub async fn get_illust_related_page(
        &self,
        illust_id: u64,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/illust/related",
            params!([
                ("filter", "for_android"),
                ("illust_id", illust_id.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_novel_related_page(
        &self,
        novel_id: u64,
    ) -> Result<NovelPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/novel/related",
            params!([
                ("filter", "for_android"),
                ("novel_id", novel_id.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_user_related_page(
        &self,
        offset: i32,
        seed_user_id: u64,
    ) -> Result<IllustPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/user/related",
            params!([
                ("filter", "for_android"),
                ("offset", offset.to_string()),
                ("seed_user_id", seed_user_id.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_illust_detail(
        &self,
        illust_id: u64,
    ) -> Result<IllustDetailResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/illust/detail",
            params!([
                ("filter", "for_android"),
                ("illust_id", illust_id.to_string()),
            ]),
        )
        .await
    }

    pub async fn get_webview_novel(&self, novel_id: u64) -> Result<WebviewNovel, PixivError> {
        let html = self.get_novel_html(novel_id).await?;

        let json_str =
            extract_json_object_after_key(&html, "novel:").ok_or_else(|| PixivError {
                kind: PixivErrorKind::Json,
                message: "extract novel object failed".to_owned(),
                status: None,
                body: Some(html.to_owned()),
            })?;

        serde_json::from_str::<WebviewNovel>(json_str).map_err(|err| PixivError {
            kind: PixivErrorKind::Json,
            message: format!("parse novel object failed: {err}"),
            status: None,
            body: Some(json_str.to_owned()),
        })
    }

    pub async fn get_novel_html(&self, novel_id: u64) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::GET,
            "/webview/v2/novel",
            params!([
                ("id", novel_id.to_string()),
                ("viewer_version", "20221031_ai")
            ]),
            RequestBody::None,
        )
        .await
    }

    pub async fn get_novel_detail(&self, novel_id: u64) -> Result<NovelDetailResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/novel/detail",
            params!([("novel_id", novel_id.to_string()),]),
        )
        .await
    }

    pub async fn get_ugoira_metadata(
        &self,
        illust_id: u64,
    ) -> Result<UgoiraMetadataResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/ugoira/metadata",
            params!([("illust_id", illust_id.to_string())]),
        )
        .await
    }

    pub async fn get_illust_comment_reply_page(
        &self,
        comment_id: u64,
    ) -> Result<CommentPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/illust/comment/replies",
            params!([("comment_id", comment_id.to_string())]),
        )
        .await
    }

    pub async fn get_novel_comment_reply_page(
        &self,
        comment_id: u64,
    ) -> Result<CommentPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/novel/comment/replies",
            params!([("comment_id", comment_id.to_string())]),
        )
        .await
    }

    pub async fn get_illust_comment_page(
        &self,
        illust_id: u64,
    ) -> Result<CommentPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v3/illust/comments",
            params!([("illust_id", illust_id.to_string())]),
        )
        .await
    }

    pub async fn get_novel_comment_page(
        &self,
        novel_id: u64,
    ) -> Result<CommentPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v3/novel/comments",
            params!([("novel_id", novel_id.to_string())]),
        )
        .await
    }

    pub async fn get_search_autocomplete(
        &self,
        word: String,
    ) -> Result<SearchAutocompleteResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v2/search/autocomplete",
            params!([
                ("merge_plain_keyword_results", true.to_string()),
                ("word", word.to_owned()),
            ]),
        )
        .await
    }

    pub async fn get_search_illust_page(
        &self,
        word: String,
        sort: SearchSort,
        target: SearchTarget,
        options: SearchOptions,
    ) -> Result<SearchIllustPageResult, PixivError> {
        let mut query = params!([
            ("filter", "for_android"),
            ("include_translated_tag_results", true.to_string()),
            ("merge_plain_keyword_results", true.to_string()),
            ("word", search_word(&word, options.bookmark_total)),
            ("sort", sort.to_string()),
            ("search_target", target.to_string()),
        ]);
        push_optional(&mut query, "start_date", options.start_date);
        push_optional(&mut query, "end_date", options.end_date);

        self.get_json(Endpoint::AppApi, "/v1/search/illust", query)
            .await
    }

    pub async fn get_search_novel_page(
        &self,
        word: String,
        sort: SearchSort,
        target: SearchTarget,
        options: SearchOptions,
    ) -> Result<SearchNovelPageResult, PixivError> {
        let mut query = params!([
            ("include_translated_tag_results", true.to_string()),
            ("merge_plain_keyword_results", true.to_string()),
            ("word", search_word(&word, options.bookmark_total)),
            ("sort", sort.to_string()),
            ("search_target", target.to_string()),
        ]);
        push_optional(&mut query, "start_date", options.start_date);
        push_optional(&mut query, "end_date", options.end_date);

        self.get_json(Endpoint::AppApi, "/v1/search/novel", query)
            .await
    }

    pub async fn get_search_user_page(&self, word: String) -> Result<UserPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            "/v1/search/user",
            params!([("filter", "for_android"), ("word", word)]),
        )
        .await
    }

    pub async fn get_bookmark_tag_page(
        &self,
        user_id: u64,
        options: BookmarkTagOptions,
    ) -> Result<BookmarkTagPageResult, PixivError> {
        self.get_json(
            Endpoint::AppApi,
            &format!(
                "/v1/user/bookmark-tags/{}",
                if options.is_novel { "novel" } else { "illust" }
            ),
            params!([
                ("user_id", user_id.to_string()),
                ("restrict", options.restrict.to_string()),
            ]),
        )
        .await
    }

    pub async fn post_bookmark_add(
        &self,
        id: u64,
        options: BookmarkAddOptions,
    ) -> Result<String, PixivError> {
        let work = if options.is_novel { "novel" } else { "illust" };
        let mut form = params!([
            (format!("{work}_id"), id.to_string()),
            ("restrict", options.restrict.to_string()),
        ]);
        form.extend(options.tags.into_iter().map(|tag| ("tags".to_owned(), tag)));

        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            &format!("/v2/{work}/bookmark/add"),
            Vec::new(),
            RequestBody::Form(form),
        )
        .await
    }

    pub async fn post_bookmark_delete(
        &self,
        id: u64,
        is_novel: bool,
    ) -> Result<String, PixivError> {
        let work = if is_novel { "novel" } else { "illust" };
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            &format!("/v1/{work}/bookmark/delete"),
            Vec::new(),
            RequestBody::Form(params!([(format!("{work}_id"), id.to_string())])),
        )
        .await
    }

    pub async fn post_follow_add(
        &self,
        user_id: u64,
        restrict: Restrict,
    ) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/user/follow/add",
            Vec::new(),
            RequestBody::Form(params!([
                ("user_id", user_id.to_string()),
                ("restrict", restrict.to_string()),
            ])),
        )
        .await
    }

    pub async fn post_follow_delete(&self, user_id: u64) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/user/follow/delete",
            Vec::new(),
            RequestBody::Form(params!([("user_id", user_id.to_string())])),
        )
        .await
    }

    pub async fn post_illust_comment_add(
        &self,
        illust_id: u64,
        options: CommentAddOptions,
    ) -> Result<CommentAddResult, PixivError> {
        let mut form = params!([
            ("illust_id", illust_id.to_string()),
            ("comment", options.comment),
        ]);
        push_optional(
            &mut form,
            "stamp_id",
            options.stamp_id.map(|id| id.to_string()),
        );
        push_optional(
            &mut form,
            "parent_comment_id",
            options.parent_comment_id.map(|id| id.to_string()),
        );

        self.post_json(
            Endpoint::AppApi,
            "/v1/illust/comment/add",
            RequestBody::Form(form),
        )
        .await
    }

    pub async fn post_illust_comment_delete(&self, comment_id: u64) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/illust/comment/delete",
            Vec::new(),
            RequestBody::Form(params!([("comment_id", comment_id.to_string())])),
        )
        .await
    }

    pub async fn post_novel_comment_add(
        &self,
        novel_id: u64,
        options: CommentAddOptions,
    ) -> Result<CommentAddResult, PixivError> {
        let mut form = params!([
            ("novel_id", novel_id.to_string()),
            ("comment", options.comment),
        ]);
        push_optional(
            &mut form,
            "stamp_id",
            options.stamp_id.map(|id| id.to_string()),
        );
        push_optional(
            &mut form,
            "parent_comment_id",
            options.parent_comment_id.map(|id| id.to_string()),
        );

        self.post_json(
            Endpoint::AppApi,
            "/v1/novel/comment/add",
            RequestBody::Form(form),
        )
        .await
    }

    pub async fn post_novel_comment_delete(&self, comment_id: u64) -> Result<String, PixivError> {
        self.request_text(
            Endpoint::AppApi,
            Method::POST,
            "/v1/novel/comment/delete",
            Vec::new(),
            RequestBody::Form(params!([("comment_id", comment_id.to_string())])),
        )
        .await
    }

    async fn get_next_page<T>(&self, url: String) -> Result<T, PixivError>
    where
        T: PageList + DeserializeOwned,
    {
        let page: T = self.get_json(Endpoint::AppApi, &url, Vec::new()).await?;
        let _ = page.next_url();
        Ok(page)
    }

    async fn get_json<T>(
        &self,
        endpoint: Endpoint,
        path: &str,
        query: Params,
    ) -> Result<T, PixivError>
    where
        T: DeserializeOwned,
    {
        let text = self
            .request_text(endpoint, Method::GET, path, query, RequestBody::None)
            .await?;
        Ok(serde_json::from_str(&text)?)
    }

    async fn post_json<T>(
        &self,
        endpoint: Endpoint,
        path: &str,
        body: RequestBody,
    ) -> Result<T, PixivError>
    where
        T: DeserializeOwned,
    {
        let text = self
            .request_text(endpoint, Method::POST, path, Vec::new(), body)
            .await?;
        Ok(serde_json::from_str(&text)?)
    }

    async fn request_text(
        &self,
        endpoint: Endpoint,
        method: Method,
        path: &str,
        query: Params,
        body: RequestBody,
    ) -> Result<String, PixivError> {
        let request = self.build_request(endpoint, method, path, query, body)?;
        let mut response = self.send_with_retry(&request).await?;

        if response.status.is_success() {
            return Ok(response.body);
        }

        if response.status == StatusCode::BAD_REQUEST && is_oauth_error(&response.body) {
            self.refresh_auth_token_if_needed().await?;
            response = self.send_with_retry(&request).await?;
            if response.status.is_success() {
                return Ok(response.body);
            }
        }

        Err(http_status_error(response.status, response.body))
    }

    fn build_request(
        &self,
        endpoint: Endpoint,
        method: Method,
        path: &str,
        query: Params,
        body: RequestBody,
    ) -> Result<ApiRequest, PixivError> {
        let connect_target = endpoint.connect_target(self);
        let logical_host = endpoint.logical_host();
        let resolve_addr = resolve_addr(&connect_target);
        let mut url = endpoint_url(logical_host, path)?;
        let mut host_header = None;

        if resolve_addr.is_none() && connect_target != logical_host {
            url.set_host(Some(&connect_target)).map_err(|_| {
                PixivError::new(PixivErrorKind::InvalidEndpoint, connect_target.clone())
            })?;
            host_header = Some(logical_host.to_owned());
        }

        let client = self.client(logical_host, resolve_addr)?;
        Ok(ApiRequest {
            client,
            method,
            url,
            query,
            body,
            host_header,
        })
    }

    fn client(
        &self,
        logical_host: &str,
        resolve_addr: Option<SocketAddr>,
    ) -> Result<Client, PixivError> {
        let mut builder = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(5))
            .danger_accept_invalid_certs(self.config.accept_invalid_certs);

        if let Some(addr) = resolve_addr {
            builder = builder.resolve(logical_host, addr);
        }

        if let Some(proxy) = &self.config.proxy {
            builder = builder.proxy(Proxy::all(proxy)?);
        }

        Ok(builder.build()?)
    }

    fn headers(&self, host_header: Option<&str>) -> Result<HeaderMap, PixivError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&API_USER_AGENT.replace("{device}", &self.config.device_name))?,
        );
        headers.insert("App-OS", HeaderValue::from_static("android"));
        headers.insert("App-OS-Version", HeaderValue::from_static("11.0"));
        headers.insert("App-Version", HeaderValue::from_static("6.54.0"));
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_str(&self.config.language)?,
        );

        if let Some(host) = host_header {
            headers.insert(HOST, HeaderValue::from_str(host)?);
        }

        if let Some(account) = self.account() {
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", account.access_token))?,
            );
        }

        Ok(headers)
    }

    async fn send_with_retry(&self, request: &ApiRequest) -> Result<RawResponse, PixivError> {
        let mut last_error = None;
        for attempt in 0..=2 {
            match self.send_once(request).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    if error.kind == PixivErrorKind::HttpClient
                        && is_retryable_error_message(&error.message)
                        && attempt < 2
                    {
                        last_error = Some(error);
                    } else {
                        return Err(error);
                    }
                }
            }
        }

        Err(last_error.expect("retry loop always records an error"))
    }

    async fn send_once(&self, request: &ApiRequest) -> Result<RawResponse, PixivError> {
        let mut builder = request
            .client
            .request(request.method.clone(), request.url.clone())
            .headers(self.headers(request.host_header.as_deref())?)
            .query(&request.query);

        match &request.body {
            RequestBody::None => {}
            RequestBody::Form(form) => {
                builder = builder.form(form);
            }
        }

        let response = builder.send().await?;
        let status = response.status();
        let body = response.text().await?;
        Ok(RawResponse { status, body })
    }

    async fn refresh_auth_token_if_needed(&self) -> Result<(), PixivError> {
        let mut last_refresh = self.refresh_token_last_time.lock().await;
        let should_refresh = last_refresh
            .map(|last| last.elapsed() > REFRESH_TOKEN_MIN_INTERVAL)
            .unwrap_or(true);

        if !should_refresh {
            return Ok(());
        }

        let account = self.account().ok_or_else(PixivError::missing_account)?;
        let refreshed = self.auth.refresh_auth_token(account.refresh_token).await?;
        self.set_account(Some(refreshed));
        *last_refresh = Some(Instant::now());
        Ok(())
    }

    pub async fn init_account_auth_token(
        &self,
        code: String,
    ) -> Result<UserAccountResult, PixivError> {
        self.auth.init_account_auth_token(code).await
    }

    pub fn generate_login_url(&self) -> String {
        self.auth.generate_login_url()
    }
}

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub bookmark_total: Option<u64>,
}

impl SearchOptions {
    pub fn date_range(mut self, start_date: String, end_date: String) -> Self {
        self.start_date = Some(start_date);
        self.end_date = Some(end_date);
        self
    }

    pub fn bookmark_total(mut self, bookmark_total: u64) -> Self {
        self.bookmark_total = Some(bookmark_total);
        self
    }
}

#[derive(Debug, Clone)]
pub struct BookmarkTagOptions {
    pub restrict: Restrict,
    pub is_novel: bool,
}

impl Default for BookmarkTagOptions {
    fn default() -> Self {
        Self {
            restrict: Restrict::Public,
            is_novel: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BookmarkAddOptions {
    pub tags: Vec<String>,
    pub restrict: Restrict,
    pub is_novel: bool,
}

impl Default for BookmarkAddOptions {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            restrict: Restrict::Public,
            is_novel: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommentAddOptions {
    pub comment: String,
    pub stamp_id: Option<u64>,
    pub parent_comment_id: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
enum Endpoint {
    AppApi,
    Sketch,
}

impl Endpoint {
    fn logical_host(self) -> &'static str {
        match self {
            Self::AppApi => APP_API_HOST,
            Self::Sketch => SKETCH_HOST,
        }
    }

    fn connect_target(self, api: &PixivApi) -> String {
        match self {
            Self::AppApi => api.config.target_ip.clone(),
            Self::Sketch => SKETCH_IP.to_owned(),
        }
    }
}

#[derive(Clone)]
enum RequestBody {
    None,
    Form(Params),
}

struct ApiRequest {
    client: Client,
    method: Method,
    url: Url,
    query: Params,
    body: RequestBody,
    host_header: Option<String>,
}

struct RawResponse {
    status: StatusCode,
    body: String,
}

fn push_optional(params: &mut Params, key: &str, value: Option<String>) {
    if let Some(value) = value {
        params.push((key.to_owned(), value));
    }
}

fn restrict_param(restrict: Option<Restrict>) -> String {
    restrict
        .map(|restrict| restrict.to_string())
        .unwrap_or_else(|| "all".to_owned())
}

fn search_word(word: &str, bookmark_total: Option<u64>) -> String {
    match bookmark_total {
        Some(total) => format!("{word} {total}users入り"),
        None => word.to_owned(),
    }
}

fn endpoint_url(logical_host: &str, path_or_url: &str) -> Result<Url, PixivError> {
    let mut url = Url::parse(&format!("https://{logical_host}/"))?;

    if path_or_url.starts_with("http://") || path_or_url.starts_with("https://") {
        let parsed = Url::parse(path_or_url)?;
        url.set_path(parsed.path());
        url.set_query(parsed.query());
        return Ok(url);
    }

    Ok(url.join(path_or_url.trim_start_matches('/'))?)
}

fn resolve_addr(target: &str) -> Option<SocketAddr> {
    target.parse::<SocketAddr>().ok().or_else(|| {
        target
            .parse::<IpAddr>()
            .ok()
            .map(|ip| SocketAddr::new(ip, 443))
    })
}

fn is_retryable_error_message(message: &str) -> bool {
    message.contains("Connection closed before full header was received")
        || message.contains("Connection terminated during handshake")
        || message.contains("timed out")
}

fn is_oauth_error(body: &str) -> bool {
    serde_json::from_str::<ErrorMessage>(body)
        .ok()
        .and_then(|message| message.error.message)
        .map(|message| message.contains("OAuth"))
        .unwrap_or(false)
}

fn http_status_error(status: StatusCode, body: String) -> PixivError {
    PixivError::http_status(status.as_u16(), body)
}

fn extract_json_object_after_key<'a>(src: &'a str, key: &str) -> Option<&'a str> {
    let key_pos = src.find(key)?;
    let mut i = key_pos + key.len();

    // skip whitespace
    while let Some(ch) = src[i..].chars().next() {
        if ch.is_whitespace() {
            i += ch.len_utf8();
        } else {
            break;
        }
    }

    if src[i..].chars().next()? != '{' {
        return None;
    }

    let start = i;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, ch) in src[start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let end = start + offset + ch.len_utf8();
                    return Some(&src[start..end]);
                }
            }
            _ => {}
        }
    }

    None
}
