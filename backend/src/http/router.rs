use crate::app::{Note, save_notes};
use crate::http::Request;
use crate::http::response::write_response;
use crate::util::now_ms;
use serde::Deserialize;
use std::cmp::Ordering;
use std::io;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};

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

pub fn handle_request(req: Request, notes: Arc<Mutex<Vec<Note>>>, data_path: &Path, stream: &mut TcpStream) -> io::Result<()> {
    // Handle CORS preflight
    if req.method == "OPTIONS" {
        return write_response(stream, 204, "No Content", "text/plain", b"");
    }

    // Routing
    match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/health") => {
            return write_response(stream, 200, "OK", "text/plain", b"ok");
        }
        ("GET", "/api/notes") => {
            let notes = notes.lock().unwrap();
            // sort pinned first (updated_ms desc within pinned), then updated_ms desc
            let mut ordered: Vec<&Note> = notes.iter().collect();
            ordered.sort_by(|a, b| match (a.pinned, b.pinned) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => b.updated_ms.cmp(&a.updated_ms),
            });

            let body = match serde_json::to_string(&ordered) {
                Ok(json) => json,
                Err(_) => "[]".to_string(),
            };
            return write_response(stream, 200, "OK", "application/json", body.as_bytes());
        }
        ("POST", "/api/notes") => {
            let payload = match serde_json::from_slice::<NoteCreate>(&req.body) {
                Ok(payload) => payload,
                Err(_) => {
                    return write_response(stream, 400, "Bad Request", "application/json", b"{\"error\":\"invalid json\"}");
                }
            };

            let content = payload.content.unwrap_or_default();
            let pinned = payload.pinned.unwrap_or(false);
            let tags = payload.tags.unwrap_or_else(Vec::new);

            let t = now_ms();
            let id = (t as u64) ^ (t as u64).wrapping_mul(2654435761);

            let note = Note { id, created_ms: t, updated_ms: t, pinned, tags, content };

            {
                let mut notes = notes.lock().unwrap();
                notes.push(note.clone());
                if let Err(e) = save_notes(data_path, &notes) {
                    eprintln!("failed to save notes: {}", e);
                }
            }

            let resp = match serde_json::to_string(&note) {
                Ok(json) => json,
                Err(_) => "{}".to_string(),
            };
            return write_response(stream, 201, "Created", "application/json", resp.as_bytes());
        }
        _ => {}
    }

    // /api/notes/{id} routes
    if req.path.starts_with("/api/notes/") {
        if let Some(id) = parse_note_id(&req.path) {
            match req.method.as_str() {
                "GET" => {
                    let notes = notes.lock().unwrap();
                    if let Some(note) = notes.iter().find(|n| n.id == id) {
                        let resp = match serde_json::to_string(note) {
                            Ok(json) => json,
                            Err(_) => "{}".to_string(),
                        };
                        return write_response(stream, 200, "OK", "application/json", resp.as_bytes());
                    } else {
                        return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}");
                    }
                }
                "PATCH" => {
                    let patch = match serde_json::from_slice::<NotePatch>(&req.body) {
                        Ok(patch) => patch,
                        Err(_) => {
                            return write_response(stream, 400, "Bad Request", "application/json", b"{\"error\":\"invalid json\"}");
                        }
                    };
                    let mut notes = notes.lock().unwrap();
                    if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
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
                        let resp = match serde_json::to_string(note) {
                            Ok(json) => json,
                            Err(_) => "{}".to_string(),
                        };
                        if let Err(e) = save_notes(data_path, &notes) {
                            eprintln!("failed to save notes: {}", e);
                        }
                        return write_response(stream, 200, "OK", "application/json", resp.as_bytes());
                    } else {
                        return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}");
                    }
                }
                "DELETE" => {
                    let mut notes = notes.lock().unwrap();
                    let before = notes.len();
                    notes.retain(|n| n.id != id);
                    if notes.len() == before {
                        return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}");
                    }
                    if let Err(e) = save_notes(data_path, &notes) {
                        eprintln!("failed to save notes: {}", e);
                    }
                    return write_response(stream, 204, "No Content", "text/plain", b"");
                }
                _ => {
                    return write_response(stream, 405, "Method Not Allowed", "application/json", b"{\"error\":\"method not allowed\"}");
                }
            }
        }
    }

    write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}")
}

fn parse_note_id(path: &str) -> Option<u64> {
    // /api/notes/{id}
    let prefix = "/api/notes/";
    if !path.starts_with(prefix) {
        return None;
    }
    let id_str = &path[prefix.len()..];
    id_str.parse::<u64>().ok()
}
