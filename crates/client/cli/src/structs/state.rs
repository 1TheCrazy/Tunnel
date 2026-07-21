use serde::{Deserialize, Serialize};
use tunnel_core::structs::client::ClientServer;

use crate::{http::wrapper::get_nodes, util::{io_wrapper, state::get_mut_active_server}, write_line};

#[derive(Serialize, Deserialize)]
pub struct CliClientSave {
    pub active_server_index: i32,
    pub servers: Vec<ClientServer>
}

impl Default for CliClientSave {
    fn default() -> Self {
        Self {
            active_server_index: -1,
            servers: vec![]
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

        server.nodes = nodes;

        Ok(())
    }
}