use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;

#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    #[allow(dead_code)]
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

/// Parses an HTTP request from the given TcpStream.
pub fn parse_http_request(stream: &mut TcpStream) -> std::io::Result<Request> {
    fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack.windows(needle.len()).position(|w| w == needle)
    }
    
    // Read until we have headers, then read body based on Content-Length (if any).
    let mut buf = Vec::<u8>::new();
    let mut tmp = [0u8; 4096];

    // 1) Read until header terminator
    loop {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            // connection closed
            break;
        }
        buf.extend_from_slice(&tmp[..n]);
        if find_subsequence(&buf, b"\r\n\r\n").is_some() {
            break;
        }
        // avoid runaway in this toy server
        if buf.len() > 1024 * 1024 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "headers too large"));
        }
    }

    // If we never found headers end, treat as bad request
    let header_end = find_subsequence(&buf, b"\r\n\r\n").ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid HTTP request"))? + 4;

    let header_bytes = &buf[..header_end];
    let header_text = String::from_utf8_lossy(header_bytes);
    let mut lines = header_text.split("\r\n");

    let request_line = lines.next().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "missing request line"))?;

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();

    if method.is_empty() || path.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "bad request line"));
    }

    let mut headers = HashMap::<String, String>::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }

    let content_length = headers.get("content-length").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);

    let mut body = Vec::<u8>::new();

    // Anything already read after headers is body prefix
    let already = buf.len().saturating_sub(header_end);
    if already > 0 {
        body.extend_from_slice(&buf[header_end..]);
    }

    // Read remaining body
    while body.len() < content_length {
        let n = stream.read(&mut tmp)?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&tmp[..n]);
        if body.len() > 1024 * 1024 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "body too large"));
        }
    }

    Ok(Request { method, path, headers, body })
}
