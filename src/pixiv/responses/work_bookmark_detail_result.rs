use crate::enums::Restrict;
use serde::{Deserialize, Serialize};

/// The App API wraps bookmark state and suggested tags in bookmark_detail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkBookmarkDetailResult {
    pub bookmark_detail: WorkBookmarkDetail,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkBookmarkDetail {
    pub is_bookmarked: bool,
    pub tags: Vec<WorkBookmarkTag>,
    pub restrict: Restrict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkBookmarkTag {
    pub name: String,
    pub is_registered: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retains_registered_and_suggested_tags() {
        let response: WorkBookmarkDetailResult = serde_json::from_str(r#"{"bookmark_detail":{"is_bookmarked":true,"restrict":"private","tags":[{"name":"saved","is_registered":true},{"name":"suggested","is_registered":false}]}}"#).unwrap();
        assert!(response.bookmark_detail.tags[0].is_registered);
        assert!(!response.bookmark_detail.tags[1].is_registered);
        assert_eq!(response.bookmark_detail.restrict, Restrict::Private);
        assert!(serde_json::from_str::<WorkBookmarkDetailResult>(r#"{}"#).is_err());
    }
}
