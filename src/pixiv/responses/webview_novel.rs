use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewNovel {
    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub id: String,

    #[serde(default)]
    pub title: String,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_like",
        skip_serializing_if = "Option::is_none"
    )]
    pub series_id: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_title: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_is_watched: Option<bool>,

    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub user_id: String,

    #[serde(default)]
    pub cover_url: String,

    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub caption: String,

    #[serde(default)]
    pub cdate: String,

    #[serde(default)]
    pub rating: NovelRating,

    #[serde(default)]
    pub text: String,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_like",
        skip_serializing_if = "Option::is_none"
    )]
    pub marker: Option<String>,

    #[serde(default, deserialize_with = "deserialize_embedded_works")]
    pub illusts: Vec<WebviewNovelEmbeddedWork>,

    #[serde(default, deserialize_with = "deserialize_novel_images_as_vec")]
    pub images: Vec<WebviewNovelImage>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_navigation: Option<NovelNavigationInfo>,

    #[serde(default, deserialize_with = "deserialize_glossary_items")]
    pub glossary_items: Vec<WebviewNovelGlossaryItem>,

    #[serde(default, deserialize_with = "deserialize_string_vec_like")]
    pub replaceable_item_ids: Vec<String>,

    #[serde(default)]
    pub ai_type: i64,

    #[serde(default)]
    pub is_original: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelRating {
    #[serde(default)]
    pub like: i64,

    #[serde(default)]
    pub bookmark: i64,

    #[serde(default)]
    pub view: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewNovelImage {
    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub novel_image_id: String,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_like",
        skip_serializing_if = "Option::is_none"
    )]
    pub sl: Option<String>,

    #[serde(default)]
    pub urls: WebviewNovelImageUrls,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebviewNovelImageUrls {
    #[serde(rename = "240mw", default, skip_serializing_if = "Option::is_none")]
    pub size_240mw: Option<String>,

    #[serde(rename = "480mw", default, skip_serializing_if = "Option::is_none")]
    pub size_480mw: Option<String>,

    #[serde(rename = "1200x1200", default, skip_serializing_if = "Option::is_none")]
    pub size_1200x1200: Option<String>,

    #[serde(rename = "128x128", default, skip_serializing_if = "Option::is_none")]
    pub size_128x128: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelNavigationInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_novel: Option<NovelNavigationItem>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prev_novel: Option<NovelNavigationItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelNavigationItem {
    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub id: String,

    #[serde(default)]
    pub title: String,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_like",
        skip_serializing_if = "Option::is_none"
    )]
    pub cover_url: Option<String>,

    #[serde(
        default,
        deserialize_with = "deserialize_optional_string_like",
        skip_serializing_if = "Option::is_none"
    )]
    pub content_order: Option<String>,

    #[serde(default = "default_true")]
    pub viewable: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewable_message: Option<String>,
}

fn default_true() -> bool {
    true
}

/// An illustration embedded in a novel body, referenced by `[pixivimage:<key>]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WebviewNovelEmbeddedWork {
    pub key: String,
    pub illust_id: u64,
    pub page: u32,
    pub visible: bool,
    pub available_message: Option<String>,
    pub title: String,
    pub x_restrict: i32,
    pub user_id: String,
    pub user_name: String,
    pub user_image_url: Option<String>,
    pub image_medium: Option<String>,
    pub image_original: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebviewNovelGlossaryItem {
    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub id: String,

    #[serde(default)]
    pub name: String,

    #[serde(default)]
    pub overview: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cover_image: Option<String>,
}

// -----------------------------
// private serde helpers
// -----------------------------

fn value_string(value: Option<&Value>) -> Option<String> {
    match value? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn parse_embedded_work(key: &str, value: &Value) -> Option<WebviewNovelEmbeddedWork> {
    let object = value.as_object()?;
    let (key_id, key_page) = match key.split_once('-') {
        Some((id, page)) => (id.parse::<u64>().ok(), page.parse::<u32>().ok()),
        None => (key.parse::<u64>().ok(), None),
    };
    let illust_id = value_string(object.get("id"))
        .and_then(|id| id.parse::<u64>().ok())
        .or(key_id)?;
    let page = object
        .get("page")
        .and_then(Value::as_u64)
        .and_then(|page| u32::try_from(page).ok())
        .or(key_page)
        .unwrap_or(1);
    let illust = object.get("illust").and_then(Value::as_object);
    let user = object.get("user").and_then(Value::as_object);
    let images = illust
        .and_then(|illust| illust.get("images"))
        .and_then(Value::as_object);
    Some(WebviewNovelEmbeddedWork {
        key: key.to_string(),
        illust_id,
        page,
        visible: object
            .get("visible")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        available_message: value_string(object.get("availableMessage")),
        title: illust
            .and_then(|illust| value_string(illust.get("title")))
            .unwrap_or_default(),
        x_restrict: illust
            .and_then(|illust| illust.get("xRestrict"))
            .and_then(Value::as_i64)
            .and_then(|value| i32::try_from(value).ok())
            .unwrap_or(0),
        user_id: user
            .and_then(|user| value_string(user.get("id")))
            .unwrap_or_default(),
        user_name: user
            .and_then(|user| value_string(user.get("name")))
            .unwrap_or_default(),
        user_image_url: user.and_then(|user| value_string(user.get("image"))),
        image_medium: images.and_then(|images| value_string(images.get("medium"))),
        image_original: images.and_then(|images| value_string(images.get("original"))),
    })
}

fn deserialize_embedded_works<'de, D>(
    deserializer: D,
) -> Result<Vec<WebviewNovelEmbeddedWork>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null);
    match value {
        Value::Object(map) => Ok(map
            .iter()
            .filter_map(|(key, item)| parse_embedded_work(key, item))
            .collect()),
        _ => Ok(Vec::new()),
    }
}

fn deserialize_glossary_items<'de, D>(
    deserializer: D,
) -> Result<Vec<WebviewNovelGlossaryItem>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null);
    match value {
        Value::Array(items) => Ok(items
            .into_iter()
            .filter_map(|item| serde_json::from_value(item).ok())
            .collect()),
        _ => Ok(Vec::new()),
    }
}

fn deserialize_novel_images_as_vec<'de, D>(
    deserializer: D,
) -> Result<Vec<WebviewNovelImage>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null);

    match value {
        Value::Null => Ok(Vec::new()),

        Value::Array(items) => {
            let mut images = Vec::new();

            for item in items {
                match item {
                    Value::Object(_) => {
                        let image: WebviewNovelImage =
                            serde_json::from_value(item).map_err(D::Error::custom)?;
                        images.push(image);
                    }
                    Value::String(url) => {
                        images.push(WebviewNovelImage {
                            novel_image_id: String::new(),
                            sl: None,
                            urls: WebviewNovelImageUrls {
                                original: Some(url),
                                ..Default::default()
                            },
                        });
                    }
                    Value::Null => {}
                    other => {
                        images.push(WebviewNovelImage {
                            novel_image_id: String::new(),
                            sl: None,
                            urls: WebviewNovelImageUrls {
                                original: Some(other.to_string()),
                                ..Default::default()
                            },
                        });
                    }
                }
            }

            Ok(images)
        }

        Value::Object(map) => {
            let mut images = Vec::new();

            for (key, item) in map {
                match item {
                    Value::Object(_) => {
                        let mut image: WebviewNovelImage =
                            serde_json::from_value(item).map_err(D::Error::custom)?;

                        if image.novel_image_id.is_empty() {
                            image.novel_image_id = key;
                        }

                        images.push(image);
                    }
                    Value::String(url) => {
                        images.push(WebviewNovelImage {
                            novel_image_id: key,
                            sl: None,
                            urls: WebviewNovelImageUrls {
                                original: Some(url),
                                ..Default::default()
                            },
                        });
                    }
                    Value::Null => {}
                    other => {
                        images.push(WebviewNovelImage {
                            novel_image_id: key,
                            sl: None,
                            urls: WebviewNovelImageUrls {
                                original: Some(other.to_string()),
                                ..Default::default()
                            },
                        });
                    }
                }
            }

            Ok(images)
        }

        Value::String(url) => Ok(vec![WebviewNovelImage {
            novel_image_id: String::new(),
            sl: None,
            urls: WebviewNovelImageUrls {
                original: Some(url),
                ..Default::default()
            },
        }]),

        _ => Ok(Vec::new()),
    }
}

fn deserialize_string_vec_like<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null);

    match value {
        Value::Null => Ok(Vec::new()),

        Value::Array(items) => {
            let mut result = Vec::new();

            for item in items {
                match item {
                    Value::Null => {}
                    Value::String(s) => result.push(s),
                    Value::Number(n) => result.push(n.to_string()),
                    Value::Bool(b) => result.push(b.to_string()),
                    Value::Array(_) | Value::Object(_) => result.push(item.to_string()),
                }
            }

            Ok(result)
        }

        Value::String(s) => Ok(vec![s]),
        Value::Number(n) => Ok(vec![n.to_string()]),
        Value::Bool(b) => Ok(vec![b.to_string()]),

        Value::Object(_) => Ok(vec![value.to_string()]),
    }
}

fn deserialize_string_like<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null);

    match value {
        Value::Null => Ok(String::new()),
        Value::String(s) => Ok(s),
        Value::Number(n) => Ok(n.to_string()),
        Value::Bool(b) => Ok(b.to_string()),
        Value::Array(_) | Value::Object(_) => Ok(value.to_string()),
    }
}

fn deserialize_optional_string_like<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?.unwrap_or(Value::Null);

    match value {
        Value::Null => Ok(None),
        Value::String(s) => {
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(s))
            }
        }
        Value::Number(n) => Ok(Some(n.to_string())),
        Value::Bool(b) => Ok(Some(b.to_string())),
        Value::Array(_) | Value::Object(_) => Ok(Some(value.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn novel(extra: Value) -> WebviewNovel {
        let mut base = json!({ "id": "1", "title": "Sample" });
        base.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        serde_json::from_value(base).unwrap()
    }

    #[test]
    fn language_is_read() {
        assert_eq!(
            novel(json!({ "language": "zh-cn" })).language.as_deref(),
            Some("zh-cn")
        );
        assert_eq!(novel(json!({})).language, None);
    }

    #[test]
    fn series_navigation_items_are_typed() {
        let parsed = novel(json!({
            "seriesNavigation": {
                "nextNovel": {
                    "id": 29650267,
                    "title": "Next",
                    "coverUrl": "https://example.test/c.jpg",
                    "contentOrder": "12",
                    "viewable": false,
                    "viewableMessage": "locked"
                },
                "prevNovel": null
            }
        }));
        let nav = parsed.series_navigation.unwrap();
        assert!(nav.prev_novel.is_none());
        let next = nav.next_novel.unwrap();
        assert_eq!(next.id, "29650267");
        assert_eq!(
            next.cover_url.as_deref(),
            Some("https://example.test/c.jpg")
        );
        assert_eq!(next.content_order.as_deref(), Some("12"));
        assert!(!next.viewable);
        assert_eq!(next.viewable_message.as_deref(), Some("locked"));
    }

    #[test]
    fn navigation_item_defaults_to_viewable() {
        let parsed = novel(json!({
            "seriesNavigation": { "nextNovel": { "id": "5", "title": "T" } }
        }));
        let next = parsed.series_navigation.unwrap().next_novel.unwrap();
        assert!(next.viewable);
        assert!(next.cover_url.is_none());
        assert!(next.viewable_message.is_none());
    }

    #[test]
    fn embedded_works_are_read_from_keyed_object() {
        let parsed = novel(json!({
            "illusts": {
                "100-1": {
                    "id": "100",
                    "page": 1,
                    "visible": true,
                    "availableMessage": null,
                    "illust": {
                        "title": "Sketch",
                        "xRestrict": 1,
                        "sl": 6,
                        "restrict": 0,
                        "description": "d",
                        "tags": [],
                        "images": {
                            "small": null,
                            "medium": "https://example.test/m.jpg",
                            "original": null
                        }
                    },
                    "user": {
                        "id": "9",
                        "name": "Author",
                        "image": "https://example.test/u.jpg"
                    }
                },
                "200-3": {
                    "visible": false,
                    "availableMessage": "private",
                    "illust": {
                        "title": "Hidden",
                        "images": { "original": "https://example.test/o.png" }
                    }
                }
            }
        }));
        assert_eq!(parsed.illusts.len(), 2);
        let first = parsed.illusts.iter().find(|w| w.key == "100-1").unwrap();
        assert_eq!((first.illust_id, first.page), (100, 1));
        assert!(first.visible);
        assert_eq!(first.title, "Sketch");
        assert_eq!(first.x_restrict, 1);
        assert_eq!(first.user_id, "9");
        assert_eq!(first.user_name, "Author");
        assert_eq!(
            first.user_image_url.as_deref(),
            Some("https://example.test/u.jpg")
        );
        assert_eq!(
            first.image_medium.as_deref(),
            Some("https://example.test/m.jpg")
        );
        assert!(first.image_original.is_none());
        let second = parsed.illusts.iter().find(|w| w.key == "200-3").unwrap();
        assert_eq!((second.illust_id, second.page), (200, 3));
        assert!(!second.visible);
        assert_eq!(second.available_message.as_deref(), Some("private"));
        assert_eq!(
            second.image_original.as_deref(),
            Some("https://example.test/o.png")
        );
    }

    #[test]
    fn empty_embedded_work_shapes_give_empty_list() {
        for shape in [json!([]), json!(null), json!({})] {
            assert!(novel(json!({ "illusts": shape })).illusts.is_empty());
        }
        assert!(novel(json!({})).illusts.is_empty());
    }

    #[test]
    fn malformed_embedded_work_is_skipped() {
        let parsed = novel(json!({
            "illusts": {
                "bad": { "illust": { "title": "x" } },
                "300-1": "not an object",
                "400-1": { "id": "400", "illust": { "title": "Fine" } }
            }
        }));
        assert_eq!(parsed.illusts.len(), 1);
        assert_eq!(parsed.illusts[0].illust_id, 400);
    }

    #[test]
    fn glossary_items_are_typed() {
        let parsed = novel(json!({
            "glossaryItems": [
                { "id": "7", "name": "Term", "overview": "Meaning", "coverImage": null },
                {
                    "id": 8,
                    "name": "Other",
                    "overview": "More",
                    "coverImage": "https://example.test/g.jpg"
                }
            ]
        }));
        assert_eq!(parsed.glossary_items.len(), 2);
        assert_eq!(parsed.glossary_items[0].name, "Term");
        assert!(parsed.glossary_items[0].cover_image.is_none());
        assert_eq!(parsed.glossary_items[1].id, "8");
        assert_eq!(
            parsed.glossary_items[1].cover_image.as_deref(),
            Some("https://example.test/g.jpg")
        );
        assert!(novel(json!({ "glossaryItems": null }))
            .glossary_items
            .is_empty());
    }
}
