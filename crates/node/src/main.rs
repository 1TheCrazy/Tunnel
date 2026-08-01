mod http;
mod util;

use crate::http::{http_server::HttpServer, state::SharedServer};
use tunnel_core::wireguard::{
    common::{activate_service, install_service_if_not_already},
    install,
    node::{create_default_node_conf_if_not_exist, install_nat},
};

#[tokio::main]
async fn main() {
    let wireguard_installed = install::is_wireguard_available();

    if !wireguard_installed {
        panic!(
            "Wireguard not installed.\nInstalling wireguard through Tunnel is currently not supported."
        )
    }

    let http_server = SharedServer::from_config();

    {
        let node = http_server.read().unwrap();

        match create_default_node_conf_if_not_exist(&node.private_key, &node.vpn_port) {
            Ok(()) => {}
            Err(()) => panic!("Wasn't able to create default Wireguard node config."),
        };

        match install_service_if_not_already("tunnel_0_node") {
            Ok(()) => {}
            Err(()) => panic!("Wasn't able to install the node tunnel service"),
        };

        match activate_service("tunnel_0_node") {
            Ok(()) => {}
            Err(()) => panic!("Wasn't able to start the node tunnel service"),
        };

        match install_nat() {
            Ok(()) => {}
            Err(()) => panic!("Wasn't able to install NAT"),
        }
    } // Drop Lock

    // Start server
    http_server.start().await;

    // Cleanup after CTRL+C
    match http_server.cleanup() {
        Ok(()) => println!("Server successfully stoppped!"),
        Err(err) => println!("Error during server cleanup: {}", err),
    }
}
