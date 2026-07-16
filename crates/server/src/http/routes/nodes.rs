use crate::http::state::AppState;
use tunnel_core::{constants::TUNNEL_SERVICE_PORT, structs::{http::{CreateClientOnNodeRequest, CreateClientOnNodeRespone, CreateNodeRequest, CreateNodeResponse, DiscoverNodeRequest, DiscoverNodeResponse, UpdateNodeRequest}, server::ServerNode}, util::{authorization, rand::get_128_bit_random}};
use axum::{
    Router, extract::{Json, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/register", post(register))
        .route("/discover", post(discover))
        .route("/update", post(update))
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

async fn register(State(state): State<AppState>, headers: HeaderMap, Json(body): Json<CreateNodeRequest>) -> impl IntoResponse {
    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers){
        return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
    }

    let assigned_id = get_128_bit_random();

    let node = ServerNode {
        port: body.ip,
        ip: body.port,
        public_key: body.public_key,
        id: assigned_id.to_owned()
    };

    state.server.write().unwrap().nodes.push(node);

    (
        StatusCode::OK, 
        Json(CreateNodeResponse {
            succes: true,
            assigned_id: assigned_id.to_owned()
        })
    ).into_response()
}

#[axum::debug_handler]
async fn discover(State(state): State<AppState>, headers: HeaderMap, Json(body): Json<DiscoverNodeRequest>) -> impl IntoResponse {
    
    let (password, target_ip) = {
        let server = state.server.read().unwrap();
        let password = server.password.clone();

        if !authorization::is_request_authorized(&password, &headers){
            return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
        }

        let target_ip = match server.nodes.iter().find(|node| node.id == body.id){
            Some(node) => node.ip.clone(),
            None => return (StatusCode::BAD_REQUEST, "Node id not found").into_response()
        };

        (password, target_ip)
    };

    let client = state.http_client;
    let target_url = format!("http://{}:{}", target_ip, TUNNEL_SERVICE_PORT);

    let req_body = CreateClientOnNodeRequest {
        public_client_key: body.public_client_key
    };

    let creation_req_res = match client
        .post(target_url)
        .header("Tunnel-Authorization", password)
        .json(&req_body)
        .send()
        .await 
    {
        Ok(res) => res,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response()
    };


    if creation_req_res.status() != StatusCode::OK {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Node refused to discover").into_response()
    }

    let res_json: CreateClientOnNodeRespone = match creation_req_res.json().await {
        Ok(body) => body,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response()
    };

    (
        StatusCode::OK, 
        Json(DiscoverNodeResponse {
            success: true,
            assigned_vpn_ip: res_json.vpn_network_ip.to_owned()
        })
    ).into_response()
}

async fn update(State(state): State<AppState>, headers: HeaderMap, Json(body) : Json<UpdateNodeRequest>) -> impl IntoResponse {
    let mut server = state.server.write().unwrap();

    if !authorization::is_request_authorized(&server.password, &headers){
        return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
    }

    if let Some(index) = server.nodes.iter().position(|x| x.id == body.id) {
        server.nodes[index].ip = body.ip;
    }
    else{
        return (StatusCode::BAD_REQUEST, "Node id not valid").into_response()
    }

    return (StatusCode::OK, "OK").into_response();
}