use serde::{Deserialize, Serialize};

use crate::constants::TUNNEL_SERVICE_PORT;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub vpn_port: String,
    pub password: String,
    pub server_host: String
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    pub password: String,
}


impl Default for NodeConfig{
    fn default() -> Self {
        Self { 
            vpn_port: "51820".to_string(),
            password: "".to_string(),
            server_host: format!("localhost:{}", TUNNEL_SERVICE_PORT).to_string()
        }
    }
}

impl Default for ServerConfig{
    fn default() -> Self {
        Self { 
            password: "".to_string(), 
        }
    }
}