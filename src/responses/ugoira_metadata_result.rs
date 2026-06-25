use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UgoiraMetadataResult {
    #[serde(rename = "ugoira_metadata")]
    pub ugoira_metadata: UgoiraMetadataContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UgoiraMetadataContent {
    #[serde(rename = "zip_urls")]
    pub zip_urls: ZipUrls,
    pub frames: Vec<Frame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZipUrls {
    pub medium: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    pub file: String,
    pub delay: u64,
}
