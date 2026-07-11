use serde::Deserialize;
use crate::structs::client::ClientServer;

#[derive(Debug, Deserialize, Default)]
pub struct ClientConfig {
    pub servers: Vec<ClientServer>,
}

pub struct ServerConfig {
    pub password: String,

}

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