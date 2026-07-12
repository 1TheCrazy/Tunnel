use crate::http::state::AppState;
use crate::util::authorization;
use tunnel_core::structs::http::CreateNodeRequest;

use axum::{
    Router, extract::{Json, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/register", post(register))
}

async fn list(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers) {
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }

    let nodes = &state.server.read().unwrap().nodes;
    
    match serde_json::to_string(nodes) {
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response(),
        Ok(res) => return (StatusCode::OK, res).into_response()
    }
}

async fn register(State(state): State<AppState>, Json(body): Json<CreateNodeRequest>) -> impl IntoResponse {
    (StatusCode::OK, "OK")
}