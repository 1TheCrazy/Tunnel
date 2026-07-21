use crate::{util::{io_wrapper, out::print_node_table}, write_line};

pub fn list_nodes() -> Result<(), ()> {
    let state= io_wrapper::get_ref_save();
    
    if state.active_server_index == -1 {
        write_line!("There was no selected server. To list nodes select a server using 'server set <NAME>' or see '--help'");

        return Err(());
    }

    let server = &state.servers[state.active_server_index as usize];
    let nodes = &server.nodes;

    print_node_table(nodes);
    
    Ok(())
}