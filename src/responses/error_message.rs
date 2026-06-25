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
    #[serde(
        rename = "user_message_details",
        default,
        deserialize_with = "deserialize_json_value_as_string",
        skip_serializing_if = "Option::is_none"
    )]
    pub user_message_details: Option<String>,
}

fn deserialize_json_value_as_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    Ok(value.map(|value| match value {
        Value::String(text) => text,
        other => other.to_string(),
    }))
}
