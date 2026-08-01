use std::error::Error;

use tunnel_core::structs::client::ClientServer;

use crate::{
    http::wrapper::get_nodes,
    structs::{
        cli::ServerCommand::{self, *},
        error::GenericError,
    },
    util::io_wrapper::get_mut_save,
    write_line,
};

pub async fn server(command: ServerCommand) -> Result<(), ()> {
    match command {
        Add {
            name,
            host,
            password,
        } => match server_add(&name, &host, password).await {
            Ok(_) => {
                write_line!("Successfully added the server");
                return Ok(());
            }
            Err(err) => {
                write_line!("Wasn't able to add the server: \n{}", err);
                return Err(());
            }
        },
        Remove { name } => match server_remove(&name) {
            Ok(_) => {
                write_line!("Successfully removed the server '{}'", name);
                return Ok(());
            }
            Err(err) => {
                write_line!("Wasn't able to remove the server: \n{}", err);
                return Err(());
            }
        },
        Set { name } => match server_set(&name) {
            Ok(_) => {
                write_line!("'{}' is now the active server", &name);
                return Ok(());
            }
            Err(err) => {
                write_line!("Wasn't able to set the active server: \n{}", err);
                return Err(());
            }
        },
    }
}

async fn server_add(
    name: &str,
    host: &str,
    password: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let mut state = get_mut_save();

    if state.servers.iter().any(|server| server.name == name) {
        return Err(GenericError(
            "Server name already in use. To modify a server see '--help'".to_owned(),
        )
        .into());
    }

    if let Some(server) = state.servers.iter().find(|server| server.host == host) {
        return Err(GenericError(format!(
            "A server with the same host is already present ('{}')",
            server.name
        ))
        .into());
    }

    let pw = password.unwrap_or("".to_owned());

    let node_list = match get_nodes(&host, &pw).await {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let server = ClientServer {
        host: host.to_owned(),
        name: name.to_owned(),
        password: pw.to_owned(),
        nodes: node_list.to_owned(),
    };

    state.servers.push(server);

    if state.active_server_index == -1 {
        state.active_server_index = state.servers.len() as i32 - 1;
    }

    Ok(())
}

fn server_remove(name: &str) -> Result<(), Box<dyn Error>> {
    let mut state = get_mut_save();

    let target_server_index = match state.servers.iter().position(|s| s.name == name) {
        Some(index) => index,
        None => {
            return Err(
                GenericError(format!("There was no server with the name '{}'", name)).into(),
            );
        }
    };

    state.servers.remove(target_server_index);

    if target_server_index as i32 == state.active_server_index {
        state.active_server_index = -1;
    }

    if target_server_index as i32 <= state.active_server_index {
        state.active_server_index -= 1;
    }

    Ok(())
}

fn server_set(name: &str) -> Result<(), Box<dyn Error>> {
    let mut state = get_mut_save();

    let target_server_index = match state.servers.iter().position(|s| s.name == name) {
        Some(index) => index,
        None => {
            return Err(
                GenericError(format!("There was no server with the name '{}'", name)).into(),
            );
        }
    };

    state.active_server_index = target_server_index as i32;

    Ok(())
}
