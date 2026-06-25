use serde::{Deserialize, Serialize};

use crate::models::Illust;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IllustDetailResult {
    pub illust: Illust,
}
