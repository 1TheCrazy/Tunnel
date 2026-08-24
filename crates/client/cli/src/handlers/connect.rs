use tunnel_core::wireguard::client::{
    create_and_activate_client_conf, update_and_activate_client_conf,
};

use crate::{
    http::wrapper::discover_node,
    structs::state::CliClientSave,
    util::{io_wrapper, state::get_mut_active_server},
    write_line,
};

pub async fn connect(id: &str) -> Result<(), ()> {
    // A node can change networks while the client is offline. Always obtain its
    // current endpoint before either creating or reactivating its tunnel.
    CliClientSave::refresh().await?;

    let mut state = io_wrapper::get_mut_save();

    let server = match get_mut_active_server(&mut state) {
        Some(server) => server,
        None => {
            write_line!(
                "There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help"
            );
            return Err(());
        }
    };

    let Some(node) = server.nodes.iter().find(|n| n.id == id) else {
        write_line!("The selected server didn't contian a node with the given id");
        return Err(());
    };

    let node = node.clone();
    let discovered = node.discovered;
    drop(state);

    if !discovered {
        return connect_fresh(id).await;
    } else {
        return connect_known(id, &node);
    }
}

fn connect_known(id: &str, node: &tunnel_core::structs::client::ClientNode) -> Result<(), ()> {
    let endpoint = format!("{}:{}", node.ip, node.port);

    match update_and_activate_client_conf(id, &node.public_key, &endpoint) {
        Ok(()) => return Ok(()),
        Err(()) => {
            // This could be due to another service running and WireGuard refusing to start this one, or the .conf doesn't exist, or any other reason
            write_line!("Wasn't able to activate service.");
            return Err(());
        }
    }
}

async fn connect_fresh(id: &str) -> Result<(), ()> {
    let mut state = io_wrapper::get_mut_save();
    let ref_state = io_wrapper::get_ref_save();

    let server = match get_mut_active_server(&mut state) {
        Some(server) => server,
        None => {
            write_line!(
                "There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help"
            );
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
        None => return Err(()),
    };
    node.discovered = true;

    let endpoint = format!("{}:{}", node.ip, node.port);

    match create_and_activate_client_conf(
        id,
        &ref_state.private_key.clone(),
        &res.assigned_vpn_ip,
        &node.public_key,
        &endpoint,
    ) {
        Err(()) => {
            write_line!("Failed to create or start wireguard service");
            return Err(());
        }
        Ok(()) => {
            write_line!("Connected to node '{}'", id);
            return Ok(());
        }
    }
}
