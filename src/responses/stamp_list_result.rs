use serde::{Deserialize, Serialize};

use crate::models::Stamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StampListResult {
    pub stamps: Vec<Stamp>,
}
