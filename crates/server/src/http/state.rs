use std::sync::Arc;
use tokio::sync::RwLock;

use tunnel_core::structs::server::ServerNode;

#[derive(Clone)]
pub struct AppState {
    pub nodes: Arc<RwLock<Vec<ServerNode>>>,
}