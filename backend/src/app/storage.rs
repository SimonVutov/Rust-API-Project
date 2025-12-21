use crate::util::{json_get_bool, json_get_string, json_get_string_array, json_get_u64, json_get_u128};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::{Note, note_to_json};

pub fn notes_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data").join("note.json")
}

pub fn load_notes(path: &Path) -> io::Result<Vec<Note>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(path)?;
    if text.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut notes = Vec::new();
    for obj in extract_objects(&text) {
        let note = parse_note(obj).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "invalid note data"))?;
        notes.push(note);
    }
    Ok(notes)
}

pub fn save_notes(path: &Path, notes: &[Note]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = notes_to_json(notes);
    fs::write(path, json.as_bytes())
}

fn notes_to_json(notes: &[Note]) -> String {
    let list = notes.iter().map(note_to_json).collect::<Vec<_>>().join(",");
    format!("[{}]", list)
}

fn parse_note(body: &str) -> Option<Note> {
    let id = json_get_u64(body, "id")?;
    let created_ms = json_get_u128(body, "created_ms")?;
    let updated_ms = json_get_u128(body, "updated_ms")?;
    let pinned = json_get_bool(body, "pinned").unwrap_or(false);
    let tags = json_get_string_array(body, "tags").unwrap_or_default();
    let content = json_get_string(body, "content").unwrap_or_default();

    Some(Note { id, created_ms, updated_ms, pinned, tags, content })
}

fn extract_objects(text: &str) -> Vec<&str> {
    let mut objects = Vec::new();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut start: Option<usize> = None;

    for (idx, ch) in text.char_indices() {
        if in_string {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(st) = start {
                            objects.push(&text[st..=idx]);
                            start = None;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    objects
}
