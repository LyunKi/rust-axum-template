use axum::{routing::get, Router};

pub fn init() -> Router {
    Router::new().route("/health-check", get(|| async { "Hello, World!" }))
}

