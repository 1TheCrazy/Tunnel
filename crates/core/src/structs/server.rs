use serde::{Deserialize, Serialize};
use crate::util::rand::get_128_bit_random;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerNode {
    pub ip: String,
    pub port: String, 
    pub public_key: String,
    pub id: String,
}

impl Default for ServerNode {
    fn default() -> Self {
        Self {
            ip: String::new(),
            port: "51820".into(),
            public_key: String::new(),
            id: get_128_bit_random()
        }
    }
}

#[derive(Debug, Default)]
pub struct Server {
    pub nodes: Vec<ServerNode>,
    pub port: String,
    pub password: String
}