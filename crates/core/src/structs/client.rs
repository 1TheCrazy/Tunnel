use serde::{Deserialize, Serialize};

use crate::wireguard::common::gen_key_pair;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientServer {
    pub host: String,
    pub name: String,
    pub password: String,
    #[serde(default)]
    pub host_fingerprint: String,
    pub nodes: Vec<ClientNode>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientNode {
    #[serde(default)]
    pub name: String,
    pub ip: String,
    pub port: String,
    pub public_key: String,
    pub id: String,
    pub discovered: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientSave {
    pub active_server_index: i32,
    pub servers: Vec<ClientServer>,
    pub public_key: String,
    pub private_key: String,
}

impl Default for ClientSave {
    fn default() -> Self {
        let keys = gen_key_pair();

        Self {
            active_server_index: -1,
            servers: vec![],
            public_key: keys.public,
            private_key: keys.private,
        }
    }
}
