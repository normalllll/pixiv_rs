use serde::{Deserialize, Serialize};

use super::ImageUrls;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaPage {
    #[serde(rename = "image_urls")]
    pub image_urls: ImageUrls,
}
