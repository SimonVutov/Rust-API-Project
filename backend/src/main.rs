use serde::Deserialize;
use std::{cmp::Ordering, path::Path};
use std::sync::{Arc, Mutex};

mod app;
mod util;

use crate::app::*;
use rusthttp::{Request, Router, Method, write_response, serve};
use crate::util::*;
use bcrypt;
use rand::{RngCore, rngs::OsRng};

#[derive(Deserialize)]
struct NoteCreate {
    content: Option<String>,
    pinned: Option<bool>,
    tags: Option<Vec<String>>,
    session_token: String,
}

#[derive(Deserialize)]
struct NotePatch {
    content: Option<String>,
    pinned: Option<bool>,
    tags: Option<Vec<String>>,
}

struct CheckSessionTokenResponse {
    valid: bool,
    username: String,
}

fn check_session_token(token: &str, sessions: &Arc<Mutex<Vec<Session>>>) -> CheckSessionTokenResponse {
    for session in sessions.lock().unwrap().iter() {
        if session.session_token == token && session.expires_at_ms > now_ms() {
            return CheckSessionTokenResponse { valid: true, username: session.username.clone() };
        }
    }
    CheckSessionTokenResponse { valid: false, username: String::new() }
}

fn get_bearer_token(req: &Request) -> Option<String> {
    let h = req.headers.get("authorization")?;
    let h = h.trim();
    h.strip_prefix("Bearer ").or_else(|| h.strip_prefix("bearer ")).map(|s| s.to_string())
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
    let sessions_path = sessions_path();
    let initial_sessions = match load_sessions(&data_path) {
        Ok(sessions) => sessions,
        Err(e) => {
            eprintln!("failed to load sessions: {}", e);
            Vec::new()
        }
    };

    let notes: Arc<Mutex<Vec<Note>>> = Arc::new(Mutex::new(initial_notes));
    let sessions: Arc<Mutex<Vec<Session>>> = Arc::new(Mutex::new(initial_sessions));

    let mut router = Router::new();

    let sessions_for_get_notes = Arc::clone(&sessions);

    router.add_route(Method::Get, "/health", |_req, stream| write_response(stream, 200, "OK", "text/plain", b"ok"));

    let notes_list = Arc::clone(&notes);
    router.add_route(Method::Get, "/api/notes", move |req, stream| {
        let token = match get_bearer_token(&req) {
            Some(t) => t,
            None => return write_response(stream, 401, "Unauthorized", "application/json", b"{\"error\":\"missing authorization header\"}"),
        };

        let session_check = check_session_token(&token, &sessions_for_get_notes);
        if !session_check.valid {
            return write_response(stream, 401, "Unauthorized", "application/json", b"{\"error\":\"invalid session token\"}");
        }

        let notes = notes_list.lock().unwrap();
        let mut ordered: Vec<&Note> = notes.iter().filter(|n| n.username == session_check.username).collect();
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

    let sessions_for_post_notes = Arc::clone(&sessions);
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

        let session_check = check_session_token(&payload.session_token, &sessions_for_post_notes);
        if !session_check.valid {
            return write_response(stream, 401, "Unauthorized", "application/json", b"{\"error\":\"invalid session token\"}");
        }

        let note = Note { username: session_check.username, id, created_ms: t, updated_ms: t, pinned, tags, content, changes: Vec::new() };

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

    router.add_route(Method::Post, "/api/signup", move |req, stream| {
        let payload = match serde_json::from_slice::<SignPayload>(&req.body) {
            Ok(payload) => payload,
            Err(_) => return write_response(stream, 400, "Bad Request", "application/json", b"{\"error\":\"invalid json\"}"),
        };

        let hashed_password = match bcrypt::hash(&payload.password, bcrypt::DEFAULT_COST) {
            Ok(h) => h,
            Err(_) => return write_response(stream, 500, "Internal Server Error", "application/json", b"{\"error\":\"hash failed\"}"),
        };

        let user_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data").join("users.json");
        if let Err(e) = save_user(&user_path, &payload.username, &hashed_password) {
            eprintln!("failed to save user: {}", e);
            return write_response(stream, 500, "Internal Server Error", "application/json", b"{\"error\":\"internal server error\"}");
        }
        write_response(stream, 200, "OK", "application/json", b"{\"status\":\"user created\"}")
    });

    let sessions_for_post_signin = Arc::clone(&sessions);
    router.add_route(Method::Post, "/api/signin", move |req, stream| {
        let payload = match serde_json::from_slice::<SignPayload>(&req.body) {
            Ok(payload) => payload,
            Err(_) => return write_response(stream, 400, "Bad Request", "application/json", b"{\"error\":\"invalid json\"}"),
        };

        let user_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data").join("users.json");
        let check_user_response = check_user(&user_path, &payload.username, &payload.password);

        if check_user_response.exists == false || check_user_response.correct_password == false {
            return write_response(stream, 401, "Unauthorized", "application/json", b"{\"error\":\"invalid credentials\"}");
        }

        // 32 random bytes -> hex string token
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let session_token: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();

        sessions.lock().unwrap().push(Session {
            username: payload.username.clone(),
            session_token: session_token.clone(),
            expires_at_ms: now_ms() + 3600 * 1000, // 1 hour
        });

        let body = serde_json::json!({
            "status": "logged in",
            "session_token": session_token,
            "expires_at_ms": now_ms() + 3600 * 1000
        })
        .to_string();

        if let Err(e) = save_sessions(&sessions_path, &sessions_for_post_signin.lock().unwrap()) {
            eprintln!("failed to save sessions: {}", e);
        }

        write_response(stream, 200, "OK", "application/json", body.as_bytes())
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
