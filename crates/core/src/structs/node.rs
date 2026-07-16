#[derive(Clone)]
pub struct Node {
    pub used_ips: Vec<String>,
    pub password: String,
    pub self_id: String,
}