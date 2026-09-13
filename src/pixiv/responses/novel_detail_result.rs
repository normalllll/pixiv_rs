use serde::{Deserialize, Serialize};

use crate::models::Novel;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NovelDetailResult {
    pub novel: Novel,
}
