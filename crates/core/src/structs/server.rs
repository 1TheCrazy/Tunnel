use crate::util::rand::get_128_bit_random;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerNode {
    #[serde(default)]
    pub name: String,
    pub ip: String,
    pub port: String,
    pub public_key: String,
    pub id: String,
}

impl Default for ServerNode {
    fn default() -> Self {
        Self {
            name: String::new(),
            ip: String::new(),
            port: "51820".into(),
            public_key: String::new(),
            id: get_128_bit_random(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Server {
    pub nodes: Vec<ServerNode>,
    pub password: String,
}
