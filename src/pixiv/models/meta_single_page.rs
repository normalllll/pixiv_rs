use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaSinglePage {
    #[serde(
        rename = "original_image_url",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub original_image_url: Option<String>,
}
