use serde::{Deserialize, Serialize};

use crate::models::Tag;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchAutocompleteResult {
    pub tags: Vec<Tag>,
}
