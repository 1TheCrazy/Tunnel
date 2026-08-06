mod net;
mod util;

use crate::net::{state::SharedState, websocket::WebSocketProvider};
use tunnel_core::wireguard::{
    common::{activate_service, install_service_if_not_already},
    install,
    node::{create_default_node_conf_if_not_exist, install_nat},
};

#[tokio::main]
async fn main() {
    #[cfg(target_os = "macos")]
    panic!("MacOS does is currently not supported by Tunnel::Node");

    let wireguard_installed = install::is_wireguard_available();

    if !wireguard_installed {
        panic!(
            "Wireguard not installed."
        )
    }

    let state = SharedState::from_config();

    {
        let node = state.read().unwrap();

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

    tokio::select! {
        _ = state.connect() => {
            println!("node: websocket connection ended");
        }
        result = tokio::signal::ctrl_c() => {
            match result {
                Ok(()) => println!("node: shutdown signal received"),
                Err(error) => println!("node: failed to listen for shutdown signal error={error}"),
            }
        }
    }

    match state.cleanup() {
        Ok(()) => println!("Server successfully stoppped!"),
        Err(err) => println!("Error during server cleanup: {}", err),
    }
}
