use std::fmt;
use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::structs::server::ServerNode;
use crate::structs::client::ClientServer;

#[derive(Debug, Deserialize, Default, Serialize)]
pub struct ClientConfig {
    pub servers: Vec<ClientServer>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeConfig {
    pub used_ips: Vec<String>,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ServerConfig {
    pub password: String,
    pub nodes: Vec<ServerNode>,
}

impl Default for ServerConfig{
    fn default() -> Self {
        Self { 
            password: "".to_string(), 
            nodes: vec![]
        }
    }
}

#[derive(Debug)]
pub enum ConfigError {
    IO(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IO(err) => write!(f, "encountered IO error: {}", err),
            ConfigError::Json(err) => write!(f, "encountered serde error: {}", err),
        }
    }
}

impl Error for ConfigError{}