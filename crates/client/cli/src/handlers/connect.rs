use tunnel_core::wireguard::{client::{activate_service, create_and_activate_client_conf}, util::interface_name_from_node_id};

use crate::{http::wrapper::discover_node, util::{io_wrapper, state::get_mut_active_server}, write_line};

pub async fn connect(id: &str) -> Result<(), ()> {
    let mut state = io_wrapper::get_mut_save();

    let server = match get_mut_active_server(&mut state) {
        Some(server) => server,
        None => { 
            write_line!("There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help"); 
            return Err(()); 
        }
    };

    let Some(node) = server.nodes.iter().find(|n| n.id == id) else {
        write_line!("The selected server didn't contian a node with the given id");
        return Err(());
    };

    if !node.discovered {
        return connect_fresh(id).await;
    }
    else {
        return connect_known(id);
    }
}

fn connect_known(id: &str) -> Result<(), ()> {
    let service_name = interface_name_from_node_id(id);

    match activate_service(&service_name) {
        Ok(()) => return Ok(()),
        Err(()) => {
            // This could be due to another service running and WireGuard refusing to start this one, or the .conf doesn't exist, or any other reason
            write_line!("Wasn't able to activate service.");
            return Err(())
        } 
    }
}

async fn connect_fresh(id: &str) -> Result<(), ()>{
    let mut state = io_wrapper::get_mut_save();
    let ref_state = io_wrapper::get_ref_save();

    let server = match get_mut_active_server(&mut state) {
        Some(server) => server,
        None => { 
            write_line!("There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help"); 
            return Err(()); 
        }
    };

    if !server.nodes.iter().any(|n| n.id == id) {
        write_line!("The selected server didn't contian anode with the given id");
        return Err(());
    }
    
    let res = match discover_node(id, server, &ref_state.public_key.clone()).await {
        Ok(res) => res,
        Err(err) => {
            write_line!("Wasn't able to discover node: \n{}", err);
            return Err(());
        }
    };
    
    let node = match server.nodes.iter_mut().find(|n| n.id == id) {
        Some(value) => value,
        // Shouldn't happen
        None => return Err(())
    };
    node.discovered = true;
    
    let endpoint = format!("{}:{}", node.ip, node.port);

    match create_and_activate_client_conf(id, &ref_state.private_key.clone(), &res.assigned_vpn_ip, &node.public_key, &endpoint) {
        Err(()) => {
            write_line!("Failed to create or start wireguard service");
            return Err(())
        },
        Ok(()) => {
            write_line!("Connected to node '{}'", id);
            return Ok(())
        }
    }
}