use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientServer{
    pub adress: String,
    pub name: String,
    pub password: String
}