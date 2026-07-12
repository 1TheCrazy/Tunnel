use std::path::PathBuf;
use std::fs;
use serde::Serialize;
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

pub fn read_config_or_default<T>(path: &PathBuf) -> T where T: DeserializeOwned + Default {
    let save_content = match fs::read_to_string(path) {
        Err(_) => return T::default(),
        Ok(content) => content
    };

    match serde_json::from_str(&save_content) {
        Err(_) => return T::default(),
        Ok(deserialized) => return deserialized
    };
}

pub fn read_config<T>(path: &PathBuf) -> Result<T, ConfigError> where T: DeserializeOwned{
    let save_content: String = fs::read_to_string(path)?;
    let config: T = serde_json::from_str(&save_content)?;

    Ok(config)
}

pub fn write_config<T>(config: &T, path: &PathBuf) -> Result<(), ConfigError> where T : Serialize {
    ensure_parent_dir(path)?;

    let config_string = serde_json::to_string(config)?;
    fs::write(path, config_string)?;

    Ok(())
}

pub fn ensure_parent_dir(path: &PathBuf) -> Result<(), std::io::Error>{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}