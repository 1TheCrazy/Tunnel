use crate::{util::{io_wrapper, out::print_server_table}, write_line};

pub fn list_servers() -> Result<(), ()> {
    let state = io_wrapper::get_ref_save();

    let servers = state.servers;

    if servers.len() == 0 {
        write_line!("There was no connected server");
        return  Ok(());
    }

    print_server_table(&servers, state.active_server_index);
    
    Ok(())
}