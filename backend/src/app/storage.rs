use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::Note;

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
    let notes = serde_json::from_str::<Vec<Note>>(&text).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(notes)
}

pub fn save_notes(path: &Path, notes: &[Note]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_string_pretty(notes).map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
    fs::write(path, json.as_bytes())
}
