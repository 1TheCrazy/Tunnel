use std::time::Duration;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub used_ips: Vec<String>,
    pub password: String,
    pub self_id: String,
    pub private_key: String,
    pub public_key: String,
    pub vpn_port: String,
    pub server_host: String,
    pub update_period: Duration,
    pub blindly_trust_host: bool,
    pub host_fingerprint: String
}
