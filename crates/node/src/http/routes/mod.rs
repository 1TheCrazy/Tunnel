mod discover;

use axum::Router;
use crate::http::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/discover", discover::router())
}