use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientServer {
    pub host: String,
    pub name: String,
    pub password: String,
    pub nodes: Vec<ClientNode>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClientNode {
    pub ip: String,
    pub port: String,
    pub public_key: String,
    pub id: String,
    pub discovered: bool
}