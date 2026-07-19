use std::error::Error;

use crate::structs::cli::{ServerCommand, ServerCommand::*};

pub fn server(command: ServerCommand) -> Result<(), Box<dyn Error>> {

    match command {
        Add {name, host, password} => return server_add(name, host, password),
        Remove { name } => return server_remove(name),
        Set { name } => return server_set(name)
    }
}

fn server_add(name: String, host: String, password: Option<String>) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn server_remove(name: String) -> Result<(), Box<dyn Error>> {
    Ok(())
}
fn server_set(name: String) -> Result<(), Box<dyn Error>> {
    Ok(())
}