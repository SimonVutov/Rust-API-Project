use rusthttp::{Router, Method, serve};

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.add_route(Method::Get, "/health", |_req, stream| {
        rusthttp::write_response(stream, 200, "OK", "text/plain", b"ok")
    });

    println!("Starting example server on 127.0.0.1:8081");
    serve("127.0.0.1:8081", router)
}
