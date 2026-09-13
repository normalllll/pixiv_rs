use super::*;

#[test]
fn rejects_unexpected_pages_and_origins() {
    assert!(
        parse_article(
            "<html>challenge</html>",
            "https://www.pixivision.net/en/a/1"
        )
        .is_err()
    );
    assert!(
        parse_article_page("<html>challenge</html>", "https://www.pixivision.net/en/").is_err()
    );
    assert!(parse_tag_directory("<html></html>", "https://www.pixivision.net/en/t").is_err());
    for url in [
        "http://www.pixivision.net/en/",
        "https://evil.test/en/",
        "https://www.pixivision.net.evil.test/en/",
        "https://user@www.pixivision.net/en/",
    ] {
        assert!(super::parse_shared::site_url(url).is_err());
    }
}

#[test]
fn query_encoding_preserves_keyword() {
    let keyword = "猫 & ?/#";
    let url = super::client::feed_url(
        Language::Japanese,
        ArticleFeed::Search {
            keyword: keyword.into(),
        },
        2,
    )
    .unwrap();
    assert_eq!(
        url.query_pairs().collect::<Vec<_>>(),
        vec![("q".into(), keyword.into()), ("p".into(), "2".into())]
    );
    assert!(super::client::feed_url(Language::Japanese, ArticleFeed::Latest, 0).is_err());
}

#[test]
fn parses_article_and_sanitizes_rich_content() {
    let html = r#"<article class="am__article-body-container"><header class="am__header"><h1 class="am__title">Example</h1><time datetime="2026-01-01"></time></header><div class="am__body"><div class="_feature-article-body"><div class="article-item _feature-article-body__heading" id="section"><h2>Heading</h2></div><div class="article-item _feature-article-body__paragraph"><p>Hello <strong>world</strong><script>alert(1)</script><a href="javascript:alert(1)">bad</a><img data-src="/image.jpg" onerror="alert(1)"></p></div><div class="article-item _feature-article-body__movie"><iframe src="https://www.youtube.com/embed/example"></iframe></div></div></div></article>"#;
    let article = parse_article(html, "https://www.pixivision.net/en/a/1").unwrap();
    assert_eq!(article.title, "Example");
    assert_eq!(article.blocks.len(), 3);
    assert_eq!(article.blocks[0].kind, BlockKind::Heading);
    assert_eq!(article.blocks[0].anchor.as_deref(), Some("section"));
    assert!(article.blocks[1].html.contains("<strong>world</strong>"));
    assert!(!article.blocks[1].html.contains("alert"));
    assert_eq!(
        article.blocks[1].images[0].url,
        "https://www.pixivision.net/image.jpg"
    );
    assert_eq!(article.blocks[2].embeds.len(), 1);
    assert!(!article.blocks[2].html.contains("iframe"));
}

#[test]
fn parses_list_and_pagination() {
    let html = r#"<main class="main-column-container"><article class="_article-card"><a data-gtm-action="ClickTitle" href="/en/a/12">Title</a><time datetime="2026-01-01"></time></article><div class="_pager"><a class="next" href="?p=2">Next</a></div></main>"#;
    let page = parse_article_page(html, "https://www.pixivision.net/en/").unwrap();
    assert_eq!(page.articles[0].id, 12);
    assert_eq!(
        page.next_url.as_deref(),
        Some("https://www.pixivision.net/en/?p=2")
    );
}

#[tokio::test]
#[ignore = "Requires network; optional .local/session.json proxy configuration"]
async fn live_website() {
    let settings: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(".local/session.json").unwrap()).unwrap();
    let api = PixivisionApi::new(PixivisionConfig {
        language: Language::SimplifiedChinese,
        proxy: settings["proxy"].as_str().map(str::to_owned),
        accept_invalid_certs: settings["accept_invalid_certs"].as_bool().unwrap_or(false),
    })
    .unwrap();
    let page = api.get_article_page(ArticleFeed::Latest, 1).await.unwrap();
    assert!(!page.articles.is_empty());
    assert_eq!(page.categories.len(), 9);
    assert!(page.articles.iter().all(|a| a.thumbnail.is_some()));
    assert!(!page.monthly_ranking.is_empty());
    let article = api.get_article(page.articles[0].id).await.unwrap();
    assert!(!article.blocks.is_empty());
    let next = api
        .get_next_article_page(page.next_url.unwrap())
        .await
        .unwrap();
    assert!(!next.articles.is_empty());
    let tags = api.get_tag_directory().await.unwrap();
    assert!(!tags.groups.is_empty());
    assert!(tags.groups.iter().any(|g| !g.nodes.is_empty()));
    for category in [
        Category::Illustration,
        Category::Manga,
        Category::Novel,
        Category::Tutorial,
        Category::Making,
        Category::Materials,
        Category::Interview,
        Category::Column,
        Category::News,
    ] {
        assert!(
            !api.get_article_page(ArticleFeed::Category { category }, 1)
                .await
                .unwrap()
                .articles
                .is_empty(),
            "{category:?}"
        );
    }
    for language in [
        Language::Japanese,
        Language::English,
        Language::SimplifiedChinese,
        Language::TraditionalChinese,
        Language::Korean,
        Language::Thai,
        Language::Malay,
    ] {
        let page = api
            .get_next_article_page(format!("https://www.pixivision.net/{}/", language.path()))
            .await
            .unwrap();
        assert!(!page.articles.is_empty(), "{language:?}");
    }
    for id in [11772, 12013, 539] {
        let article = api.get_article(id).await.unwrap();
        assert!(!article.blocks.is_empty());
        assert!(!article.translations.is_empty());
        assert!(!article.tags.is_empty());
        assert!(article.thumbnail.is_some());
        if id == 11772 {
            assert!(article.blocks.iter().any(|b| !b.works.is_empty()));
        }
        if id == 12013 {
            assert!(article.blocks.iter().any(|b| b.kind == BlockKind::Question));
        }
    }
    assert!(
        !api.get_article_page(
            ArticleFeed::Search {
                keyword: "猫".into()
            },
            1
        )
        .await
        .unwrap()
        .articles
        .is_empty()
    );
    assert!(
        !api.get_article_page(ArticleFeed::Tag { tag_id: 19 }, 1)
            .await
            .unwrap()
            .articles
            .is_empty()
    );
    assert!(
        api.get_article_page(
            ArticleFeed::Search {
                keyword: "freepivnonexistentqz123456789".into()
            },
            1
        )
        .await
        .unwrap()
        .articles
        .is_empty()
    );
}

#[test]
fn tag_tree_preserves_groups_counts_and_children() {
    let html = r#"<main class="_tag-list-page"><section class="tlp__section"><h2 class="tlp__section-heading">Themes</h2><ul class="tlc__list"><li class="tlc__node"><span class="tlc__name">Animals<span class="tlc__count">(12)</span></span><ul class="tlc__children"><li class="tlc__node" data-count="5"><a class="tlc__name" href="/en/t/19">Cat<span class="tlc__count">5</span></a></li></ul></li></ul></section></main>"#;
    let directory = parse_tag_directory(html, "https://www.pixivision.net/en/t").unwrap();
    let group = &directory.groups[0].nodes[0];
    assert_eq!(group.name, "Animals");
    assert_eq!(group.article_count, Some(12));
    assert_eq!(group.children[0].tag.as_ref().unwrap().id, 19);
    assert_eq!(group.children[0].name, "Cat");
}

#[test]
fn legacy_work_links_and_unknown_content_survive() {
    let html = r#"<article class="am__article-body-container"><h1 class="am__title">Legacy</h1><time datetime="2015-01-01"></time><div class="am__body"><div class="am__work"><h3 class="am__work__title"><a href="https://www.pixiv.net/member_illust.php?mode=medium&amp;illust_id=42">Drawing</a></h3><div class="am__work__user-name"><a href="https://www.pixiv.net/member.php?id=7">Artist</a></div></div><div class="future-block"><a href="https://www.pixiv.net/novel/show.php?id=99">Story</a><ruby>字<rt>reading</rt></ruby></div></div></article>"#;
    let article = parse_article(html, "https://www.pixivision.net/en/a/1").unwrap();
    assert_eq!(article.blocks[0].works[0].id, 42);
    assert_eq!(article.blocks[0].works[0].user_id, Some(7));
    assert_eq!(article.blocks[1].works[0].kind, FeaturedWorkKind::Novel);
    assert!(article.blocks[1].html.contains("<ruby>"));
}
