use std::sync::Arc;
use std::sync::RwLock;

use tunnel_core::structs::server::ServerNode;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<ServerNode>>>,
    pub password: String
}