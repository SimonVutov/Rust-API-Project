use std::io;
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use crate::{Router, request::parse_http_request, response::write_response};

/// Starts an HTTP server listening on the given address, using the provided router to handle requests.
pub fn serve(addr: &str, router: Router) -> io::Result<()> {
    let listener = TcpListener::bind(addr)?;
    println!("Listening on http://{}", addr);

    let router = Arc::new(router);

    for stream in listener.incoming() {
        let router = Arc::clone(&router);
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    let req = match parse_http_request(&mut stream) {
                        Ok(r) => r,
                        Err(_) => {
                            let _ = write_response(&mut stream, 400, "Bad Request", "application/json", b"{\"error\":\"bad request\"}");
                            return;
                        }
                    };

                    let _ = router.handle(req, &mut stream);
                });
            }
            Err(e) => eprintln!("connection failed: {}", e),
        }
    }

    Ok(())
}
