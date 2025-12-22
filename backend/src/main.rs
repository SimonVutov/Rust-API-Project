use serde::Deserialize;
use std::cmp::Ordering;
use std::sync::{Arc, Mutex};

mod app;
mod http;
mod util;

use crate::app::*;
use crate::http::*;
use crate::util::now_ms;

#[derive(Deserialize)]
struct NoteCreate {
    content: Option<String>,
    pinned: Option<bool>,
    tags: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct NotePatch {
    content: Option<String>,
    pinned: Option<bool>,
    tags: Option<Vec<String>>,
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let data_path = notes_path();
    let initial_notes = match load_notes(&data_path) {
        Ok(notes) => notes,
        Err(e) => {
            eprintln!("failed to load notes: {}", e);
            Vec::new()
        }
    };
    let notes: Arc<Mutex<Vec<Note>>> = Arc::new(Mutex::new(initial_notes));

    let mut router = Router::new();

    router.add_route(Method::Get, "/health", |_req, stream| write_response(stream, 200, "OK", "text/plain", b"ok"));

    let notes_list = Arc::clone(&notes);
    router.add_route(Method::Get, "/api/notes", move |_req, stream| {
        let notes = notes_list.lock().unwrap();
        let mut ordered: Vec<&Note> = notes.iter().collect();
        ordered.sort_by(|a, b| match (a.pinned, b.pinned) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => b.updated_ms.cmp(&a.updated_ms),
        });

        let body = serde_json::to_string(&ordered).unwrap_or_else(|_| "[]".to_string());
        write_response(stream, 200, "OK", "application/json", body.as_bytes())
    });

    let notes_create = Arc::clone(&notes);
    let data_path_create = data_path.clone();
    router.add_route(Method::Post, "/api/notes", move |req, stream| {
        let payload = match serde_json::from_slice::<NoteCreate>(&req.body) {
            Ok(payload) => payload,
            Err(_) => return write_response(stream, 400, "Bad Request", "application/json", b"{\"error\":\"invalid json\"}"),
        };

        let content = payload.content.unwrap_or_default();
        let pinned = payload.pinned.unwrap_or(false);
        let tags = payload.tags.unwrap_or_else(Vec::new);

        let t = now_ms();
        let id = (t as u64) ^ (t as u64).wrapping_mul(2654435761);

        let note = Note { id, created_ms: t, updated_ms: t, pinned, tags, content, changes: Vec::new() };

        {
            let mut notes = notes_create.lock().unwrap();
            notes.push(note.clone());
            if let Err(e) = save_notes(&data_path_create, &notes) {
                eprintln!("failed to save notes: {}", e);
            }
        }

        let resp = serde_json::to_string(&note).unwrap_or_else(|_| "{}".to_string());
        write_response(stream, 201, "Created", "application/json", resp.as_bytes())
    });

    let notes_get_one = Arc::clone(&notes);
    router.add_prefix_route(Method::Get, "/api/notes/", move |req, stream| {
        let id = match parse_note_id(&req.path) {
            Some(id) => id,
            None => return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}"),
        };

        let notes = notes_get_one.lock().unwrap();
        if let Some(note) = notes.iter().find(|n| n.id == id) {
            let resp = serde_json::to_string(note).unwrap_or_else(|_| "{}".to_string());
            write_response(stream, 200, "OK", "application/json", resp.as_bytes())
        } else {
            write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}")
        }
    });

    let notes_patch = Arc::clone(&notes);
    let data_path_patch = data_path.clone();
    router.add_prefix_route(Method::Patch, "/api/notes/", move |req, stream| {
        let id = match parse_note_id(&req.path) {
            Some(id) => id,
            None => return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}"),
        };
        let patch = match serde_json::from_slice::<NotePatch>(&req.body) {
            Ok(patch) => patch,
            Err(_) => return write_response(stream, 400, "Bad Request", "application/json", b"{\"error\":\"invalid json\"}"),
        };

        let before_pin_change = {
            let notes = notes_patch.lock().unwrap();
            notes.iter().find(|n| n.id == id).map(|n| n.pinned)
        };
        let before_tag_change = {
            let notes = notes_patch.lock().unwrap();
            notes.iter().find(|n| n.id == id).map(|n| n.tags.clone())
        };
        let before_content_change = {
            let notes = notes_patch.lock().unwrap();
            notes.iter().find(|n| n.id == id).map(|n| n.content.clone())
        };

        let mut notes = notes_patch.lock().unwrap();
        let note_index = notes.iter().position(|n| n.id == id);
        if let Some(index) = note_index {
            let note = &mut notes[index];
            if let Some(content) = patch.content {
                note.content = content;
            }
            if let Some(pinned) = patch.pinned {
                note.pinned = pinned;
            }
            if let Some(tags) = patch.tags {
                note.tags = tags;
            }
            note.updated_ms = now_ms();
            note.changes.push(Change {
                change_date_ms: now_ms(),
                pin_change: PinChange { before: before_pin_change.unwrap_or(note.pinned), after: note.pinned },
                tag_change: TagChange { before: before_tag_change.unwrap_or_else(|| note.tags.clone()), after: note.tags.clone() },
                content_change: ContentChange { before: before_content_change.unwrap_or_else(|| note.content.clone()), after: note.content.clone() },
            });
            if let Err(e) = save_notes(&data_path_patch, &notes) {
                eprintln!("failed to save notes: {}", e);
            }
            let resp = serde_json::to_string(&notes[index]).unwrap_or_else(|_| "{}".to_string());
            write_response(stream, 200, "OK", "application/json", resp.as_bytes())
        } else {
            write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}")
        }
    });

    let notes_delete = Arc::clone(&notes);
    let data_path_delete = data_path.clone();
    router.add_prefix_route(Method::Delete, "/api/notes/", move |req, stream| {
        let id = match parse_note_id(&req.path) {
            Some(id) => id,
            None => return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}"),
        };

        let mut notes = notes_delete.lock().unwrap();
        let before = notes.len();
        notes.retain(|n| n.id != id);
        if notes.len() == before {
            return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}");
        }
        if let Err(e) = save_notes(&data_path_delete, &notes) {
            eprintln!("failed to save notes: {}", e);
        }
        write_response(stream, 204, "No Content", "text/plain", b"")
    });

    let notes_changes = Arc::clone(&notes);
    router.add_prefix_route(Method::Get, "/api/notes-changes/", move |req, stream| {
        let id = match parse_note_changes_id(&req.path) {
            Some(id) => id,
            None => return write_response(stream, 404, "Not Found", "text/plain", b"invalid note id"),
        };

        let notes = notes_changes.lock().unwrap();
        let note = match notes.iter().find(|n| n.id == id) {
            Some(n) => n,
            None => return write_response(stream, 404, "Not Found", "text/plain", b"note not found"),
        };

        let mut s = String::new();

        for c in note.changes.iter() {
            s.push_str(" => ");
            if c.pin_change.before != c.pin_change.after {
                s.push_str(&format!("Pin changed from {} to {} at {}\n", c.pin_change.before, c.pin_change.after, c.change_date_ms));
            }
            if c.tag_change.before != c.tag_change.after {
                s.push_str(&format!("Tags changed from {:?} to {:?} at {}\n", c.tag_change.before, c.tag_change.after, c.change_date_ms));
            }
            if c.content_change.before != c.content_change.after {
                s.push_str(&format!("Content changed at {}\n", c.change_date_ms));
            }
        }

        write_response(stream, 200, "OK", "text/plain", s.as_bytes())
    });

    serve(addr, router)
}

fn parse_note_changes_id(path: &str) -> Option<u64> {
    let prefix = "/api/notes-changes/";
    if !path.starts_with(prefix) {
        return None;
    }
    let id_str = &path[prefix.len()..];
    if id_str.is_empty() {
        return None;
    }
    id_str.parse::<u64>().ok()
}

fn parse_note_id(path: &str) -> Option<u64> {
    let prefix = "/api/notes/";
    if !path.starts_with(prefix) {
        return None;
    }
    let id_str = &path[prefix.len()..];
    id_str.parse::<u64>().ok()
}

/*
curl -i http://127.0.0.1:8080/health

curl -i -X POST http://127.0.0.1:8080/api/notes \
  -H 'Content-Type: application/json' \
  -d '{"content":"hello from rust","pinned":false,"tags":["rust","notes"]}'

curl -i http://127.0.0.1:8080/api/notes

curl -i -X PATCH http://127.0.0.1:8080/api/notes/{id} \
  -H 'Content-Type: application/json' \
  -d '{"content":"updated","pinned":true,"tags":["pinned"]}'

curl -i -X DELETE http://127.0.0.1:8080/api/notes/{id}
*/
