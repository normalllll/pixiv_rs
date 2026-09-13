use super::{ArticleLink, ArticleSummary, PixivisionTag};
use crate::{PixivError, PixivErrorKind};
use reqwest::Url;
use scraper::{ElementRef, Html, Selector};
use std::collections::HashSet;

pub(super) fn select(css: &str) -> Selector {
    Selector::parse(css).expect("static pixivision selector")
}
pub(super) fn invalid(message: &str) -> PixivError {
    PixivError::new(PixivErrorKind::InvalidEndpoint, message.into())
}
pub(super) fn malformed(message: &str) -> PixivError {
    PixivError::new(PixivErrorKind::Html, message.into())
}

pub(super) fn site_url(input: &str) -> Result<Url, PixivError> {
    let url = Url::parse(input)?;
    if url.scheme() != "https"
        || url.host_str() != Some("www.pixivision.net")
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(invalid("Expected an HTTPS www.pixivision.net URL"));
    }
    if url
        .path_segments()
        .and_then(|mut p| p.next())
        .and_then(super::Language::from_path)
        .is_none()
    {
        return Err(invalid("Unsupported pixivision language"));
    }
    Ok(url)
}

pub(super) fn absolute(base: &Url, input: &str) -> Option<String> {
    let url = base.join(input.trim()).ok()?;
    if !matches!(url.scheme(), "https" | "http")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return None;
    }
    Some(url.to_string())
}

pub(super) fn text(element: ElementRef<'_>) -> String {
    let mut result = String::new();
    for node in element.descendants() {
        if node
            .ancestors()
            .filter_map(ElementRef::wrap)
            .any(|e| matches!(e.value().name(), "script" | "style" | "noscript"))
        {
            continue;
        }
        if let Some(t) = node.value().as_text() {
            result.push_str(t);
        } else if let Some(e) = ElementRef::wrap(node)
            && matches!(
                e.value().name(),
                "br" | "p" | "div" | "li" | "h1" | "h2" | "h3" | "h4" | "tr"
            )
        {
            result.push('\n');
        }
    }
    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn first_text(root: ElementRef<'_>, css: &str) -> Option<String> {
    root.select(&select(css)).map(text).find(|s| !s.is_empty())
}

pub(super) fn link(element: ElementRef<'_>, base: &Url) -> Option<ArticleLink> {
    Some(ArticleLink {
        title: text(element),
        url: absolute(base, element.attr("href")?)?,
    })
}

pub(super) fn route_id(url: &str, segment: &str) -> Option<u64> {
    let url = Url::parse(url).ok()?;
    if url.host_str() != Some("www.pixivision.net") {
        return None;
    }
    let parts: Vec<_> = url.path_segments()?.collect();
    if parts.get(1) != Some(&segment) {
        return None;
    }
    parts.get(2)?.parse().ok()
}

pub(super) fn tag(element: ElementRef<'_>, base: &Url) -> Option<PixivisionTag> {
    let link = link(element, base)?;
    Some(PixivisionTag {
        id: route_id(&link.url, "t")?,
        name: link.title,
        url: link.url,
    })
}

pub(super) fn tags(root: ElementRef<'_>, base: &Url) -> Vec<PixivisionTag> {
    let mut seen = HashSet::new();
    root.select(&select("a[href]"))
        .filter_map(|a| tag(a, base))
        .filter(|t| seen.insert(t.id))
        .collect()
}

pub(super) fn image_url(element: ElementRef<'_>, base: &Url) -> Option<String> {
    for attr in ["data-src", "data-original", "src"] {
        if let Some(value) = element.attr(attr).and_then(|s| absolute(base, s)) {
            return Some(value);
        }
    }
    let srcset = element
        .attr("data-srcset")
        .or_else(|| element.attr("srcset"));
    if let Some(srcset) = srcset
        && let Some(value) = srcset
            .split(',')
            .next()
            .and_then(|s| s.split_whitespace().next())
            .and_then(|s| absolute(base, s))
    {
        return Some(value);
    }
    let style = element.attr("style")?;
    let start = style.find("url(")? + 4;
    let end = style[start..].find(')')? + start;
    absolute(base, style[start..end].trim().trim_matches(['\'', '"']))
}

pub(super) fn thumbnail(root: ElementRef<'_>, base: &Url) -> Option<String> {
    root.select(&select("._thumbnail, img"))
        .find_map(|e| image_url(e, base))
}

pub(super) fn summary(root: ElementRef<'_>, base: &Url) -> Option<ArticleSummary> {
    let title_link = root.select(&select("a[data-gtm-action='ClickTitle'], .arc__title a, .asc__title-link, .aec__title a, .arrct__title a")).next()
        .or_else(|| root.select(&select("a[href]")).find(|a| a.attr("href").and_then(|h| absolute(base,h)).and_then(|u| route_id(&u,"a")).is_some()))?;
    let article = link(title_link, base)?;
    if article.title.is_empty() {
        return None;
    }
    let category = root
        .select(&select("a[href]"))
        .filter_map(|e| link(e, base))
        .find(|l| {
            Url::parse(&l.url).is_ok_and(|u| {
                u.host_str() == Some("www.pixivision.net") && u.path().contains("/c/")
            })
        });
    Some(ArticleSummary {
        id: route_id(&article.url, "a")?,
        title: article.title,
        url: article.url,
        thumbnail: thumbnail(root, base),
        publish_date: root
            .select(&select("time[datetime]"))
            .next()
            .and_then(|t| t.attr("datetime"))
            .map(str::to_owned),
        category,
        tags: tags(root, base),
    })
}

pub(super) fn summaries(root: ElementRef<'_>, css: &str, base: &Url) -> Vec<ArticleSummary> {
    let mut seen = HashSet::new();
    root.select(&select(css))
        .filter_map(|e| summary(e, base))
        .filter(|a| seen.insert(a.id))
        .collect()
}

pub(super) fn navigation(root: ElementRef<'_>, css: &str, base: &Url) -> Option<String> {
    root.select(&select(css))
        .filter_map(|e| e.attr("href"))
        .filter_map(|h| absolute(base, h))
        .find(|u| u != base.as_str() && site_url(u).is_ok())
}

pub(super) fn metadata(document: &Html, css: &str) -> Option<String> {
    document
        .select(&select(css))
        .next()
        .and_then(|e| e.attr("content"))
        .map(str::to_owned)
}

pub(super) fn sidebar(
    root: ElementRef<'_>,
    base: &Url,
) -> (Vec<ArticleSummary>, Vec<ArticleSummary>) {
    let ranking = summaries(root, ".alc__articles-list-group--ranking article", base);
    let recommended = root
        .select(&select("._articles-list-card"))
        .filter(|section| {
            section
                .select(&select(".alc__articles-list-group--ranking"))
                .next()
                .is_none()
        })
        .flat_map(|section| summaries(section, "article", base))
        .collect();
    (ranking, recommended)
}
