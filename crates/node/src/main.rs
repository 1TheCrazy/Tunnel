mod http;
mod util;

use crate::http::{http_server::HttpServer, state::SharedServer};
use tunnel_core::wireguard::install;

#[tokio::main]
async fn main() {
    let wireguard_installed = install::is_wireguard_available();

    if !wireguard_installed {
        panic!("Wireguard not installed.\nInstalling wireguard through Tunnel is currently not supported.")
    }

    let http_server = SharedServer::from_config();
    
    // Start server
    http_server.start().await;
    
    // Cleanup after CTRL+C
    match http_server.cleanup(){
        Ok(()) => println!("Server successfully stoppped!"),
        Err(err) => println!("Error during server cleanup: {}", err)
    }
}
