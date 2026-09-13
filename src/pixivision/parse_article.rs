use super::parse_shared::*;
use super::*;
use crate::PixivError;
use ammonia::{Builder, UrlRelative};
use reqwest::Url;
use scraper::{ElementRef, Html};
use std::collections::HashSet;

pub fn parse_article(html: &str, url: &str) -> Result<Article, PixivError> {
    let base = site_url(url)?;
    let id = route_id(url, "a").ok_or_else(|| invalid("Expected a pixivision article URL"))?;
    let language = base
        .path_segments()
        .and_then(|mut p| p.next())
        .and_then(Language::from_path)
        .ok_or_else(|| invalid("Unsupported language"))?;
    let doc = Html::parse_document(html);
    let root = doc.root_element();
    let article = root
        .select(&select(".am__article-body-container"))
        .next()
        .ok_or_else(|| malformed("Pixivision article container not found"))?;
    let title = first_text(article, ".am__title")
        .ok_or_else(|| malformed("Pixivision article title not found"))?;
    let body = article
        .select(&select("._feature-article-body"))
        .next()
        .or_else(|| article.select(&select(".am__body")).next())
        .ok_or_else(|| malformed("Pixivision article body not found"))?;
    let mut blocks = Vec::new();
    collect_blocks(body, &base, &mut blocks);
    if blocks.is_empty() {
        return Err(malformed("Pixivision article has no readable content"));
    }
    let publish_date = article
        .select(&select("time[datetime]"))
        .next()
        .and_then(|e| e.attr("datetime"))
        .ok_or_else(|| malformed("Pixivision article date not found"))?
        .to_owned();
    let category = article
        .select(&select(".am__header a[href]"))
        .filter_map(|e| link(e, &base))
        .find(|l| {
            Url::parse(&l.url)
                .is_ok_and(|u| u.host_str() == base.host_str() && u.path().contains("/c/"))
        });
    let article_tags = article
        .select(&select(".am__header-tags"))
        .next()
        .map(|e| tags(e, &base))
        .unwrap_or_default();
    let translations = root
        .select(&select("link[rel='alternate'][hreflang][href]"))
        .filter_map(|e| {
            let url = absolute(&base, e.attr("href")?)?;
            route_id(&url, "a")?;
            Some(ArticleLink {
                title: e.attr("hreflang")?.to_owned(),
                url,
            })
        })
        .collect();
    let related = root
        .select(&select("._related-articles"))
        .map(|section| ArticleSection {
            title: first_text(section, ".rla__heading, h2, h3").unwrap_or_default(),
            url: navigation(section, ".rla__heading a", &base),
            articles: summaries(section, "article", &base),
        })
        .collect();
    let (monthly_ranking, recommended) = sidebar(root, &base);
    Ok(Article {
        id,
        url: base.to_string(),
        language,
        title,
        description: metadata(&doc, "meta[name='description']"),
        thumbnail: metadata(&doc, "meta[property='og:image']").and_then(|s| absolute(&base, &s)),
        publish_date,
        category,
        tags: article_tags,
        translations,
        blocks,
        related,
        monthly_ranking,
        recommended,
        next_url: navigation(article, "._pager a.next, a[rel='next']", &base),
        previous_url: navigation(article, "._pager a.back, a[rel='prev']", &base),
    })
}

fn collect_blocks(root: ElementRef<'_>, base: &Url, output: &mut Vec<ArticleBlock>) {
    // Modern pages have article-item blocks. Older pages use direct body children.
    // Descend through layout wrappers only when they contain actual article items.
    for node in root.children() {
        let Some(element) = ElementRef::wrap(node) else {
            if let Some(value) = node.value().as_text().filter(|t| !t.trim().is_empty()) {
                let fragment = Html::parse_fragment(&format!("<p>{}</p>", escape(value)));
                if let Some(paragraph) = fragment.root_element().child_elements().next() {
                    output.push(block(paragraph, base));
                }
            }
            continue;
        };
        if matches!(element.value().name(), "script" | "style" | "noscript") {
            continue;
        }
        if !element
            .value()
            .has_class("article-item", scraper::CaseSensitivity::CaseSensitive)
            && element.select(&select(".article-item")).next().is_some()
        {
            collect_blocks(element, base, output);
            continue;
        }
        if element
            .value()
            .classes()
            .any(|c| c.starts_with("ads-") || c == "_ad" || c == "ad-container")
        {
            continue;
        }
        let parsed = block(element, base);
        if !parsed.text.is_empty()
            || !parsed.images.is_empty()
            || !parsed.embeds.is_empty()
            || parsed.kind == BlockKind::Divider
        {
            output.push(parsed);
        }
    }
}

fn block(root: ElementRef<'_>, base: &Url) -> ArticleBlock {
    let mut source = root.html();
    let mut images = Vec::new();
    let mut seen_images = HashSet::new();
    let image_elements: Vec<_> = std::iter::once(root)
        .chain(root.select(&select("img")))
        .filter(|e| e.value().name() == "img")
        .collect();
    for image in image_elements {
        if let Some(url) = image_url(image, base) {
            let alt = image.attr("alt").unwrap_or_default().to_owned();
            // Normalize lazy images before sanitization; the source may be a data: placeholder.
            let replacement = format!("<img src=\"{}\" alt=\"{}\">", escape(&url), escape(&alt));
            source = source.replace(&image.html(), &replacement);
            if seen_images.insert(url.clone()) {
                images.push(ArticleImage {
                    url,
                    alt,
                    width: image.attr("width").and_then(|s| s.parse().ok()),
                    height: image.attr("height").and_then(|s| s.parse().ok()),
                });
            }
        }
    }
    let works = featured_works(root, base);
    let embeds = embeds(root, base);
    let kind = block_kind(root, &works, &embeds);
    let heading = root
        .select(&select("h1, h2, h3, h4, h5, h6"))
        .next()
        .or_else(|| root.value().name().starts_with('h').then_some(root));
    let heading_level = if kind == BlockKind::Heading {
        heading
            .and_then(|e| e.value().name().strip_prefix('h'))
            .and_then(|s| s.parse().ok())
    } else {
        None
    };
    let anchor = root
        .attr("id")
        .or_else(|| heading.and_then(|h| h.attr("id")))
        .map(str::to_owned);
    let html = Builder::default()
        .url_relative(UrlRelative::RewriteWithBase(base.clone()))
        .add_generic_attributes(&["class", "id"])
        .add_tags(&[
            "figure",
            "figcaption",
            "picture",
            "ruby",
            "rt",
            "rp",
            "video",
            "audio",
            "source",
        ])
        .add_tag_attributes("video", &["src", "poster", "controls", "width", "height"])
        .add_tag_attributes("audio", &["src", "controls"])
        .add_tag_attributes("source", &["src", "type"])
        .clean(&source)
        .to_string();
    let clean = Html::parse_fragment(&html);
    let text = text(clean.root_element());
    let mut seen_links = HashSet::new();
    let links = std::iter::once(root)
        .chain(root.select(&select("a[href]")))
        .filter(|e| e.value().name() == "a")
        .filter_map(|a| link(a, base))
        .filter(|l| seen_links.insert(l.url.clone()))
        .collect();
    ArticleBlock {
        kind,
        anchor,
        heading_level,
        html,
        text,
        images,
        links,
        works,
        embeds,
    }
}

fn block_kind(root: ElementRef<'_>, works: &[FeaturedWork], embeds: &[ArticleEmbed]) -> BlockKind {
    let classes: Vec<_> = root.value().classes().collect();
    for (suffix, kind) in [
        ("table_of_contents", BlockKind::TableOfContents),
        ("heading", BlockKind::Heading),
        ("profile", BlockKind::Profile),
        ("question", BlockKind::Question),
        ("answer", BlockKind::Answer),
        ("article_card", BlockKind::ArticleCard),
        ("paragraph", BlockKind::Paragraph),
        ("image", BlockKind::Image),
        ("article_thumbnail", BlockKind::Image),
        ("movie", BlockKind::Video),
    ] {
        if classes.iter().any(|c| c.ends_with(&format!("__{suffix}"))) {
            return kind;
        }
    }
    if !works.is_empty() {
        return BlockKind::PixivWork;
    }
    if !embeds.is_empty() {
        return BlockKind::Video;
    }
    match root.value().name() {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => BlockKind::Heading,
        "p" => BlockKind::Paragraph,
        "img" | "figure" | "picture" => BlockKind::Image,
        "blockquote" => BlockKind::Quote,
        "ul" | "ol" => BlockKind::List,
        "table" => BlockKind::Table,
        "pre" | "code" => BlockKind::Code,
        "hr" => BlockKind::Divider,
        _ => BlockKind::Unknown,
    }
}

fn featured_works(root: ElementRef<'_>, base: &Url) -> Vec<FeaturedWork> {
    let scopes: Vec<_> = root.select(&select(".am__work, .am__novel")).collect();
    let scopes = if scopes.is_empty() {
        vec![root]
    } else {
        scopes
    };
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for scope in scopes {
        for anchor in scope.select(&select("a[href]")) {
            let Some(url) = anchor
                .attr("href")
                .and_then(|h| absolute(base, h))
                .and_then(|s| Url::parse(&s).ok())
            else {
                continue;
            };
            let Some((kind, id)) = work_id(&url) else {
                continue;
            };
            if !seen.insert((matches!(kind, FeaturedWorkKind::Novel), id)) {
                continue;
            }
            let user_link = scope
                .select(&select(
                    ".am__work__user-name a, a[href*='/users/'], a[href*='member.php']",
                ))
                .find_map(|a| {
                    let url = Url::parse(&absolute(base, a.attr("href")?)?).ok()?;
                    let id = user_id(&url)?;
                    Some((id, text(a)))
                });
            let preview = scope
                .select(&select(
                    ".am__work__main img, .am__work__illust, .am__novel img",
                ))
                .find_map(|i| image_url(i, base));
            let count = first_text(scope, ".mic__label").and_then(|s| {
                s.chars()
                    .filter(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .ok()
            });
            result.push(FeaturedWork {
                id,
                kind,
                title: first_text(scope, ".am__work__title, .am__novel__title")
                    .unwrap_or_else(|| text(anchor)),
                url: url.to_string(),
                user_id: user_link.as_ref().map(|u| u.0),
                user_name: user_link.map(|u| u.1).filter(|s| !s.is_empty()),
                preview,
                page_count: count,
            });
        }
    }
    result
}

fn work_id(url: &Url) -> Option<(FeaturedWorkKind, u64)> {
    if !matches!(url.host_str(), Some("www.pixiv.net" | "pixiv.net")) {
        return None;
    }
    let parts: Vec<_> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
    if let Some(index) = parts.iter().position(|s| *s == "artworks") {
        return Some((
            FeaturedWorkKind::Illustration,
            parts.get(index + 1)?.parse().ok()?,
        ));
    }
    if url.path() == "/member_illust.php" {
        return Some((
            FeaturedWorkKind::Illustration,
            url.query_pairs()
                .find(|(k, _)| k == "illust_id")?
                .1
                .parse()
                .ok()?,
        ));
    }
    if url.path().ends_with("/novel/show.php") {
        return Some((
            FeaturedWorkKind::Novel,
            url.query_pairs().find(|(k, _)| k == "id")?.1.parse().ok()?,
        ));
    }
    None
}

fn user_id(url: &Url) -> Option<u64> {
    if !matches!(url.host_str(), Some("www.pixiv.net" | "pixiv.net")) {
        return None;
    }
    let parts: Vec<_> = url.path_segments()?.collect();
    if let Some(index) = parts.iter().position(|s| *s == "users") {
        return parts.get(index + 1)?.parse().ok();
    }
    if url.path() == "/member.php" {
        return url.query_pairs().find(|(k, _)| k == "id")?.1.parse().ok();
    }
    None
}

fn embeds(root: ElementRef<'_>, base: &Url) -> Vec<ArticleEmbed> {
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for element in std::iter::once(root).chain(root.select(&select(
        "iframe, video, audio, source, blockquote.twitter-tweet a[href]",
    ))) {
        let name = element.value().name();
        let kind = match name {
            "iframe" => EmbedKind::Frame,
            "video" => EmbedKind::Video,
            "audio" => EmbedKind::Audio,
            "source" => {
                if element
                    .ancestors()
                    .filter_map(ElementRef::wrap)
                    .any(|e| e.value().name() == "audio")
                {
                    EmbedKind::Audio
                } else if element
                    .ancestors()
                    .filter_map(ElementRef::wrap)
                    .any(|e| e.value().name() == "video")
                {
                    EmbedKind::Video
                } else {
                    continue;
                }
            }
            "a" => EmbedKind::SocialPost,
            _ => continue,
        };
        let attr = if name == "a" { "href" } else { "src" };
        let Some(url) = element.attr(attr).and_then(|h| absolute(base, h)) else {
            continue;
        };
        if name == "a" && !url.contains("/status/") {
            continue;
        }
        if !seen.insert(url.clone()) {
            continue;
        }
        result.push(ArticleEmbed {
            kind,
            url,
            title: element.attr("title").map(str::to_owned),
            poster: element.attr("poster").and_then(|p| absolute(base, p)),
        });
    }
    result
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
