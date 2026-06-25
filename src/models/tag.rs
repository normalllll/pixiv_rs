use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    #[serde(
        rename = "translated_name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub translated_name: Option<String>,
}
