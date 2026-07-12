use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateNodeRequest {
    ip: String,
    port: String,
    public_key: String
}