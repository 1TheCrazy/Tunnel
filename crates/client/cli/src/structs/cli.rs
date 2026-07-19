use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Refresh the node list and ips from the active server
    #[arg(short, long, global = true)]
    pub refresh: bool,

    /// Disables output
    #[arg(long, short, global = true)]
    pub quiet: bool
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all nodes avilable on the currently selected server
    ListNodes,

    /// List all servers that were added to the client
    ListServers,

    /// Manage servers
    Server {
        #[command(subcommand)]
        command: ServerCommand,
    },

    /// Connect to a node from the currently active server
    Connect {
        /// Id of the node to connect to (use `tunnel list-nodes` to obtain the id)
        id: i32
    }
}

#[derive(Subcommand, Debug)]
pub enum ServerCommand {
    /// Add a server
    Add {
        /// Name of the server
        name: String,

        /// Host of the server
        host: String,

        /// Password for the server
        #[arg(short, long)]
        password: Option<String>,
    },

    /// Remove a server
    Remove {
        /// Name of the server
        name: String,
    },

    /// Set a server to be the active server
    Set {
        name: String
    }
}