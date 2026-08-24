use std::net::SocketAddr;

use crate::net::state::AppState;
use axum::{
    Router,
    extract::{ConnectInfo, Json, State, WebSocketUpgrade, ws::{Message, WebSocket}},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use futures_util::{SinkExt, StreamExt};
use tokio::{sync::mpsc, time::{timeout, Duration}};
use tunnel_core::{
    structs::{
        http::{
            CreateClientOnNodeRequest, CreateNodeRequest,
            CreateNodeResponse, DiscoverNodeRequest, DiscoverNodeResponse, NodeToServerMessage,
            ServerToNodeMessage,
        },
        server::ServerNode,
    },
    util::{authorization, rand::get_128_bit_random},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/list", get(list))
        .route("/register", post(register))
        .route("/discover", post(discover))
        .route("/websocket", get(websocket))
}

async fn list(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("server: request GET /nodes/list from {}", addr);

    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers) {
        println!(
            "server: request GET /nodes/list from {} -> 401 unauthorized",
            addr
        );
        return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response();
    }

    let nodes = &state.server.read().unwrap().nodes;

    match serde_json::to_string(nodes) {
        Err(err) => {
            println!(
                "server: request GET /nodes/list from {} -> 500 serialization_error={}",
                addr, err
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response();
        }
        Ok(res) => {
            println!(
                "server: request GET /nodes/list from {} -> 200 nodes={}",
                addr,
                nodes.len()
            );
            return (StatusCode::OK, res).into_response();
        }
    }
}

async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<CreateNodeRequest>,
) -> impl IntoResponse {
    println!(
        "server: request POST /nodes/register from {} vpn_port={}",
        addr, body.port
    );

    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers) {
        println!(
            "server: request POST /nodes/register from {} -> 401 unauthorized",
            addr
        );
        return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
    }

    let assigned_id = get_128_bit_random();

    let node = ServerNode {
        name: body.name,
        ip: addr.ip().to_string(),
        port: body.port,
        public_key: body.public_key,
        id: assigned_id.to_owned(),
    };

    state.server.write().unwrap().nodes.push(node);

    println!(
        "server: request POST /nodes/register from {} -> 200 assigned_id={}",
        addr, assigned_id
    );

    (
        StatusCode::OK,
        Json(CreateNodeResponse {
            succes: true,
            assigned_id: assigned_id.to_owned(),
        }),
    )
        .into_response()
}

#[axum::debug_handler]
async fn discover(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(body): Json<DiscoverNodeRequest>,
) -> impl IntoResponse {
    println!(
        "server: request POST /nodes/discover from {} node_id={}",
        addr, body.id
    );

    let node_id = {
        let server = state.server.read().unwrap();

        if !authorization::is_request_authorized(&server.password, &headers) {
            println!(
                "server: request POST /nodes/discover from {} -> 401 unauthorized",
                addr
            );
            return (StatusCode::UNAUTHORIZED, "Invalid Credentials").into_response();
        }

        match server.nodes.iter().find(|node| node.id == body.id) {
            Some(node) => node.id.clone(),
            None => {
                println!(
                    "server: request POST /nodes/discover from {} -> 400 node_id_not_found={}",
                    addr, body.id
                );
                return (StatusCode::BAD_REQUEST, "Node id not found").into_response();
            }
        }
    };

    let sender = match state.node_connections.read().unwrap().get(&node_id).cloned() {
        Some(sender) => sender,
        None => return (StatusCode::SERVICE_UNAVAILABLE, "Node is not connected").into_response(),
    };

    let request_id = get_128_bit_random();
    let (response_sender, response_receiver) = tokio::sync::oneshot::channel();
    state.pending_discoveries.write().unwrap().insert(request_id.clone(), response_sender);

    let message = ServerToNodeMessage::DiscoverRequest {
        request_id: request_id.clone(),
        request: CreateClientOnNodeRequest { public_client_key: body.public_client_key },
    };
    let text = match serde_json::to_string(&message) {
        Ok(text) => text,
        Err(error) => {
            state.pending_discoveries.write().unwrap().remove(&request_id);
            println!("server: failed to serialize discover request error={error}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Internal Error").into_response();
        }
    };

    if sender.send(Message::Text(text.into())).await.is_err() {
        state.pending_discoveries.write().unwrap().remove(&request_id);
        return (StatusCode::SERVICE_UNAVAILABLE, "Node is not connected").into_response();
    }

    let response = match timeout(Duration::from_secs(15), response_receiver).await {
        Ok(Ok(response)) => response,
        Ok(Err(_)) => return (StatusCode::BAD_GATEWAY, "Node connection closed").into_response(),
        Err(_) => {
            state.pending_discoveries.write().unwrap().remove(&request_id);
            return (StatusCode::GATEWAY_TIMEOUT, "Node response timed out").into_response();
        }
    };

    if !response.success {
        return (StatusCode::CONFLICT, "Node refused client registration").into_response();
    }

    println!(
        "server: request POST /nodes/discover from {} -> 200 node_id={} assigned_vpn_ip={}",
        addr, body.id, response.vpn_network_ip
    );

    (
        StatusCode::OK,
        Json(DiscoverNodeResponse {
            success: true,
            assigned_vpn_ip: response.vpn_network_ip,
        }),
    )
        .into_response()
}

async fn websocket(
    State(state): State<AppState>,
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    websocket: WebSocketUpgrade,
) -> impl IntoResponse {
    if !authorization::is_request_authorized(&state.server.read().unwrap().password, &headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    websocket.on_upgrade(move |socket| handle_websocket(socket, state, addr))
}

async fn handle_websocket(socket: WebSocket, state: AppState, addr: SocketAddr) {
    let (mut sink, mut stream) = socket.split();
    let (sender, mut receiver) = mpsc::channel(16);
    let writer = tokio::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if sink.send(message).await.is_err() { break; }
        }
    });

    let node_id = match stream.next().await {
        Some(Ok(Message::Text(text))) => match serde_json::from_str::<NodeToServerMessage>(&text) {
            Ok(NodeToServerMessage::Connected { node_id }) => node_id,
            _ => return,
        },
        _ => return,
    };

    if !state.server.read().unwrap().nodes.iter().any(|node| node.id == node_id) {
        return;
    }
    state.node_connections.write().unwrap().insert(node_id.clone(), sender.clone());
    println!("server: node websocket connected node_id={} addr={}", node_id, addr);

    while let Some(result) = stream.next().await {
        match result {
            Ok(Message::Text(text)) => match serde_json::from_str::<NodeToServerMessage>(&text) {
                Ok(NodeToServerMessage::Update(update)) if update.id == node_id => {
                    if let Some(node) = state.server.write().unwrap().nodes.iter_mut().find(|node| node.id == node_id) {
                        node.ip = addr.ip().to_string();
                        node.name = update.name;
                    }
                }
                Ok(NodeToServerMessage::DiscoverResponse { request_id, response }) => {
                    if let Some(waiter) = state.pending_discoveries.write().unwrap().remove(&request_id) {
                        let _ = waiter.send(response);
                    }
                }
                Ok(_) => println!("server: rejected websocket message from node_id={}", node_id),
                Err(error) => println!("server: invalid node websocket payload error={error}"),
            },
            Ok(Message::Ping(data)) => {
                if sender.send(Message::Pong(data)).await.is_err() { break; }
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }

    {
        let mut connections = state.node_connections.write().unwrap();
        if connections
            .get(&node_id)
            .is_some_and(|current| current.same_channel(&sender))
        {
            connections.remove(&node_id);
        }
    }
    drop(sender);
    let _ = writer.await;
    println!("server: node websocket disconnected node_id={}", node_id);
}
