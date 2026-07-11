#[derive(Debug, Clone)]
pub struct ServerNode {
    pub ip: String,
    pub port: String, 
    pub public_key: String,
}

impl Default for ServerNode {
    fn default() -> Self {
        Self {
            ip: String::new(),
            port: "51820".into(),
            public_key: String::new()
        }
    }
}

#[derive(Debug, Default)]
pub struct Server {
    pub nodes: Vec<ServerNode>,
    pub port: String
}