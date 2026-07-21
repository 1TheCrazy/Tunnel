use crate::{util::{io_wrapper, out::print_node_table, state::get_active_server}, write_line};

pub fn list_nodes() -> Result<(), ()> {
    let state = io_wrapper::get_ref_save();
    
    let server = match get_active_server(&state) {
        Some(server) => server,
        None => { 
            write_line!("There was no selected server. To list nodes, select a server using 'server set <NAME>' or see '--help'");
            return Err(());
        }
    };

    let nodes = &server.nodes;

    print_node_table(nodes);
    
    Ok(())
}