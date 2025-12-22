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
    pub kind: ChangeKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ChangeKind {
    PinChange { before: bool, after: bool },
    TagChange { before: Vec<String>, after: Vec<String> },
    ContentChange { before: String, after: String },
}
