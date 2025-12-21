use crate::util::json_escape;

#[derive(Clone, Debug)]
pub struct Note {
    pub id: u64,
    pub created_ms: u128,
    pub updated_ms: u128,
    pub pinned: bool,
    pub tags: Vec<String>,
    pub content: String,
}

pub fn note_to_json(note: &Note) -> String {
    let tags_json = note.tags.iter().map(|t| format!("\"{}\"", json_escape(t))).collect::<Vec<_>>().join(",");

    format!("{{\"id\":{},\"created_ms\":{},\"updated_ms\":{},\"pinned\":{},\"tags\":[{}],\"content\":\"{}\"}}", note.id, note.created_ms, note.updated_ms, if note.pinned { "true" } else { "false" }, tags_json, json_escape(&note.content))
}
