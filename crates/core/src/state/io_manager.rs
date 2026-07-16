use std::path::PathBuf;
use std::fs;
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::structs::save::SaveError;

pub const SERVER_CONFIG_PATH: fn() -> PathBuf = || config_path().join("server.toml");
pub const NODE_CONFIG_PATH: fn() -> PathBuf = || config_path().join("node.toml");
pub const CLIENT_CONFIG_PATH: fn() -> PathBuf = || config_path().join("client.toml");

pub const SERVER_SAVE_PATH: fn() -> PathBuf = || save_path().join("server.save");
pub const NODE_SAVE_PATH: fn() -> PathBuf = || save_path().join("node.save");
pub const CLIENT_SAVE_PATH: fn() -> PathBuf = || save_path().join("client.save");

pub fn config_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().expect("Could not find config directory");

    config_dir.push("1thecrazy");
    config_dir.push("tunnel");

    return config_dir;
}

pub fn save_path() -> PathBuf {
    let mut config_dir = dirs::config_dir().expect("Could not find config directory");

    config_dir.push("1thecrazy");
    config_dir.push("tunnel");
    config_dir.push("save");

    return config_dir;
}

pub fn read_config_or_default<T>(path: &PathBuf) -> T where T: DeserializeOwned + Default {
    let config_content = match fs::read_to_string(path) {
        Err(_) => return T::default(),
        Ok(content) => content
    };

    match toml::from_str(&config_content) {
        Err(_) => return T::default(),
        Ok(deserialized) => return deserialized
    };
}

pub fn read_save_or_default<T>(path: &PathBuf) -> T  where T: DeserializeOwned + Default {
    let save_content = match fs::read_to_string(path) {
        Err(_) => return T::default(),
        Ok(content) => content
    };

    match serde_json::from_str(&save_content) {
        Err(_) => return T::default(),
        Ok(save) => return save,
    }
}

pub fn write_save<T>(obj: &T, path: &PathBuf) -> Result<(), SaveError> where T : Serialize {
    ensure_parent_dir(path)?;

    let save_string = serde_json::to_string(obj)?;
    fs::write(path, save_string)?;

    Ok(())
}

pub fn ensure_parent_dir(path: &PathBuf) -> Result<(), std::io::Error>{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    Ok(())
}