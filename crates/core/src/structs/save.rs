use serde::{Deserialize, Serialize};
use crate::structs::server::ServerNode;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeSave {
    pub used_ips: Vec<String>,
    pub self_id: String,
    pub private_key: String
}

#[derive(Deserialize, Serialize)]
pub struct ServerSave {
    pub nodes: Vec<ServerNode>,
}

impl Default for ServerSave {
    fn default() -> Self {
        Self { 
            nodes: vec![]
        }
    }
}