use std::sync::{Arc, RwLock};
use tunnel_core::structs::server::Server;

pub type SharedServer = Arc<RwLock<Server>>;

#[derive(Clone)]
pub struct AppState {
    pub server: SharedServer
}