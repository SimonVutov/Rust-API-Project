use std::io::Write;
use std::net::TcpStream;

/// Writes an HTTP response to the given TcpStream.
pub fn write_response(stream: &mut TcpStream, status_code: u16, status_text: &str, content_type: &str, body: &[u8]) -> std::io::Result<()> {
    // Minimal CORS headers for browser calls
    let headers = format!(
        "HTTP/1.1 {} {}\r\n\
         Content-Type: {}\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         Access-Control-Allow-Origin: http://localhost:3000\r\n\
         Access-Control-Allow-Credentials: true\r\n\
         Access-Control-Allow-Headers: Content-Type, Authorization\r\n\
         Access-Control-Allow-Methods: GET,POST,PATCH,DELETE,OPTIONS\r\n\
         Access-Control-Max-Age: 86400\r\n\
         \r\n",
        status_code,
        status_text,
        content_type,
        body.len()
    );
    stream.write_all(headers.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}
