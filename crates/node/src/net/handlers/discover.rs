use crate::net::state::SharedState;
use tokio::sync::mpsc;
use tunnel_core::{
    structs::http::{CreateClientOnNodeRequest, CreateClientOnNodeRespone, NodeToServerMessage},
    wireguard::node::register_client,
};


pub async fn discover(
    state: SharedState,
    request_id: String,
    request: CreateClientOnNodeRequest,
    sender: mpsc::Sender<NodeToServerMessage>,
) {
    let assigned_ip = {
        let mut node = state.write().unwrap();
        register_client(&request.public_client_key, &mut node)
    };

    let response = match assigned_ip {
        Some(assigned_ip) => {
            println!(
                "node: request discover -> assigned_vpn_ip={}",
                assigned_ip
            );

            CreateClientOnNodeRespone {
                success: true,
                vpn_network_ip: assigned_ip,
            }
        }
        None => {
            println!(
                "node: request discover -> client_registration_refused"
            );
            CreateClientOnNodeRespone {
                success: false,
                vpn_network_ip: String::new(),
            }
        }
    };

    match sender.send(NodeToServerMessage::DiscoverResponse { request_id, response }).await {
        Ok(_) => {},
        Err(_) => println!("Failed to send client registration to server"),
    }
}
