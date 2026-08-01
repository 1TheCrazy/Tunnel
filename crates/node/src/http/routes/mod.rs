mod discover;

use crate::http::state::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new().nest("/discover", discover::router())
}
