use super::*;
use serde_json::json;

#[test]
fn session_and_pagination_boundaries() {
    assert_eq!(
        super::client::normalize_session(" FANBOXSESSID=sample ").unwrap(),
        "sample"
    );
    assert!(super::client::normalize_session("x; other=y").is_err());
    for url in [
        "https://api.fanbox.cc.evil.test/post.listHome",
        "http://api.fanbox.cc/post.listHome",
        "https://api.fanbox.cc/follow.create",
    ] {
        assert!(super::client::api_url(url, &["post.listHome"]).is_err());
    }
}
#[test]
fn article_order_styles_and_unknown_blocks() {
    let post=super::parse::post(&json!({"id":"1","type":"article","body":{"blocks":[{"type":"header","text":"Title"},{"type":"p","text":"Hello","styles":[{"type":"bold","offset":0,"length":5}]},{"type":"image","imageId":"a"},{"type":"future","text":"Keep"}],"imageMap":{"a":{"id":"a","originalUrl":"https://downloads.fanbox.cc/a.png"}}}})).unwrap();
    assert_eq!(post.blocks.len(), 4);
    assert!(matches!(&post.blocks[1],FanboxBlock::Paragraph{spans,..} if spans[0].bold));
    assert!(matches!(&post.blocks[2],FanboxBlock::Image{image} if image.id=="a"));
    assert!(
        matches!(&post.blocks[3],FanboxBlock::Unknown{raw_json,..} if raw_json.contains("future"))
    );
}
#[test]
fn restricted_posts_do_not_expose_body() {
    let post = super::parse::post(&json!({"id":"1","isRestricted":true,"body":{"text":"hidden"}}))
        .unwrap();
    assert!(post.blocks.is_empty());
    assert!(post.unknown_body_json.is_none());
}
#[test]
fn list_variants_preserve_cursors() {
    for value in [
        json!({"items":[{"id":"1"}],"nextUrl":"https://api.fanbox.cc/post.listHome?maxId=1"}),
        json!({"posts":{"items":[{"id":"1"}],"nextUrl":"https://api.fanbox.cc/post.listCreator?maxId=1"}}),
    ] {
        let page = super::parse::post_page(&value).unwrap();
        assert_eq!(page.posts[0].id, "1");
        assert!(page.next_url.is_some());
    }
}

#[test]
fn plans_and_restricted_cards_preserve_amounts() {
    let value = json!({"id":"7","creatorId":"creator","feeRequired":500,"isRestricted":true,"cover":{"url":"https://downloads.fanbox.cc/cover.png"},"title":"Locked"});
    let post = super::parse::post(&value).unwrap();
    assert_eq!(post.fee_required, 500);
    assert!(post.is_restricted);
    assert!(post.cover_url.is_some());
    let plan = super::parse::plan(
        &json!({"id":"8","creatorId":"creator","fee":500,"title":"Plan","description":"Benefits"}),
    );
    assert_eq!(plan.fee, 500);
    assert_eq!(plan.description, "Benefits");
}

#[test]
fn all_media_body_formats_are_retained() {
    let post=super::parse::post(&json!({"id":"1","type":"image","body":{"text":"caption","images":[{"id":"i","originalUrl":"https://downloads.fanbox.cc/image.gif","extension":"gif"}],"files":[{"id":"f","url":"https://downloads.fanbox.cc/file.zip","name":"archive","size":100}],"video":{"serviceProvider":"youtube","videoId":"example"}}})).unwrap();
    assert!(matches!(&post.blocks[0], FanboxBlock::Paragraph { .. }));
    assert!(matches!(&post.blocks[1], FanboxBlock::Image { .. }));
    assert!(matches!(&post.blocks[2], FanboxBlock::File { .. }));
    assert!(
        matches!(&post.blocks[3],FanboxBlock::Embed{url:Some(url),..} if url.contains("youtube.com"))
    );
    let entry=super::parse::post(&json!({"id":"2","type":"entry","body":{"html":"<script>bad()</script><p>Entry</p><iframe src=\"https://www.youtube.com/embed/example\"></iframe>"}})).unwrap();
    assert!(
        matches!(&entry.blocks[0],FanboxBlock::Embed{url:Some(_),html:Some(html)} if !html.contains("script") && html.contains("Entry"))
    );
}

#[test]
fn messages_keep_sender_date_and_unread_state() {
    let message = super::parse::notice(
        &json!({"id":"1","body":"Message","createdAt":"2026-01-01","isRead":false,"creator":{"creatorId":"creator","user":{"name":"Sender"}}}),
    );
    assert_eq!(message.user_name, "Sender");
    assert_eq!(message.date, "2026-01-01");
    assert!(message.is_unread);
    assert_eq!(message.creator_id.as_deref(), Some("creator"));
}

#[tokio::test]
#[ignore = "Requires user-provided FANBOX session; read-only"]
async fn live_fanbox() {
    let config: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(".local/fanbox-session.json")
            .expect("Provide ignored FANBOX session file"),
    )
    .unwrap();
    let session = config["session_id"]
        .as_str()
        .or_else(|| config["session"].as_str())
        .unwrap_or_default()
        .to_owned();
    let api = FanboxApi::new(
        session,
        config["proxy"].as_str().map(str::to_owned),
        "en-US".into(),
        config["accept_invalid_certs"].as_bool().unwrap_or(false),
    )
    .unwrap();
    api.validate_session().await.unwrap();
    for feed in [FanboxFeed::Home, FanboxFeed::Supporting] {
        let page = api.get_posts(feed).await.unwrap();
        if let Some(url) = page.next_url {
            api.get_next_posts(url).await.unwrap();
        }
    }
    api.get_plans(None).await.unwrap();
    api.get_notices().await.unwrap();
    api.get_messages().await.unwrap();
    api.refresh_csrf_token().await.unwrap();
    let creators = api
        .get_creators(FanboxCreatorList::Recommended)
        .await
        .unwrap();
    let creator = creators.first().expect("recommended creators");
    api.get_creator(creator.creator_id.clone()).await.unwrap();
    api.get_plans(Some(creator.creator_id.clone()))
        .await
        .unwrap();
    api.get_creator_tags(creator.creator_id.clone())
        .await
        .unwrap();
    let page = api
        .get_posts(FanboxFeed::Creator {
            creator_id: creator.creator_id.clone(),
        })
        .await
        .unwrap();
    if let Some(url) = page.next_url {
        api.get_next_posts(url).await.unwrap();
    }
    for summary in page.posts.iter().take(3) {
        let post = api.get_post(summary.id.clone()).await.unwrap();
        assert_eq!(post.id, summary.id);
        if !post.is_restricted {
            api.get_comments(post.id).await.unwrap();
        }
    }
    api.search_creators("illustration".into(), 1).await.unwrap();
    api.search_tags("illustration".into()).await.unwrap();
    let official = api
        .get_posts(FanboxFeed::Creator {
            creator_id: "official".into(),
        })
        .await
        .unwrap();
    let summary = official
        .posts
        .iter()
        .find(|post| !post.is_restricted)
        .expect("official public article");
    let public = api.get_post(summary.id.clone()).await.unwrap();
    assert!(!public.blocks.is_empty());
    let comments = api.get_comments(public.id.clone()).await.unwrap();
    if let Some(url) = comments.next_url {
        api.get_next_comments(url).await.unwrap();
    }
    if let Some(tag) = public.tags.first() {
        api.get_posts(FanboxFeed::Tag {
            tag: tag.clone(),
            creator_id: Some("official".into()),
            page: 1,
        })
        .await
        .unwrap();
    }
    if let Some(FanboxBlock::Image { image }) = public
        .blocks
        .iter()
        .find(|b| matches!(b, FanboxBlock::Image { .. }))
    {
        assert!(
            !api.get_media_bytes(image.original_url.clone())
                .await
                .unwrap()
                .is_empty()
        );
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!(".local/fanbox-test-media-{stamp}.bin");
        api.download_media(image.original_url.clone(), path.clone())
            .await
            .unwrap();
        assert!(std::fs::metadata(path).unwrap().len() > 0);
    }
    println!("FANBOX read-only endpoints passed; no account or response data logged");
}
