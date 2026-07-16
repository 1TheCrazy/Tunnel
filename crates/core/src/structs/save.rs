use std::fmt;
use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::structs::server::ServerNode;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeSave {
    pub used_ips: Vec<String>,
    pub self_id: String,
    pub private_key: String
}

#[derive(Deserialize, Serialize)]
pub struct ServerSave {
    pub nodes: Vec<ServerNode>,
}

impl Default for ServerSave{
    fn default() -> Self {
        Self { 
            nodes: vec![]
        }
    }
}

#[derive(Debug)]
pub enum SaveError {
    IO(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::IO(err) => write!(f, "encountered IO error: {}", err),
            SaveError::Json(err) => write!(f, "encountered serde error: {}", err),
        }
    }
}

impl Error for SaveError{}