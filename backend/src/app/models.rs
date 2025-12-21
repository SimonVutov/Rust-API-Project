use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub created_ms: u128,
    pub updated_ms: u128,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub content: String,
}
