use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Note {
    pub id: u64,
    pub created_ms: u128,
    pub updated_ms: u128,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub content: String,
    pub changes: Vec<Change>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Change {
    pub change_date_ms: u128,
    pub pin_change: PinChange,
    pub tag_change: TagChange,
    pub content_change: ContentChange,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PinChange {
    pub before: bool,
    pub after: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TagChange {
    pub before: Vec<String>,
    pub after: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentChange {
    pub before: String,
    pub after: String,
}