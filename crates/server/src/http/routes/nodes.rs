use std::net::SocketAddr;

use crate::http::state::AppState;
use tunnel_core::{constants::TUNNEL_SERVICE_PORT, structs::{http::{CreateClientOnNodeRequest, CreateClientOnNodeRespone, CreateNodeRequest, CreateNodeResponse, DiscoverNodeRequest, DiscoverNodeResponse, UpdateNodeRequest}, server::ServerNode}, util::{authorization, rand::get_128_bit_random}};
use axum::{
    Router, extract::{ConnectInfo, Json, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, routing::{get, post}
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/register", post(register))
        .route("/discover", post(discover))
        .route("/update", post(update))
}

async fn list(State(state): State<AppState>, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
    println!("server: request GET /nodes/list from {}", addr);

    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers) {
        println!("server: request GET /nodes/list from {} -> 401 unauthorized", addr);
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
    }

    let nodes = &state.server.read().unwrap().nodes;
    
    match serde_json::to_string(nodes) {
        Err(err) => {
            println!(
                "server: request GET /nodes/list from {} -> 500 serialization_error={}",
                addr,
                err
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response()
        },
        Ok(res) => {
            println!(
                "server: request GET /nodes/list from {} -> 200 nodes={}",
                addr,
                nodes.len()
            );
            return (StatusCode::OK, res).into_response()
        }
    }
}

async fn register(State(state): State<AppState>, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(body): Json<CreateNodeRequest>) -> impl IntoResponse {
    println!(
        "server: request POST /nodes/register from {} vpn_port={}",
        addr,
        body.port
    );

    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers){
        println!("server: request POST /nodes/register from {} -> 401 unauthorized", addr);
        return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
    }

    let assigned_id = get_128_bit_random();

    let node = ServerNode {
        ip: addr.ip().to_string(),
        port: body.port,
        public_key: body.public_key,
        id: assigned_id.to_owned()
    };

    state.server.write().unwrap().nodes.push(node);

    println!(
        "server: request POST /nodes/register from {} -> 200 assigned_id={}",
        addr,
        assigned_id
    );

    (
        StatusCode::OK, 
        Json(CreateNodeResponse {
            succes: true,
            assigned_id: assigned_id.to_owned()
        })
    ).into_response()
}

#[axum::debug_handler]
async fn discover(State(state): State<AppState>, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(body): Json<DiscoverNodeRequest>) -> impl IntoResponse {
    println!(
        "server: request POST /nodes/discover from {} node_id={}",
        addr,
        body.id
    );

    let (password, target_ip) = {
        let server = state.server.read().unwrap();
        let password = server.password.clone();

        if !authorization::is_request_authorized(&password, &headers){
            println!("server: request POST /nodes/discover from {} -> 401 unauthorized", addr);
            return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
        }

        let target_ip = match server.nodes.iter().find(|node| node.id == body.id){
            Some(node) => node.ip.clone(),
            None => {
                println!(
                    "server: request POST /nodes/discover from {} -> 400 node_id_not_found={}",
                    addr,
                    body.id
                );
                return (StatusCode::BAD_REQUEST, "Node id not found").into_response()
            }
        };

        (password, target_ip)
    };

    let client = state.http_client;
    let target_url = format!("http://{}:{}/discover", target_ip, TUNNEL_SERVICE_PORT);

    println!(
        "server: forwarding discover request node_id={} target_url={}",
        body.id,
        target_url
    );

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
        Err(err) => {
            println!(
                "server: request POST /nodes/discover from {} -> 500 forward_error={}",
                addr,
                err
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response()
        }
    };


    if creation_req_res.status() != StatusCode::OK {
        println!(
            "server: request POST /nodes/discover from {} -> 500 node_status={}",
            addr,
            creation_req_res.status()
        );
        return (StatusCode::INTERNAL_SERVER_ERROR, "Node refused to discover").into_response()
    }

    let res_json: CreateClientOnNodeRespone = match creation_req_res.json().await {
        Ok(body) => body,
        Err(err) => {
            println!(
                "server: request POST /nodes/discover from {} -> 500 node_response_decode_error={}",
                addr,
                err
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response()
        }
    };

    println!(
        "server: request POST /nodes/discover from {} -> 200 node_id={} assigned_vpn_ip={}",
        addr,
        body.id,
        res_json.vpn_network_ip
    );

    (
        StatusCode::OK, 
        Json(DiscoverNodeResponse {
            success: true,
            assigned_vpn_ip: res_json.vpn_network_ip.to_owned()
        })
    ).into_response()
}

async fn update(State(state): State<AppState>, headers: HeaderMap, ConnectInfo(addr): ConnectInfo<SocketAddr>, Json(body) : Json<UpdateNodeRequest>) -> impl IntoResponse {
    println!(
        "server: request POST /nodes/update from {} node_id={} new_ip={}",
        addr,
        body.id,
        addr.ip().to_string(),
    );

    let mut server = state.server.write().unwrap();

    if !authorization::is_request_authorized(&server.password, &headers){
        println!("server: request POST /nodes/update from {} -> 401 unauthorized", addr);
        return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
    }

    if let Some(node) = server.nodes.iter_mut().find(|node| node.id == body.id) {
        node.ip = addr.ip().to_string();
    }
    else{
        println!(
            "server: request POST /nodes/update from {} -> 400 node_id_not_found={}",
            addr,
            body.id
        );
        return (StatusCode::BAD_REQUEST, "Node id not valid").into_response()
    }

    println!(
        "server: request POST /nodes/update from {} -> 200 node_id={}",
        addr,
        body.id
    );

    return (StatusCode::OK, "OK").into_response();
}
