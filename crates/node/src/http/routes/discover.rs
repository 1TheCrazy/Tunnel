use crate::http::state::AppState;
use tunnel_core::{structs::http::{CreateClientOnNodeRequest, CreateClientOnNodeRespone}, util::authorization, wireguard::node::register_client};
use axum::{
    Router, extract::{Json, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{post}
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

        if !authorization::is_request_authorized(&password, &headers) {
            return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
        }
    } // Kill RWLock
    
    match register_client(&body.public_client_key) {
        Some(assigned_ip) => return 
            (
                StatusCode::OK,
                Json(
                    CreateClientOnNodeRespone {
                        success: true,
                        vpn_network_ip: assigned_ip
                    }
                )
            ).into_response(),    
        None => return (StatusCode::INTERNAL_SERVER_ERROR, "Registering client was refused").into_response()
    }
}