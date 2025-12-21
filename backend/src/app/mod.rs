pub mod models;
pub mod storage;

pub use models::Note;
pub use storage::{load_notes, notes_path, save_notes};
