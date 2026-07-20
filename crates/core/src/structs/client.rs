use serde::{Deserialize, Serialize};

use crate::structs::server::ServerNode;

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientServer {
    pub host: String,
    pub name: String,
    pub password: String,
    pub nodes: Vec<ServerNode>
}