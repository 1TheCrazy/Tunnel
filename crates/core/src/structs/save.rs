use serde::{Deserialize, Serialize};
use crate::{structs::server::ServerNode, wireguard::common::gen_key_pair};

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeSave {
    pub used_ips: Vec<String>,
    pub self_id: String,
    pub private_key: String,
    pub public_key: String
}

impl Default for NodeSave {
    fn default() -> Self {
        let keys = gen_key_pair();
        
        Self {
            used_ips: vec![],
            self_id: "".to_owned(),
            public_key: keys.public,
            private_key: keys.private
        }
    }
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