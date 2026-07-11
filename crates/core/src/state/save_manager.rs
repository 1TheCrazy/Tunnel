use std::path::PathBuf;
use std::fs;
use serde::de::DeserializeOwned;
use crate::structs::state::ConfigError;

pub const SERVER_CONFIG_PATH: fn() -> PathBuf = || config_path().join("server.config");
pub const NODE_CONFIG_PATH: fn() -> PathBuf = || config_path().join("node.config");
pub const CLIENT_CONFIG_PATH: fn() -> PathBuf = || config_path().join("client.config");

pub fn config_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().expect("Could not find config directory");

    config_dir.push("1thecrazy");
    config_dir.push("tunnel");

    return config_dir;
}

pub fn read_config<T>(path: PathBuf) -> Result<T, ConfigError> where T: DeserializeOwned{
    let save_content: String = fs::read_to_string(path)?;
    let config: T = serde_json::from_str(&save_content)?;

    Ok(config)
}