use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeConfig {
    pub vpn_port: String,
    pub password: String,
    pub server_host: String,
    #[serde(with = "humantime_serde")]
    pub update_period: Duration,
    pub blindly_trust_host: bool,
    pub host_fingerprint: String
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ServerConfig {
    pub password: String,
    pub self_hostname: String
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            vpn_port: "51820".to_string(),
            password: "".to_string(),
            server_host: "localhost".to_string(),
            update_period: Duration::from_mins(10),
            blindly_trust_host: true,
            host_fingerprint: "".to_string(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            password: "".to_string(),
            self_hostname: "localhost".to_string()
        }
    }
}
