use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateNodeRequest {
    pub port: String,
    pub public_key: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateNodeResponse {
    pub succes: bool,
    pub assigned_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct DiscoverNodeRequest {
    pub id: String,
    pub public_client_key: String
}

#[derive(Serialize, Deserialize)]
pub struct DiscoverNodeResponse {
    pub success: bool,
    pub assigned_vpn_ip: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateClientOnNodeRequest {
    pub public_client_key: String
}

#[derive(Serialize, Deserialize)]
pub struct CreateClientOnNodeRespone {
    pub success: bool,
    pub vpn_network_ip: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateNodeRequest{
    pub id: String,
}