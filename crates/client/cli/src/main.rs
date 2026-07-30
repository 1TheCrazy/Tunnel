mod structs;
mod util;
mod handlers;
mod http;

use std::process::ExitCode;
use clap::Parser;
use structs::cli::Cli;

use crate::{handlers::{connect::connect, disconnect::disconnect, list_nodes::list_nodes, list_servers::list_servers, server::server, }, structs::{cli::Commands, state::CliClientSave}, util::constants};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    constants::QUIET.set(cli.quiet).unwrap();

    if cli.refresh {
        write_line!("Refreshing nodes...\n");
        match CliClientSave::refresh().await {
            Ok(_) => { },
            Err(_) => return ExitCode::FAILURE
        };
    }

    let operation_res = match cli.command {
        Some(Commands::ListNodes) => list_nodes(),
        Some(Commands::ListServers) => list_servers(),
        Some(Commands::Connect { id } ) => connect(&id).await,
        Some(Commands::Server { command }) => server(command).await,
        Some(Commands::Disconnect) => disconnect(),
        None => /* --help or only global flags */ Ok(())
    };

    match operation_res {
        Ok(_) => return ExitCode::SUCCESS,
        Err(_) => return ExitCode::FAILURE
    }
}
