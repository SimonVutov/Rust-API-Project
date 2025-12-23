//! rusthttp — a tiny, dependency-free HTTP server crate intended for small apps and examples.
//!
//! # Examples
//!
//! ```rust
//! use rusthttp::{Router, Method, serve};
//!
//! fn main() -> std::io::Result<()> {
//!     let mut router = Router::new();
//!     router.add_route(Method::Get, "/health", |_req, stream| {
//!         rusthttp::write_response(stream, 200, "OK", "text/plain", b"ok")
//!     });
//!
//!     // Blocks and serves requests.
//!     serve("127.0.0.1:8080", router)
//! }
//! ```

pub mod request;
pub mod response;
pub mod router;
pub mod server;

pub use request::{Request, parse_http_request};
pub use response::write_response;
pub use router::{Method, Router};
pub use server::serve;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_simple_get() {
        let mut data = Cursor::new(b"GET /test HTTP/1.1\r\nHost: example\r\n\r\n".to_vec());
        let req = parse_http_request(&mut data).expect("parse");
        assert_eq!(req.method, "GET");
        assert_eq!(req.path, "/test");
        assert_eq!(req.body.len(), 0);
    }

    #[test]
    fn parse_post_with_body() {
        let mut data = Cursor::new(b"POST /echo HTTP/1.1\r\nContent-Length: 5\r\n\r\nhello".to_vec());
        let req = parse_http_request(&mut data).expect("parse");
        assert_eq!(req.method, "POST");
        assert_eq!(req.path, "/echo");
        assert_eq!(req.body, b"hello");
    }

    #[test]
    fn write_response_contains_headers_and_body() {
        let mut out = Vec::new();
        write_response(&mut out, 200, "OK", "text/plain", b"hi").unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("HTTP/1.1 200 OK"));
        assert!(s.contains("Content-Type: text/plain"));
        assert!(s.ends_with("hi"));
    }

    #[test]
    fn router_dispatches() {
        let mut router = Router::new();
        router.add_route(Method::Get, "/x", |_req, stream| {
            write_response(stream, 200, "OK", "text/plain", b"ok")
        });

        let req = Request { method: "GET".into(), path: "/x".into(), headers: std::collections::HashMap::new(), body: Vec::new() };
        let mut out = Vec::new();
        router.handle(req, &mut out).unwrap();
        let s = String::from_utf8_lossy(&out);
        assert!(s.contains("200 OK"));
    }
}
