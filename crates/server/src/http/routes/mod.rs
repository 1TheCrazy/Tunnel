mod health;
mod nodes;

use axum::Router;
use crate::http::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .nest("/nodes", nodes::router())
}