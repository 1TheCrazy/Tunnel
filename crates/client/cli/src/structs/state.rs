use std::{collections::HashMap, ops::{Deref, DerefMut}};

use serde::{Deserialize, Serialize};
use tunnel_core::{
    structs::client::{ClientNode, ClientSave},
};

use crate::{
    http::wrapper::get_nodes,
    util::{io_wrapper, state::get_mut_active_server},
    write_line,
};

#[derive(Debug, Default, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct CliClientSave(pub  ClientSave);

impl Deref for CliClientSave {
    type Target = ClientSave;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CliClientSave {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CliClientSave {
    pub async fn refresh() -> Result<(), ()> {
        let mut state = io_wrapper::get_mut_save();

        let server = match get_mut_active_server(&mut state) {
            Some(server) => server,
            None => {
                write_line!(
                    "There was no selected server. To refresh nodes, select a server 'server set <NAME>' or see '--help'"
                );
                return Err(());
            }
        };

        let nodes = match get_nodes(server).await {
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
            .map(|node| match existing_nodes.remove(&node.id) {
                Some(existing_node) => ClientNode {
                    ip: node.ip,
                    id: existing_node.id,
                    port: node.port,
                    public_key: existing_node.public_key,
                    discovered: existing_node.discovered,
                },
                None => node,
            })
            .collect();

        Ok(())
    }
}
