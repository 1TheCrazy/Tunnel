use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClientServer{
    pub adress: String,
    pub name: String,
    pub password: String
}