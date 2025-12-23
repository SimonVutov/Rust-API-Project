# rusthttp

A tiny, dependency-free HTTP server crate intended for small apps, tests, and learning.

Features:
- Minimal HTTP request parsing
- Small router with exact and prefix matching
- Simple response writer with CORS headers for local development

Usage:

Add to your `Cargo.toml` as a path dependency for now:

```toml
[dependencies]
rusthttp = { path = "rusthttp" }
```

Then:

```rust
use rusthttp::{Router, Method, serve};

fn main() -> std::io::Result<()> {
    let mut router = Router::new();
    router.add_route(Method::Get, "/health", |_req, stream| {
        rusthttp::write_response(stream, 200, "OK", "text/plain", b"ok")
    });
    serve("127.0.0.1:8080", router)
}
```

Run the example:

```bash
cargo run --example basic_server --manifest-path backend/rusthttp/Cargo.toml
```

When ready, consider publishing to crates.io: only publish when API is stable and you have tests, CI and a license file.
