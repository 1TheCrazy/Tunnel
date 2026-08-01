use tunnel_core::structs::client::ClientServer;

use crate::structs::state::CliClientSave;

pub fn get_active_server(save: &CliClientSave) -> Option<&ClientServer> {
    if save.active_server_index == -1 {
        return None;
    }

    let server: &ClientServer = &save.servers[save.active_server_index as usize];

    Some(server)
}

pub fn get_mut_active_server(save: &mut CliClientSave) -> Option<&mut ClientServer> {
    if save.active_server_index == -1 {
        return None;
    }

    let index = save.active_server_index as usize;
    let server: &mut ClientServer = &mut save.servers[index];

    Some(server)
}
