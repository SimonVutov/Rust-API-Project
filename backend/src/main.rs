use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

mod app;
mod http;
mod util;

use crate::app::*;
use crate::http::*;

fn handle_connection(mut stream: TcpStream, notes: Arc<Mutex<Vec<Note>>>, data_path: PathBuf) {
    thread::spawn(move || {
        let req = match parse_http_request(&mut stream) {
            Ok(r) => r,
            Err(_) => {
                let _ = write_response(&mut stream, 400, "Bad Request", "application/json", b"{\"error\":\"bad request\"}");
                return;
            }
        };

        let _ = handle_request(req, notes, &data_path, &mut stream);
    });
}

fn main() -> std::io::Result<()> {
    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(addr)?;
    println!("Listening on http://{}", addr);

    let data_path = notes_path();
    let initial_notes = match load_notes(&data_path) {
        Ok(notes) => notes,
        Err(e) => {
            eprintln!("failed to load notes: {}", e);
            Vec::new()
        }
    };
    let notes: Arc<Mutex<Vec<Note>>> = Arc::new(Mutex::new(initial_notes));

    for stream in listener.incoming() {
        let notes = Arc::clone(&notes);
        match stream {
            Ok(stream) => handle_connection(stream, notes, data_path.clone()),
            Err(e) => eprintln!("connection failed: {}", e),
        }
    }

    Ok(())
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
