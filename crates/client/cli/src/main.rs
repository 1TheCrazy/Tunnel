mod structs;
mod util;
mod handlers;

use std::error::Error;
use clap::Parser;
use structs::cli::Cli;
use handlers::connect;

use crate::{handlers::{list_nodes::list_nodes, list_servers::list_servers, server::server}, structs::cli::Commands, util::constants};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    constants::QUIET.set(cli.quiet).unwrap();

    if cli.refresh {
        write_line!("Refreshing node ips...")
    }

    match cli.command {
        Some(Commands::ListNodes) => return list_nodes(),
        Some(Commands::ListServers) => return list_servers(),
        Some(Commands::Connect { id } ) => return connect::connect(id),
        Some(Commands::Server { command }) => return server(command),
        None => { /* --help or only global flags */}
    }

    Ok(())
}
