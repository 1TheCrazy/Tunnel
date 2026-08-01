use crate::http::state::AppState;
use axum::{Router, routing::get};

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(health))
}

async fn health() -> &'static str {
    println!("server: request GET /health -> 200 OK");
    "OK"
}
