use std::sync::{Arc, RwLock};

use tunnel_core::structs::node::Node;

pub type SharedServer = Arc<RwLock<Node>>;

#[derive(Clone)]
pub struct AppState {
    pub server: SharedServer,
}
