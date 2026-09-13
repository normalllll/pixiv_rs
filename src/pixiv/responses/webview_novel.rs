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

    #[serde(default, deserialize_with = "deserialize_string_vec_like")]
    pub illusts: Vec<String>,

    #[serde(default, deserialize_with = "deserialize_novel_images_as_vec")]
    pub images: Vec<WebviewNovelImage>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series_navigation: Option<NovelNavigationInfo>,

    #[serde(default, deserialize_with = "deserialize_string_vec_like")]
    pub glossary_items: Vec<String>,

    #[serde(default, deserialize_with = "deserialize_string_vec_like")]
    pub replaceable_item_ids: Vec<String>,

    #[serde(default)]
    pub ai_type: i64,

    #[serde(default)]
    pub is_original: bool,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NovelNavigationItem {
    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub id: String,

    #[serde(default)]
    pub title: String,

    #[serde(default, deserialize_with = "deserialize_string_like")]
    pub user_id: String,

    #[serde(default)]
    pub user_name: String,
}

// -----------------------------
// private serde helpers
// -----------------------------

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

fn deserialize_optional_string_like<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
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