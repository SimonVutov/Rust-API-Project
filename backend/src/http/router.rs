use std::io;
use std::net::TcpStream;
use crate::http::{Request, response::write_response};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
    Delete,
}

impl Method {
    fn from_str(method: &str) -> Option<Self> {
        match method {
            "GET" => Some(Self::Get),
            "POST" => Some(Self::Post),
            "PATCH" => Some(Self::Patch),
            "DELETE" => Some(Self::Delete),
            _ => None,
        }
    }
}

enum RouteMatch {
    Exact(String),
    Prefix(String),
}

impl RouteMatch {
    fn matches(&self, path: &str) -> bool {
        match self {
            RouteMatch::Exact(expected) => path == expected,
            RouteMatch::Prefix(prefix) => path.starts_with(prefix),
        }
    }
}

struct Route {
    method: Method,
    matcher: RouteMatch,
    handler: Box<dyn Fn(&Request, &mut TcpStream) -> io::Result<()> + Send + Sync + 'static>,
}

pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn add_route<F>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(&Request, &mut TcpStream) -> io::Result<()> + Send + Sync + 'static,
    {
        self.routes.push(Route { method, matcher: RouteMatch::Exact(path.to_string()), handler: Box::new(handler) });
    }

    pub fn add_prefix_route<F>(&mut self, method: Method, prefix: &str, handler: F)
    where
        F: Fn(&Request, &mut TcpStream) -> io::Result<()> + Send + Sync + 'static,
    {
        self.routes.push(Route { method, matcher: RouteMatch::Prefix(prefix.to_string()), handler: Box::new(handler) });
    }

    pub fn handle(&self, req: Request, stream: &mut TcpStream) -> io::Result<()> {
        if req.method == "OPTIONS" {
            return write_response(stream, 204, "No Content", "text/plain", b"");
        }

        let method = match Method::from_str(&req.method) {
            Some(method) => method,
            None => {
                return write_response(stream, 405, "Method Not Allowed", "application/json", b"{\"error\":\"method not allowed\"}");
            }
        };

        for route in &self.routes {
            if route.method == method && route.matcher.matches(&req.path) {
                return (route.handler)(&req, stream);
            }
        }

        write_response(stream, 404, "Not Found", "application/json", b"{\"error\":\"not found\"}")
    }
}
