use std::{collections::HashMap, sync::{Arc, RwLock}};
use tokio::sync::{mpsc, oneshot};
use tunnel_core::structs::server::Server;
use tunnel_core::structs::http::CreateClientOnNodeRespone;
use axum::extract::ws::Message;

pub type SharedServer = Arc<RwLock<Server>>;

#[derive(Clone)]
pub struct AppState {
    pub server: SharedServer,
    pub node_connections: Arc<RwLock<HashMap<String, mpsc::Sender<Message>>>>,
    pub pending_discoveries: Arc<RwLock<HashMap<String, oneshot::Sender<CreateClientOnNodeRespone>>>>,
}
