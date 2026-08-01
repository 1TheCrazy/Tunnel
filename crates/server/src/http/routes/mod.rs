mod health;
mod nodes;

use crate::http::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/health", health::router())
        .nest("/nodes", nodes::router())
}
