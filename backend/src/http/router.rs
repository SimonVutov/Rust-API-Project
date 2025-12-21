use crate::app::{Note, note_to_json, save_notes};
use crate::http::Request;
use crate::http::response::write_response;
use crate::util::{json_get_bool, json_get_string, json_get_string_array, now_ms};
use std::cmp::Ordering;
use std::io;
use std::net::TcpStream;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
            let mut idxs: Vec<usize> = (0..notes.len()).collect();
            idxs.sort_by(|&a, &b| {
                let na = &notes[a];
                let nb = &notes[b];
                match (na.pinned, nb.pinned) {
                    (true, false) => Ordering::Less,
                    (false, true) => Ordering::Greater,
                    _ => nb.updated_ms.cmp(&na.updated_ms),
                }
            });

            let json_list = idxs.into_iter().map(|i| note_to_json(&notes[i])).collect::<Vec<_>>().join(",");

            let body = format!("[{}]", json_list);
            return write_response(stream, 200, "OK", "application/json", body.as_bytes());
        }
        ("POST", "/api/notes") => {
            let body_str = String::from_utf8_lossy(&req.body);

            let content = json_get_string(&body_str, "content").unwrap_or_default();
            let pinned = json_get_bool(&body_str, "pinned").unwrap_or(false);
            let tags = json_get_string_array(&body_str, "tags").unwrap_or_else(|| vec![]);

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

            let resp = note_to_json(&note);
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
                        let resp = note_to_json(note);
                        return write_response(stream, 200, "OK", "application/json", resp.as_bytes());
                    } else {
                        return write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}");
                    }
                }
                "PATCH" => {
                    let body_str = String::from_utf8_lossy(&req.body);
                    let mut notes = notes.lock().unwrap();
                    if let Some(note) = notes.iter_mut().find(|n| n.id == id) {
                        if let Some(content) = json_get_string(&body_str, "content") {
                            note.content = content;
                        }
                        if let Some(pinned) = json_get_bool(&body_str, "pinned") {
                            note.pinned = pinned;
                        }
                        if let Some(tags) = json_get_string_array(&body_str, "tags") {
                            note.tags = tags;
                        }
                        note.updated_ms = now_ms();
                        let resp = note_to_json(note);
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
