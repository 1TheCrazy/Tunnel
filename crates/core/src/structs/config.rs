use std::fmt::{self};
use std::error::Error;
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

#[derive(Debug)]
pub enum ConfigError {
    IO(std::io::Error),
    TomlDesirialization(toml::de::Error),
    TomlSerialization(toml::ser::Error)
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlSerialization(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDesirialization(error)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IO(err) => write!(f, "encountered IO error: {}", err),
            ConfigError::TomlSerialization(err) => write!(f, "encountered toml serialization error: {}", err),
            ConfigError::TomlDesirialization(err) => write!(f, "encountered toml deserialization error: {}", err),
        }
    }
}

impl Error for ConfigError{}