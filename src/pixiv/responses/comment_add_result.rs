use serde::{Deserialize, Serialize};

use crate::models::Comment;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommentAddResult {
    pub comment: Comment,
}
