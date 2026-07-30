use std::net::SocketAddr;

use crate::http::state::AppState;
use tunnel_core::{structs::http::{CreateClientOnNodeRequest, CreateClientOnNodeRespone}, util::authorization, wireguard::node::register_client};
use axum::{
    Router, extract::{ConnectInfo, Json, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{post}
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(discover))
}

#[axum::debug_handler]
async fn discover(State(state): State<AppState>, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(body): Json<CreateClientOnNodeRequest>) -> impl IntoResponse {
    println!(
        "node: request POST /discover from {} public_client_key_len={}",
        addr,
        body.public_client_key.len()
    );

    let mut server = state.server.write().unwrap();
    let password = server.password.clone();

    if !authorization::is_request_authorized(&password, &headers) {
        println!("node: request POST /discover from {} -> 401 unauthorized", addr);
        return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
    }

    match register_client(&body.public_client_key, &mut server) {
        Some(assigned_ip) => {
            println!(
                "node: request POST /discover from {} -> 200 assigned_vpn_ip={}",
                addr,
                assigned_ip
            );
            return (
                StatusCode::OK,
                Json(
                    CreateClientOnNodeRespone {
                        success: true,
                        vpn_network_ip: assigned_ip
                    }
                )
            ).into_response()
        },
        None => {
            println!("node: request POST /discover from {} -> 500 client_registration_refused", addr);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Registering client was refused").into_response()
        }
    }
}
