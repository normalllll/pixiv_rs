use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub error: ApiErrorBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiErrorBody {
    #[serde(
        rename = "user_message",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub user_message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Pixiv's per-field messages, keyed by the request field name.
    #[serde(
        rename = "user_message_details",
        default,
        deserialize_with = "deserialize_field_messages",
        skip_serializing_if = "Option::is_none"
    )]
    pub user_message_details: Option<HashMap<String, String>>,
}

/// Reads the details leniently: a string stays, an array of strings is joined
/// with newlines, any other value is skipped, and a non-object becomes `None`.
/// This never fails, so a surprise here cannot hide the rest of the error.
fn deserialize_field_messages<'de, D>(
    deserializer: D,
) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer).unwrap_or(Value::Null);
    let Value::Object(fields) = value else {
        return Ok(None);
    };
    let messages = fields
        .into_iter()
        .filter_map(|(name, value)| {
            let text = match value {
                Value::String(text) => text,
                Value::Array(items) => items
                    .into_iter()
                    .filter_map(|item| match item {
                        Value::String(text) => Some(text),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => return None,
            };
            Some((name, text))
        })
        .collect();
    Ok(Some(messages))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn details(json: &str) -> Option<HashMap<String, String>> {
        serde_json::from_str::<ErrorMessage>(json)
            .unwrap()
            .error
            .user_message_details
    }

    #[test]
    fn keeps_string_field_messages() {
        let map =
            details(r#"{"error":{"user_message_details":{"profile_image":"too big"}}}"#).unwrap();
        assert_eq!(map["profile_image"], "too big");
    }

    #[test]
    fn joins_array_field_messages() {
        let map = details(r#"{"error":{"user_message_details":{"a":["one","two",3]}}}"#).unwrap();
        assert_eq!(map["a"], "one\ntwo");
    }

    #[test]
    fn skips_unknown_value_shapes() {
        let map = details(
            r#"{"error":{"user_message_details":{"a":1,"b":{"x":"y"},"c":null,"d":"ok"}}}"#,
        )
        .unwrap();
        assert_eq!(map.len(), 1);
        assert_eq!(map["d"], "ok");
    }

    #[test]
    fn non_object_details_become_none() {
        assert_eq!(
            details(r#"{"error":{"user_message_details":"text"}}"#),
            None
        );
        assert_eq!(details(r#"{"error":{"user_message_details":[]}}"#), None);
        assert_eq!(details(r#"{"error":{"user_message_details":null}}"#), None);
        assert_eq!(details(r#"{"error":{"message":"m"}}"#), None);
    }
}
