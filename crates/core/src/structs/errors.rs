use std::fmt::{self};
use std::error::Error;

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