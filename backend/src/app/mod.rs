pub mod models;
pub mod storage;

pub use models::{Note, note_to_json};
pub use storage::{load_notes, notes_path, save_notes};
