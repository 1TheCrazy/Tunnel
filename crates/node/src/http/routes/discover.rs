use crate::http::state::AppState;
use tunnel_core::{util::authorization, structs::http::CreateClientOnNodeRequest};
use axum::{
    Router, extract::{Json, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(discover))
}

#[axum::debug_handler]
async fn discover(State(state): State<AppState>, headers: HeaderMap, Json(body): Json<CreateClientOnNodeRequest>) -> impl IntoResponse {
    {
        let server = state.server.read().unwrap();
        let password = server.password.clone();

        if !authorization::is_request_authorized(&password, &headers){
            return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
        }
    } // Kill RWLock
    
    (StatusCode::OK, "OK").into_response()
}