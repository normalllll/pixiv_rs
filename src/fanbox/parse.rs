use super::*;
use crate::{PixivError, PixivErrorKind};
use serde_json::Value;

pub(super) fn string(v: &Value, key: &str) -> String {
    match &v[key] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}
pub(super) fn optional(v: &Value, key: &str) -> Option<String> {
    let s = string(v, key);
    (!s.is_empty()).then_some(s)
}
fn number(v: &Value, key: &str) -> u64 {
    v[key]
        .as_u64()
        .or_else(|| v[key].as_str()?.parse().ok())
        .unwrap_or(0)
}
fn flag(v: &Value, key: &str) -> bool {
    v[key].as_bool().unwrap_or(false)
}
pub(super) fn array<'a>(v: &'a Value, key: &str) -> &'a [Value] {
    v[key].as_array().map(Vec::as_slice).unwrap_or(&[])
}
pub(super) fn malformed() -> PixivError {
    PixivError::new(
        PixivErrorKind::Json,
        "Unexpected FANBOX response structure".into(),
    )
}
pub(super) fn items<'a>(v: &'a Value, keys: &[&str]) -> Result<&'a [Value], PixivError> {
    if let Some(a) = v.as_array() {
        return Ok(a);
    }
    for key in keys {
        if let Some(a) = v[*key].as_array() {
            return Ok(a);
        }
    }
    Err(malformed())
}
pub(super) fn user(v: &Value) -> FanboxUser {
    FanboxUser {
        id: string(v, "userId"),
        name: string(v, "name"),
        icon_url: optional(v, "iconUrl"),
    }
}
pub(super) fn creator(v: &Value) -> Result<FanboxCreator, PixivError> {
    let id = optional(v, "creatorId").ok_or_else(malformed)?;
    Ok(FanboxCreator {
        creator_id: id,
        user: user(&v["user"]),
        description: string(v, "description"),
        cover_url: optional(v, "coverImageUrl"),
        is_followed: flag(v, "isFollowed"),
        is_supported: flag(v, "isSupported"),
        has_adult_content: flag(v, "hasAdultContent"),
        profile_links: array(v, "profileLinks")
            .iter()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect(),
        profile_images: array(v, "profileItems")
            .iter()
            .filter_map(|x| optional(x, "imageUrl"))
            .collect(),
    })
}
fn image(v: &Value) -> FanboxImage {
    FanboxImage {
        id: string(v, "id"),
        original_url: string(v, "originalUrl"),
        thumbnail_url: string(v, "thumbnailUrl"),
        width: number(v, "width") as u32,
        height: number(v, "height") as u32,
        extension: string(v, "extension"),
    }
}
fn file(v: &Value) -> FanboxFile {
    FanboxFile {
        id: string(v, "id"),
        name: string(v, "name"),
        extension: string(v, "extension"),
        size: number(v, "size"),
        url: string(v, "url"),
    }
}
fn embed(v: &Value) -> FanboxBlock {
    let provider = string(v, "serviceProvider");
    let id = optional(v, "videoId").or_else(|| optional(v, "contentId"));
    let url = optional(v, "url")
        .or_else(|| {
            optional(v, "html").and_then(|h| {
                let doc = scraper::Html::parse_fragment(&h);
                let selector = scraper::Selector::parse("iframe[src], video[src], audio[src]")
                    .expect("static selector");
                doc.select(&selector)
                    .next()
                    .and_then(|e| e.attr("src"))
                    .map(str::to_owned)
            })
        })
        .or_else(|| {
            id.and_then(|id| match provider.as_str() {
                "youtube" => Some(format!("https://www.youtube.com/watch?v={id}")),
                "vimeo" => Some(format!("https://vimeo.com/{id}")),
                "soundcloud" => Some(format!("https://soundcloud.com/{id}")),
                _ => None,
            })
        });
    FanboxBlock::Embed {
        url,
        html: optional(v, "html").map(|h| ammonia::clean(&h)),
    }
}
fn paragraph(v: &Value) -> FanboxBlock {
    let mut spans = Vec::new();
    for x in array(v, "styles") {
        spans.push(FanboxTextSpan {
            offset: number(x, "offset") as u32,
            length: number(x, "length") as u32,
            bold: string(x, "type") == "bold",
            italic: string(x, "type") == "italic",
            url: None,
        });
    }
    for x in array(v, "links") {
        spans.push(FanboxTextSpan {
            offset: number(x, "offset") as u32,
            length: number(x, "length") as u32,
            bold: false,
            italic: false,
            url: optional(x, "url"),
        });
    }
    FanboxBlock::Paragraph {
        text: string(v, "text"),
        spans,
    }
}
fn blocks(body: &Value) -> Vec<FanboxBlock> {
    let mut result = Vec::new();
    if let Some(text) = optional(body, "text") {
        result.push(FanboxBlock::Paragraph {
            text,
            spans: vec![],
        })
    }
    for x in array(body, "blocks") {
        let block = match string(x, "type").as_str() {
            "p" => paragraph(x),
            "header" => FanboxBlock::Heading {
                text: string(x, "text"),
            },
            "image" if body["imageMap"][string(x, "imageId")].is_object() => FanboxBlock::Image {
                image: image(&body["imageMap"][string(x, "imageId")]),
            },
            "file" if body["fileMap"][string(x, "fileId")].is_object() => FanboxBlock::File {
                file: file(&body["fileMap"][string(x, "fileId")]),
            },
            "embed" => embed(&body["embedMap"][string(x, "embedId")]),
            "url_embed" => {
                let item = &body["urlEmbedMap"][string(x, "urlEmbedId")];
                if item["postInfo"].is_object() {
                    let post = &item["postInfo"];
                    FanboxBlock::PostLink {
                        post_id: string(post, "id"),
                        creator_id: string(post, "creatorId"),
                        title: string(post, "title"),
                    }
                } else {
                    embed(item)
                }
            }
            _ => FanboxBlock::Unknown {
                text: string(x, "text"),
                raw_json: x.to_string(),
            },
        };
        result.push(block);
    }
    result.extend(
        array(body, "images")
            .iter()
            .map(|v| FanboxBlock::Image { image: image(v) }),
    );
    result.extend(
        array(body, "files")
            .iter()
            .map(|v| FanboxBlock::File { file: file(v) }),
    );
    if body["video"].is_object() {
        result.push(embed(&body["video"]))
    }
    if body["html"].is_string() {
        result.push(embed(body))
    }
    result
}
pub(super) fn post(value: &Value) -> Result<FanboxPost, PixivError> {
    let v = if value["post"].is_object() {
        &value["post"]
    } else {
        value
    };
    let id = optional(v, "id").ok_or_else(malformed)?;
    let restricted = flag(v, "isRestricted");
    let body = &v["body"];
    let blocks = if restricted { vec![] } else { blocks(body) };
    let unknown_body_json = (!restricted
        && !body.is_null()
        && (blocks.is_empty()
            || !matches!(
                string(v, "type").as_str(),
                "article" | "image" | "file" | "text" | "video" | "entry"
            )))
    .then(|| body.to_string());
    Ok(FanboxPost {
        id,
        creator_id: string(v, "creatorId"),
        user: user(&v["user"]),
        title: string(v, "title"),
        excerpt: string(v, "excerpt"),
        cover_url: optional(v, "coverImageUrl")
            .or_else(|| optional(&v["cover"], "url"))
            .or_else(|| optional(v, "imageForShare")),
        published_datetime: string(v, "publishedDatetime"),
        updated_datetime: string(v, "updatedDatetime"),
        fee_required: number(v, "feeRequired"),
        is_restricted: restricted,
        is_liked: flag(v, "isLiked"),
        like_count: number(v, "likeCount"),
        comment_count: number(v, "commentCount"),
        tags: array(v, "tags")
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        blocks,
        previous_post_id: optional(&v["prevPost"], "id"),
        next_post_id: optional(&v["nextPost"], "id"),
        unknown_body_json,
    })
}
pub(super) fn post_page(v: &Value) -> Result<FanboxPostPage, PixivError> {
    let list = if v["posts"].is_object() {
        &v["posts"]
    } else {
        v
    };
    Ok(FanboxPostPage {
        posts: items(list, &["items", "posts"])?
            .iter()
            .map(post)
            .collect::<Result<_, _>>()?,
        next_url: optional(list, "nextUrl"),
        next_page: list["nextPage"].as_u64().map(|x| x as u32),
    })
}
pub(super) fn plan(v: &Value) -> FanboxPlan {
    FanboxPlan {
        id: string(v, "id"),
        creator_id: string(v, "creatorId"),
        user: user(&v["user"]),
        title: string(v, "title"),
        description: string(v, "description"),
        fee: number(v, "fee"),
        cover_url: optional(v, "coverImageUrl"),
    }
}
pub(super) fn tag(v: &Value) -> FanboxTag {
    FanboxTag {
        name: v
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| optional(v, "tag").unwrap_or_else(|| string(v, "name"))),
        count: v["count"].as_u64(),
    }
}
pub(super) fn comment(v: &Value) -> FanboxComment {
    FanboxComment {
        id: string(v, "id"),
        user: user(&v["user"]),
        body: string(v, "body"),
        created_datetime: string(v, "createdDatetime"),
        is_liked: flag(v, "isLiked"),
        is_own: flag(v, "isOwn"),
        like_count: number(v, "likeCount"),
        replies: array(v, "replies").iter().map(comment).collect(),
    }
}
pub(super) fn notice(v: &Value) -> FanboxNotice {
    let user = if v["creator"]["user"].is_object() {
        &v["creator"]["user"]
    } else if v["post"]["user"].is_object() {
        &v["post"]["user"]
    } else {
        &v["user"]
    };
    FanboxNotice {
        id: string(v, "id"),
        kind: optional(v, "type").unwrap_or_else(|| "newsletter".into()),
        user_name: optional(v, "userName").unwrap_or_else(|| string(user, "name")),
        title: optional(v, "postTitle")
            .or_else(|| optional(v, "title"))
            .or_else(|| optional(&v["post"], "title"))
            .unwrap_or_default(),
        body: optional(v, "postCommentBody")
            .or_else(|| optional(v, "body"))
            .or_else(|| optional(v, "text"))
            .unwrap_or_default(),
        date: optional(v, "notifiedDatetime")
            .or_else(|| optional(v, "publishedDatetime"))
            .or_else(|| optional(v, "createdDatetime"))
            .or_else(|| optional(v, "createdAt"))
            .unwrap_or_default(),
        is_unread: v["isUnread"]
            .as_bool()
            .unwrap_or_else(|| v["isRead"].as_bool().is_some_and(|read| !read)),
        post_id: optional(v, "postId").or_else(|| optional(&v["post"], "id")),
        creator_id: optional(v, "creatorId").or_else(|| optional(&v["creator"], "creatorId")),
        icon_url: optional(v, "userProfileImg").or_else(|| optional(user, "iconUrl")),
    }
}
