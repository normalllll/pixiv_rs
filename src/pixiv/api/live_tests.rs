use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct LiveConfig {
    account: UserAccountResult,
    proxy: Option<String>,
    #[serde(default)]
    accept_invalid_certs: bool,
}

fn live_api() -> PixivApi {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".local/session.json");
    let text = std::fs::read_to_string(path).expect("Create ignored .local/session.json first");
    let session: LiveConfig =
        serde_json::from_str(&text).unwrap_or_else(|_| panic!("Invalid local session format"));
    let mut config = PixivApiConfig::new(
        String::new(),
        "en-US".into(),
        "api-tests".into(),
        Some(session.account),
        session.accept_invalid_certs,
    );
    config.proxy = session.proxy;
    PixivApi::new(config)
}

async fn checked<T>(result: Result<T, PixivError>, label: &str) -> T {
    // Never print response bodies, account data, tags, or URLs from private lists.
    let value = result.unwrap_or_else(|e| panic!("{label}: {:?}, status {:?}", e.kind, e.status));
    println!("PASS {label}");
    tokio::time::sleep(Duration::from_millis(700)).await;
    value
}

#[tokio::test]
#[ignore = "Uses a real Pixiv account and network; run explicitly"]
async fn live_read_only() {
    let api = live_api();
    let user_id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    checked(api.get_user_detail(user_id).await, "session validation").await;
    checked(api.get_mypixiv_user_page(user_id).await, "MyPixiv friends").await;
    for restrict in [Restrict::Public, Restrict::Private] {
        let options = BookmarkPageOptions {
            restrict,
            ..Default::default()
        };
        let illusts = checked(
            api.get_user_illust_bookmark_page_with_options(user_id, options.clone())
                .await,
            "illustration bookmarks",
        )
        .await;
        if let Some(url) = illusts.next_url {
            checked(
                api.get_next_illust_page(url).await,
                "illustration bookmark pagination",
            )
            .await;
        }
        let novels = checked(
            api.get_user_novel_bookmark_page_with_options(user_id, options)
                .await,
            "novel bookmarks",
        )
        .await;
        if let Some(url) = novels.next_url {
            checked(
                api.get_next_novel_page(url).await,
                "novel bookmark pagination",
            )
            .await;
        }
        for is_novel in [false, true] {
            let tags = checked(
                api.get_bookmark_tag_page(user_id, BookmarkTagOptions { restrict, is_novel })
                    .await,
                "bookmark tags",
            )
            .await;
            let no_tags = tags.bookmark_tags.is_empty();
            let tag = tags
                .bookmark_tags
                .first()
                .map(|tag| tag.name.clone())
                .unwrap_or_else(|| "__pixiv_rs_live_test_no_match__".into());
            let options = BookmarkPageOptions {
                restrict,
                tag: Some(tag),
                max_bookmark_id: None,
            };
            if is_novel {
                let page = checked(
                    api.get_user_novel_bookmark_page_with_options(user_id, options)
                        .await,
                    "novel tag filter",
                )
                .await;
                if no_tags {
                    assert!(
                        page.novels.is_empty(),
                        "Unknown bookmark tag returned results"
                    );
                }
            } else {
                let page = checked(
                    api.get_user_illust_bookmark_page_with_options(user_id, options)
                        .await,
                    "illustration tag filter",
                )
                .await;
                if no_tags {
                    assert!(
                        page.illusts.is_empty(),
                        "Unknown bookmark tag returned results"
                    );
                }
            }
        }
    }
    // Fixed past date avoids requesting today's ranking before publication.
    let date = "2026-09-10".to_owned();
    let illusts = checked(
        api.get_illust_ranking_page_on_date(IllustRankingMode::Day, date.clone())
            .await,
        "historical illustration ranking",
    )
    .await;
    let historical_manga = checked(
        api.get_manga_ranking_page_on_date(MangaRankingMode::Day, date.clone())
            .await,
        "historical manga ranking",
    )
    .await;
    let novels = checked(
        api.get_novel_ranking_page_on_date(NovelRankingMode::Day, date)
            .await,
        "historical novel ranking",
    )
    .await;
    assert!(
        !historical_manga.illusts.is_empty(),
        "Historical manga ranking is empty"
    );
    assert!(!illusts.illusts.is_empty(), "Historical ranking is empty");
    checked(
        api.get_user_related_page(0, illusts.illusts[0].user.id)
            .await,
        "related users",
    )
    .await;
    let illust_id = illusts.illusts[0].id;
    let novel_id = novels
        .novels
        .first()
        .expect("Historical novel ranking is empty")
        .id;
    checked(
        api.get_illust_bookmark_detail(illust_id).await,
        "illustration bookmark detail",
    )
    .await;
    checked(
        api.get_novel_bookmark_detail(novel_id).await,
        "novel bookmark detail",
    )
    .await;
    let novels = checked(
        api.get_novel_ranking_page(NovelRankingMode::Day).await,
        "current novel ranking for series",
    )
    .await;
    let novel_series_id = novels
        .novels
        .iter()
        .find_map(|n| n.series.id)
        .expect("No novel series in ranking");
    let series = checked(
        api.get_novel_series_page(novel_series_id).await,
        "novel series",
    )
    .await;
    assert_eq!(series.novel_series_detail.id, novel_series_id);
    if let Some(url) = series.next_url {
        let next = checked(
            api.get_next_novel_series_page(url).await,
            "novel series pagination",
        )
        .await;
        assert_eq!(next.novel_series_detail.id, novel_series_id);
    } else {
        println!("SKIP novel series pagination: single page");
    }
    let manga = checked(
        api.get_manga_ranking_page(MangaRankingMode::Day).await,
        "current manga ranking for series",
    )
    .await;
    let manga_series_id = manga
        .illusts
        .iter()
        .find_map(|i| i.series.as_ref().and_then(|s| s.id))
        .expect("No manga series in ranking");
    let series = checked(
        api.get_illust_series_page(manga_series_id).await,
        "manga series",
    )
    .await;
    assert_eq!(series.illust_series_detail.id, manga_series_id);
    if let Some(url) = series.next_url {
        let next = checked(
            api.get_next_illust_series_page(url).await,
            "manga series pagination",
        )
        .await;
        assert_eq!(next.illust_series_detail.id, manga_series_id);
    } else {
        println!("SKIP manga series pagination: single page");
    }
    for mode in [SearchAiMode::Hide, SearchAiMode::Show] {
        checked(
            api.get_search_illust_page_with_ai(
                "風景".into(),
                SearchSort::DateDesc,
                SearchTarget::PartialMatchForTags,
                SearchOptions::default(),
                Some(mode),
            )
            .await,
            "AI search filter",
        )
        .await;
    }
}

#[tokio::test]
#[ignore = "Uses a real Pixiv account and network; run explicitly"]
async fn live_spotlight() {
    use crate::pixivision::SpotlightCategory;
    let api = live_api();
    let user_id = api.account().unwrap().user.id.parse::<u64>().unwrap();
    checked(api.get_user_detail(user_id).await, "session validation").await;
    for category in [SpotlightCategory::All, SpotlightCategory::Manga] {
        let page = checked(
            api.get_spotlight_article_page(category).await,
            "Spotlight list",
        )
        .await;
        assert!(!page.spotlight_articles.is_empty());
        if let Some(url) = page.next_url {
            let next = checked(
                api.get_next_spotlight_article_page(url).await,
                "Spotlight pagination",
            )
            .await;
            assert!(!next.spotlight_articles.is_empty());
        }
    }
}
