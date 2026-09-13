use super::parse_shared::*;
use super::*;
use crate::PixivError;
use scraper::{ElementRef, Html};

pub fn parse_article_page(html: &str, url: &str) -> Result<ArticlePage, PixivError> {
    let base = site_url(url)?;
    let doc = Html::parse_document(html);
    let root = doc.root_element();
    if root
        .select(&select(".main-column-container"))
        .next()
        .is_none()
    {
        return Err(malformed("Pixivision list container not found"));
    }
    let (monthly_ranking, recommended) = sidebar(root, &base);
    let articles = summaries(
        root,
        "._article-eyecatch-card, .main-column-container article._article-card",
        &base,
    );
    let mut seen_categories = std::collections::HashSet::new();
    let categories = root
        .select(&select("a.gnv__category-link"))
        .filter_map(|e| link(e, &base))
        .filter(|c| seen_categories.insert(c.url.clone()))
        .collect();
    if articles.is_empty()
        && root
            .select(&select(
                ".main-column-container article._article-card, ._article-eyecatch-card",
            ))
            .next()
            .is_some()
    {
        return Err(malformed("Pixivision article cards could not be parsed"));
    }
    Ok(ArticlePage {
        url: base.to_string(),
        title: first_text(
            root,
            ".ssc__header, .tdc__tag-name, .tlp__heading, .search-result-header, h1",
        )
        .or_else(|| first_text(root, "title"))
        .unwrap_or_default(),
        description: first_text(root, ".ssc__descriotion, .tdc__description"),
        articles,
        next_url: navigation(root, "._pager a.next, a[rel='next']", &base),
        previous_url: navigation(root, "._pager a.back, a[rel='prev']", &base),
        monthly_ranking,
        recommended,
        categories,
    })
}

pub fn parse_tag_directory(html: &str, url: &str) -> Result<TagDirectory, PixivError> {
    let base = site_url(url)?;
    let doc = Html::parse_document(html);
    let root = doc
        .select(&select("main._tag-list-page"))
        .next()
        .ok_or_else(|| malformed("Pixivision tag directory not found"))?;
    let mut groups = Vec::new();
    for section in root.select(&select(".tlp__section")) {
        let name = first_text(section, ".tlp__section-heading").unwrap_or_default();
        let nodes = section
            .select(&select(".tlc__list"))
            .next()
            .map(|list| child_nodes(list, &base))
            .unwrap_or_default();
        groups.push(TagGroup { name, nodes });
    }
    Ok(TagDirectory {
        url: base.to_string(),
        groups,
    })
}

fn child_nodes(list: ElementRef<'_>, base: &reqwest::Url) -> Vec<TagNode> {
    list.child_elements()
        .filter(|e| {
            e.value()
                .has_class("tlc__node", scraper::CaseSensitivity::CaseSensitive)
        })
        .filter_map(|node| {
            let label = node.child_elements().find(|e| {
                e.value()
                    .has_class("tlc__name", scraper::CaseSensitivity::CaseSensitive)
            })?;
            // Counts are nested in the label and must not become part of its name.
            let name = label
                .children()
                .filter_map(|n| n.value().as_text())
                .map(|t| t.to_string())
                .collect::<String>()
                .trim()
                .to_owned();
            let parsed_tag = tag(label, base).map(|mut tag| {
                tag.name = name.clone();
                tag
            });
            let children = node
                .child_elements()
                .find(|e| e.value().name() == "ul")
                .map(|list| child_nodes(list, base))
                .unwrap_or_default();
            let article_count = node
                .attr("data-count")
                .and_then(|s| s.parse().ok())
                .or_else(|| {
                    first_text(label, ".tlc__count").and_then(|s| {
                        s.chars()
                            .filter(char::is_ascii_digit)
                            .collect::<String>()
                            .parse()
                            .ok()
                    })
                });
            Some(TagNode {
                name,
                tag: parsed_tag,
                article_count,
                children,
            })
        })
        .collect()
}
