use super::{parse::*, *};
use crate::{PixivError, PixivErrorKind};
use reqwest::{Client, Method, Url};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};
use tokio::{io::AsyncWriteExt, sync::Mutex};

/// One immutable session per client; create a new client when switching accounts.
#[derive(Clone)]
pub struct FanboxApi {
    client: Client,
    session: String,
    language: String,
    csrf: Arc<Mutex<Option<String>>>,
}

impl FanboxApi {
    pub fn new(
        session: String,
        proxy: Option<String>,
        language: String,
        accept_invalid_certs: bool,
    ) -> Result<Self, PixivError> {
        let session = normalize_session(&session)?;
        let mut builder = Client::builder()
            .timeout(Duration::from_secs(45))
            .connect_timeout(Duration::from_secs(15))
            .danger_accept_invalid_certs(accept_invalid_certs)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Mozilla/5.0");
        if let Some(proxy) = proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy)?)
        }
        Ok(Self {
            client: builder.build()?,
            session,
            language,
            csrf: Arc::new(Mutex::new(None)),
        })
    }
    pub async fn validate_session(&self) -> Result<(), PixivError> {
        self.get_creators(FanboxCreatorList::Following)
            .await
            .map(|_| ())
    }
    pub async fn get_posts(&self, feed: FanboxFeed) -> Result<FanboxPostPage, PixivError> {
        if let FanboxFeed::Creator { creator_id } = &feed {
            let pages = self.get_creator_post_pages(creator_id.clone()).await?;
            return match pages.first() {
                Some(url) => self.get_next_posts(url.clone()).await,
                None => Ok(FanboxPostPage::default()),
            };
        }
        let (endpoint, query) = match feed {
            FanboxFeed::Home => ("post.listHome", vec![("limit", "20".into())]),
            FanboxFeed::Supporting => ("post.listSupporting", vec![("limit", "20".into())]),
            FanboxFeed::Creator { creator_id } => (
                "post.listCreator",
                vec![
                    ("creatorId", creator_id),
                    ("limit", "20".into()),
                    ("withPinned", "true".into()),
                ],
            ),
            FanboxFeed::Tag {
                tag,
                creator_id,
                page,
            } => {
                let mut query = vec![("tag", tag), ("page", page.max(1).to_string())];
                if let Some(id) = creator_id {
                    query.push(("creatorId", id))
                };
                ("post.listTagged", query)
            }
        };
        post_page(&self.get(endpoint, query).await?)
    }
    pub async fn get_next_posts(&self, url: String) -> Result<FanboxPostPage, PixivError> {
        let url = api_url(
            &url,
            &[
                "post.listHome",
                "post.listSupporting",
                "post.listCreator",
                "post.listTagged",
            ],
        )?;
        let mut page = post_page(&self.request(Method::GET, url.clone(), None).await?)?;
        if page.next_url.is_none()
            && url.path() == "/post.listCreator"
            && let Some((_, id)) = url.query_pairs().find(|(k, _)| k == "creatorId")
        {
            let pages = self.get_creator_post_pages(id.into_owned()).await?;
            if let Some(index) = pages.iter().position(|p| p == url.as_str()) {
                page.next_url = pages.get(index + 1).cloned();
            }
        }
        Ok(page)
    }
    pub async fn get_creator_post_pages(
        &self,
        creator_id: String,
    ) -> Result<Vec<String>, PixivError> {
        let body = self
            .get("post.paginateCreator", vec![("creatorId", creator_id)])
            .await?;
        items(&body, &["pageUrls"])?
            .iter()
            .map(|v| {
                v.as_str()
                    .ok_or_else(malformed)
                    .and_then(|s| api_url(s, &["post.listCreator"]).map(|u| u.to_string()))
            })
            .collect()
    }
    pub async fn get_post(&self, post_id: String) -> Result<FanboxPost, PixivError> {
        post(&self.get("post.info", vec![("postId", post_id)]).await?)
    }
    pub async fn get_creator(&self, creator_id: String) -> Result<FanboxCreator, PixivError> {
        creator(
            &self
                .get("creator.get", vec![("creatorId", creator_id)])
                .await?,
        )
    }
    pub async fn get_creators(
        &self,
        list: FanboxCreatorList,
    ) -> Result<Vec<FanboxCreator>, PixivError> {
        let endpoint = match list {
            FanboxCreatorList::Following => "creator.listFollowing",
            FanboxCreatorList::Recommended => "creator.listRecommended",
            FanboxCreatorList::Pixiv => "creator.listPixiv",
        };
        let body = self.get(endpoint, vec![]).await?;
        items(&body, &["creators"])?.iter().map(creator).collect()
    }
    pub async fn search_creators(
        &self,
        keyword: String,
        page: u32,
    ) -> Result<FanboxCreatorPage, PixivError> {
        let body = self
            .get(
                "creator.search",
                vec![("q", keyword), ("page", page.max(1).to_string())],
            )
            .await?;
        Ok(FanboxCreatorPage {
            creators: items(&body, &["creators", "items"])?
                .iter()
                .map(creator)
                .collect::<Result<_, _>>()?,
            next_page: body["nextPage"].as_u64().map(|p| p as u32),
        })
    }
    pub async fn get_plans(
        &self,
        creator_id: Option<String>,
    ) -> Result<Vec<FanboxPlan>, PixivError> {
        let body = if let Some(id) = creator_id {
            self.get("plan.listCreator", vec![("creatorId", id)])
                .await?
        } else {
            self.get("plan.listSupporting", vec![]).await?
        };
        Ok(items(&body, &["plans"])?.iter().map(plan).collect())
    }
    pub async fn get_creator_support(
        &self,
        creator_id: String,
    ) -> Result<FanboxSupport, PixivError> {
        let body = self
            .get(
                "legacy/support/creator",
                vec![("creatorId", creator_id.clone())],
            )
            .await?;
        Ok(FanboxSupport {
            creator_id,
            fan_card_url: optional(&body, "supporterCardImageUrl"),
            started_datetime: optional(&body, "supportStartDatetime"),
        })
    }
    pub async fn get_creator_tags(&self, creator_id: String) -> Result<Vec<FanboxTag>, PixivError> {
        let body = self
            .get("tag.getFeatured", vec![("creatorId", creator_id)])
            .await?;
        Ok(items(&body, &["featuredTags", "tags"])?
            .iter()
            .map(tag)
            .collect())
    }
    pub async fn search_tags(&self, keyword: String) -> Result<Vec<FanboxTag>, PixivError> {
        let body = self.get("tag.search", vec![("q", keyword)]).await?;
        Ok(items(&body, &["tags"])?.iter().map(tag).collect())
    }
    pub async fn get_comments(&self, post_id: String) -> Result<FanboxCommentPage, PixivError> {
        comments(
            &self
                .get(
                    "post.getComments",
                    vec![
                        ("postId", post_id),
                        ("offset", "0".into()),
                        ("limit", "20".into()),
                    ],
                )
                .await?,
        )
    }
    pub async fn get_next_comments(&self, url: String) -> Result<FanboxCommentPage, PixivError> {
        comments(
            &self
                .request(Method::GET, api_url(&url, &["post.getComments"])?, None)
                .await?,
        )
    }
    /// Listing deliberately keeps notifications unread.
    pub async fn get_notices(&self) -> Result<FanboxNoticePage, PixivError> {
        notices(
            &self
                .get(
                    "bell.list",
                    vec![
                        ("page", "1".into()),
                        ("skipConvertUnreadNotification", "1".into()),
                        ("commentOnly", "0".into()),
                    ],
                )
                .await?,
        )
    }
    pub async fn get_next_notices(&self, url: String) -> Result<FanboxNoticePage, PixivError> {
        let mut url = api_url(&url, &["bell.list"])?;
        let query: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(k, _)| k != "skipConvertUnreadNotification")
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        url.set_query(None);
        url.query_pairs_mut()
            .extend_pairs(query)
            .append_pair("skipConvertUnreadNotification", "1");
        notices(&self.request(Method::GET, url, None).await?)
    }
    pub async fn get_messages(&self) -> Result<Vec<FanboxNotice>, PixivError> {
        let body = self.get("newsletter.list", vec![]).await?;
        Ok(items(&body, &["items"])?.iter().map(notice).collect())
    }
    pub async fn follow_creator(&self, user_id: String) -> Result<(), PixivError> {
        self.mutate("follow.create", json!({"creatorUserId":user_id}))
            .await
    }
    pub async fn unfollow_creator(&self, user_id: String) -> Result<(), PixivError> {
        self.mutate("follow.delete", json!({"creatorUserId":user_id}))
            .await
    }
    pub async fn like_post(&self, post_id: String) -> Result<(), PixivError> {
        self.mutate("post.likePost", json!({"postId":post_id}))
            .await
    }
    pub async fn like_comment(&self, comment_id: String) -> Result<(), PixivError> {
        self.mutate("post.likeComment", json!({"commentId":comment_id}))
            .await
    }
    pub async fn add_comment(
        &self,
        post_id: String,
        body: String,
        root_comment_id: Option<String>,
        parent_comment_id: Option<String>,
    ) -> Result<(), PixivError> {
        if body.trim().is_empty() {
            return Err(invalid("Comment is empty"));
        }
        let mut json = json!({"postId":post_id,"body":body});
        if let Some(id) = root_comment_id {
            json["rootCommentId"] = id.into()
        };
        if let Some(id) = parent_comment_id {
            json["parentCommentId"] = id.into()
        };
        self.mutate("post.addComment", json).await
    }
    pub async fn delete_comment(&self, comment_id: String) -> Result<(), PixivError> {
        self.mutate("post.deleteComment", json!({"commentId":comment_id}))
            .await
    }
    pub async fn refresh_csrf_token(&self) -> Result<(), PixivError> {
        let mut guard = self.csrf.lock().await;
        let response = self
            .auth(self.client.get("https://www.fanbox.cc/"))
            .send()
            .await
            .map_err(network)?;
        let html = read_response(response).await?;
        let document = scraper::Html::parse_document(&html);
        let selector = scraper::Selector::parse("meta[name='metadata']").expect("static selector");
        let content = document
            .select(&selector)
            .next()
            .and_then(|e| e.attr("content"))
            .ok_or_else(malformed)?;
        let metadata: Value = serde_json::from_str(content).map_err(|_| malformed())?;
        *guard = Some(optional(&metadata, "csrfToken").ok_or_else(malformed)?);
        Ok(())
    }
    /// Download only FANBOX/Pixiv media; credentials never follow redirects to another host.
    pub async fn get_media_bytes(&self, url: String) -> Result<Vec<u8>, PixivError> {
        let url = media_url(&url)?;
        let request = self
            .client
            .get(url.clone())
            .header("Referer", "https://www.fanbox.cc/");
        let request = if url
            .host_str()
            .is_some_and(|h| h == "fanbox.cc" || h.ends_with(".fanbox.cc"))
        {
            self.auth(request)
        } else {
            request
        };
        let mut response = request.send().await.map_err(network)?;
        check_status(response.status().as_u16())?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(network)? {
            if bytes.len() + chunk.len() > 32 * 1024 * 1024 {
                return Err(invalid("FANBOX image exceeds 32 MiB"));
            }
            bytes.extend_from_slice(&chunk)
        }
        Ok(bytes)
    }
    pub async fn download_media(&self, url: String, path: String) -> Result<(), PixivError> {
        let url = media_url(&url)?;
        let request = self
            .client
            .get(url.clone())
            .header("Referer", "https://www.fanbox.cc/");
        let request = if url
            .host_str()
            .is_some_and(|h| h == "fanbox.cc" || h.ends_with(".fanbox.cc"))
        {
            self.auth(request)
        } else {
            request
        };
        let mut response = request.send().await.map_err(network)?;
        check_status(response.status().as_u16())?;
        let temporary = format!("{path}.part");
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .await
            .map_err(|_| invalid("Cannot create download file"))?;
        let result = async {
            while let Some(chunk) = response.chunk().await.map_err(network)? {
                file.write_all(&chunk)
                    .await
                    .map_err(|_| invalid("Cannot write download file"))?;
            }
            file.flush()
                .await
                .map_err(|_| invalid("Cannot flush download file"))?;
            drop(file);
            if tokio::fs::try_exists(&path).await.unwrap_or(true) {
                return Err(invalid("Download destination exists"));
            }
            tokio::fs::rename(&temporary, &path)
                .await
                .map_err(|_| invalid("Cannot finish download"))
        }
        .await;
        if result.is_err() {
            let _ = tokio::fs::remove_file(&temporary).await;
        }
        result
    }
    fn auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        request
            .header("Cookie", format!("FANBOXSESSID={}", self.session))
            .header("Origin", "https://www.fanbox.cc")
            .header("Referer", "https://www.fanbox.cc/")
            .header("Accept-Language", &self.language)
    }
    async fn get(&self, endpoint: &str, query: Vec<(&str, String)>) -> Result<Value, PixivError> {
        let mut url = Url::parse(&format!("https://api.fanbox.cc/{endpoint}"))?;
        url.query_pairs_mut().extend_pairs(query);
        self.request(Method::GET, url, None).await
    }
    async fn mutate(&self, endpoint: &str, body: Value) -> Result<(), PixivError> {
        if self.csrf.lock().await.is_none() {
            self.refresh_csrf_token().await?;
        }
        self.request(
            Method::POST,
            Url::parse(&format!("https://api.fanbox.cc/{endpoint}"))?,
            Some(body),
        )
        .await
        .map(|_| ())
    }
    async fn request(
        &self,
        method: Method,
        url: Url,
        body: Option<Value>,
    ) -> Result<Value, PixivError> {
        let mut request = self.auth(self.client.request(method, url));
        if let Some(body) = body {
            let token = self
                .csrf
                .lock()
                .await
                .clone()
                .ok_or_else(|| invalid("Missing CSRF token"))?;
            request = request.header("x-csrf-token", token).json(&body);
        }
        let response = request.send().await.map_err(network)?;
        let text = read_response(response).await?;
        let value: Value = serde_json::from_str(&text).map_err(|_| malformed())?;
        if value
            .get("error")
            .is_some_and(|e| !e.is_null() && e != &Value::Bool(false))
        {
            return Err(invalid("FANBOX rejected the request"));
        }
        value.get("body").cloned().ok_or_else(malformed)
    }
}
fn comments(body: &Value) -> Result<FanboxCommentPage, PixivError> {
    let list = &body["commentList"];
    if list.is_null() && body["viewMode"].is_string() {
        return Ok(FanboxCommentPage::default());
    }
    Ok(FanboxCommentPage {
        comments: items(list, &["items"])?.iter().map(comment).collect(),
        next_url: optional(list, "nextUrl"),
        can_comment: !matches!(
            body["viewMode"].as_str(),
            Some("COMMENTING_RESTRICTED" | "hidden" | "restricted" | "read_only" | "disabled")
        ),
    })
}
fn notices(body: &Value) -> Result<FanboxNoticePage, PixivError> {
    Ok(FanboxNoticePage {
        notices: items(body, &["items"])?.iter().map(notice).collect(),
        next_url: optional(body, "nextUrl"),
    })
}
fn invalid(message: &str) -> PixivError {
    PixivError::new(PixivErrorKind::InvalidEndpoint, message.into())
}
fn network(_: reqwest::Error) -> PixivError {
    PixivError::new(
        PixivErrorKind::HttpClient,
        "FANBOX network request failed".into(),
    )
}
fn check_status(status: u16) -> Result<(), PixivError> {
    if (200..300).contains(&status) {
        Ok(())
    } else {
        Err(PixivError::http_status(status, String::new()))
    }
}
async fn read_response(mut response: reqwest::Response) -> Result<String, PixivError> {
    check_status(response.status().as_u16())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network)? {
        if bytes.len() + chunk.len() > 16 * 1024 * 1024 {
            return Err(invalid("FANBOX response exceeds size limit"));
        }
        bytes.extend_from_slice(&chunk)
    }
    String::from_utf8(bytes).map_err(|_| malformed())
}
pub(super) fn normalize_session(input: &str) -> Result<String, PixivError> {
    let input = input.trim();
    let value = input.strip_prefix("FANBOXSESSID=").unwrap_or(input);
    if value.is_empty() || value.bytes().any(|b| b <= 32 || b >= 127 || b == b';') {
        return Err(invalid("Expected FANBOXSESSID value"));
    }
    Ok(value.to_owned())
}
pub(super) fn api_url(input: &str, paths: &[&str]) -> Result<Url, PixivError> {
    let url = Url::parse(input)?;
    if url.scheme() != "https"
        || url.host_str() != Some("api.fanbox.cc")
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || !paths.contains(&url.path().trim_start_matches('/'))
    {
        return Err(invalid("Unexpected FANBOX pagination URL"));
    }
    Ok(url)
}
fn media_url(input: &str) -> Result<Url, PixivError> {
    let url = Url::parse(input)?;
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || !(host == "fanbox.cc"
            || host.ends_with(".fanbox.cc")
            || host == "pximg.net"
            || host.ends_with(".pximg.net"))
    {
        return Err(invalid("Unexpected FANBOX media URL"));
    }
    Ok(url)
}
