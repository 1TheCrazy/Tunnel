use crate::structs::{server, server::ServerNode};

impl server::Server {
    pub fn get_node_ips(&self) -> Vec<String> {
        self.nodes.iter().map(|node| node.ip.clone()).collect()
    }

    pub fn get_nodes(&self) -> &[ServerNode] {
        &self.nodes
    }
}