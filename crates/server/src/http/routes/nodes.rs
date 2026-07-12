use crate::http::state::AppState;
use crate::util::authorization;

use axum::{
    Router, extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::get
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(nodes))
}

async fn nodes<'a>(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorization::is_request_authorized(&state.password, &headers) {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }

    let nodes = state.nodes.read().unwrap();
    
    match serde_json::to_string(&*nodes) {
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response(),
        Ok(res) => return (StatusCode::OK, res).into_response()
    }
}