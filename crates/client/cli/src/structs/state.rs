use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tunnel_core::{structs::client::{ClientNode, ClientServer}, wireguard::common::gen_key_pair};

use crate::{http::wrapper::get_nodes, util::{io_wrapper, state::get_mut_active_server}, write_line};

#[derive(Serialize, Deserialize)]
pub struct CliClientSave {
    pub active_server_index: i32,
    pub servers: Vec<ClientServer>,
    pub public_key: String,
    pub private_key: String
}

impl Default for CliClientSave {
    fn default() -> Self {
        let keys = gen_key_pair();
        
        Self {
            active_server_index: -1,
            servers: vec![],
            public_key: keys.public,
            private_key: keys.private
        }
    }
}

impl CliClientSave {
    pub async fn refresh() -> Result<(), ()>{
        let mut state = io_wrapper::get_mut_save();
    
        let server= match get_mut_active_server(&mut state) {
            Some(server) => server,
            None => { 
                write_line!("There was no selected server. To refresh nodes, select a server 'server set <NAME>' or see '--help'");
                return Err(());
            }
        };

        let nodes = match get_nodes(&server.host, &server.password).await {
            Ok(nodes) => nodes,
            Err(err) => {
                write_line!("Encountered an error while refreshing nodes: \n{}", err);
                return Err(());
            }
        };

        let mut existing_nodes: HashMap<_, _> = std::mem::take(&mut server.nodes)
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();

        server.nodes = nodes
            .into_iter()
            .map(|node| {
                match existing_nodes.remove(&node.id) {
                    Some(existing_node) => ClientNode {
                        ip: node.ip,
                        id: existing_node.id,
                        port: node.port,
                        public_key: existing_node.public_key,
                        discovered: existing_node.discovered,
                    },
                    None => node,
                }
            })
            .collect();

        Ok(())
    }
}