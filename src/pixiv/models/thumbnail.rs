use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thumbnail {
    pub sq800: ThumbnailUrl,
    pub w160: ThumbnailUrl,
    pub w400: ThumbnailUrl,
    pub w1280: ThumbnailUrl,
    pub original: ThumbnailUrl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThumbnailUrl {
    pub url: String,
}
