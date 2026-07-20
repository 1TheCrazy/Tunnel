use serde::{Deserialize, Serialize};
use tunnel_core::structs::client::ClientServer;

#[derive(Serialize, Deserialize)]
pub struct CliClientSave {
    pub active_server_index: i32,
    pub servers: Vec<ClientServer>
}

impl Default for CliClientSave {
    fn default() -> Self {
        Self {
            active_server_index: -1,
            servers: vec![]
        }
    }
}