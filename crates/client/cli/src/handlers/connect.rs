use crate::{http::wrapper::discover_node, util::{io_wrapper, state::get_active_server}, write_line};

pub async fn connect(id: &str) -> Result<(), ()> {
    let state = io_wrapper::get_ref_save();
    let server = match get_active_server(&state) {
        Some(server) => server,
        None => { 
            write_line!("There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help"); 
            return Err(()); 
        }
    };
    
    let res = match discover_node(id, server, &state.public_key).await {
        Ok(res) => res,
        Err(err) => {
            write_line!("Wasn't able to discover node: \n{}", err);
            return Err(());
        }
    };

    // TODO: implement client interface setting/switching
    write_line!("Assigned: {}", res.assigned_vpn_ip);

    Ok(())
}