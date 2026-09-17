use super::parse_shared::{invalid, site_url};
use super::{
    Article, ArticleFeed, ArticlePage, Language, TagDirectory, parse_article, parse_article_page,
    parse_tag_directory,
};
use crate::{PixivError, PixivErrorKind};
use reqwest::{Client, Proxy, StatusCode, Url};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct PixivisionConfig {
    pub language: Language,
    pub proxy: Option<String>,
    pub accept_invalid_certs: bool,
}

impl Default for PixivisionConfig {
    fn default() -> Self {
        Self {
            language: Language::Japanese,
            proxy: None,
            accept_invalid_certs: false,
        }
    }
}

/// Anonymous website client. Pixiv OAuth credentials are never sent to this host.
#[derive(Clone)]
pub struct PixivisionApi {
    client: Client,
    language: Language,
}

impl PixivisionApi {
    pub fn new(config: PixivisionConfig) -> Result<Self, PixivError> {
        let mut builder = crate::http::client_builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    return attempt.error("too many pixivision redirects");
                }
                if site_url(attempt.url().as_str()).is_err() {
                    return attempt.error("unexpected pixivision redirect");
                }
                attempt.follow()
            }))
            .user_agent("Mozilla/5.0 (compatible; pixiv_rs)");
        if let Some(proxy) = config.proxy {
            builder = builder.proxy(Proxy::all(proxy)?);
        }
        Ok(Self {
            client: builder.build()?,
            language: config.language,
        })
    }

    pub async fn get_article_page(
        &self,
        feed: ArticleFeed,
        page: u32,
    ) -> Result<ArticlePage, PixivError> {
        self.get_next_article_page(feed_url(self.language, feed, page)?.to_string())
            .await
    }

    /// Accepts site list links; keeps search/tag/category and pagination parameters.
    pub async fn get_next_article_page(&self, url: String) -> Result<ArticlePage, PixivError> {
        let requested = site_url(&url)?;
        let (url, html, status) = self.fetch(requested).await?;
        if status == StatusCode::NOT_FOUND && url.path().trim_end_matches('/').ends_with("/s") {
            // The site returns its normal 404 template for zero search matches.
            // Only accept that recognizable template, not arbitrary challenge HTML.
            let mut page = parse_article_page(&html, url.as_str())?;
            if page.articles.is_empty() {
                page.next_url = None;
                return Ok(page);
            }
        }
        check_status(status)?;
        parse_article_page(&html, url.as_str())
    }

    pub async fn get_article(&self, id: u64) -> Result<Article, PixivError> {
        self.get_article_by_url(format!(
            "https://www.pixivision.net/{}/a/{id}",
            self.language.path()
        ))
        .await
    }

    /// Also accepts translated article URLs and links to subsequent article pages.
    pub async fn get_article_by_url(&self, url: String) -> Result<Article, PixivError> {
        let requested = site_url(&url)?;
        let (url, html, status) = self.fetch(requested).await?;
        check_status(status)?;
        parse_article(&html, url.as_str())
    }

    pub async fn get_tag_directory(&self) -> Result<TagDirectory, PixivError> {
        let url = site_url(&format!(
            "https://www.pixivision.net/{}/t",
            self.language.path()
        ))?;
        let (url, html, status) = self.fetch(url).await?;
        check_status(status)?;
        parse_tag_directory(&html, url.as_str())
    }

    async fn fetch(&self, url: Url) -> Result<(Url, String, StatusCode), PixivError> {
        let language = url
            .path_segments()
            .and_then(|mut p| p.next())
            .and_then(Language::from_path)
            .ok_or_else(|| invalid("Unsupported pixivision language"))?;
        let mut response = self
            .client
            .get(url)
            .header("Accept-Language", language.path())
            .header(
                "Cookie",
                format!("user_lang={}", language.path().replace('-', "_")),
            )
            .send()
            .await?;
        let final_url = site_url(response.url().as_str())?;
        let status = response.status();
        if !status.is_success() && status != StatusCode::NOT_FOUND {
            check_status(status)?;
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 8 * 1024 * 1024 {
                return Err(invalid("Pixivision page exceeds 8 MiB"));
            }
            bytes.extend_from_slice(&chunk);
        }
        let html = String::from_utf8(bytes).map_err(|_| {
            PixivError::new(
                PixivErrorKind::Html,
                "Pixivision response is not UTF-8".into(),
            )
        })?;
        Ok((final_url, html, status))
    }
}

pub(super) fn feed_url(
    language: Language,
    feed: ArticleFeed,
    page: u32,
) -> Result<Url, PixivError> {
    if page == 0 {
        return Err(invalid("Page numbers start at 1"));
    }
    let mut url = site_url(&format!("https://www.pixivision.net/{}/", language.path()))?;
    match feed {
        ArticleFeed::Latest => {}
        ArticleFeed::Category { category } => {
            url.set_path(&format!("/{}/c/{}", language.path(), category.path()))
        }
        ArticleFeed::Tag { tag_id } => url.set_path(&format!("/{}/t/{tag_id}", language.path())),
        ArticleFeed::Search { keyword } => {
            if keyword.trim().is_empty() {
                return Err(invalid("Search keyword is empty"));
            }
            url.set_path(&format!("/{}/s/", language.path()));
            url.query_pairs_mut().append_pair("q", &keyword);
        }
    }
    if page > 1 {
        url.query_pairs_mut().append_pair("p", &page.to_string());
    }
    Ok(url)
}

fn check_status(status: StatusCode) -> Result<(), PixivError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(PixivError::http_status(status.as_u16(), String::new()))
    }
}
