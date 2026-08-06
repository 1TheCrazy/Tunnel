use std::sync::Arc;

use tunnel_core::{constants::TUNNEL_SERVICE_PORT, net::pinned_tls::{create_pinned_client, get_fingerprint_option}, structs::http::{CreateNodeRequest, CreateNodeResponse}};

use crate::net::state::SharedState;

pub async fn register_self(node_lock: SharedState) {
    let is_node_id_empty = {
        let node = node_lock.read().unwrap();
        node.self_id.is_empty()
    };

    if is_node_id_empty {
        let (server_host, vpn_port, host_fingerprint, public_key, password) = {
            let node = node_lock.read().unwrap();
            
            let server_host = node.server_host.clone();
            let vpn_port = node.vpn_port.clone();
            let host_fingerprint = node.host_fingerprint.clone();
            let public_key = node.public_key.clone();
            let password = node.password.clone();

            (server_host, vpn_port, host_fingerprint, public_key, password)
        }; // Release lock

        println!(
            "node: registering with server host={} vpn_port={}",
            server_host, vpn_port
        );

        let callback_state = Arc::clone(&node_lock);

        let client = create_pinned_client(
            &get_fingerprint_option(&host_fingerprint),
            &server_host, 
            Box::new(move |print| {
                let mut node = callback_state.write().unwrap();

                if node.host_fingerprint.is_empty() {
                    node.host_fingerprint = print;
                }
            }) 
        )
        .expect("Failed to create https client");
        
        let req_body = CreateNodeRequest {
            port: vpn_port.to_owned(),
            public_key: public_key.to_owned(),
        };

        let register_req_res = match client
            .post(format!(
                "https://{}:{}/nodes/register",
                &server_host,
                TUNNEL_SERVICE_PORT
            ))
            .header("Tunnel-Authorization", &password)
            .json(&req_body)
            .send()
            .await
        {
            Ok(res) => res,
            Err(err) => {
                println!("node: self registration request failed error={}", err);
                panic!("Wasn't able to register self: {}", err)
            }
        };

        if !register_req_res.status().is_success() {
            let status = register_req_res.status();
            let text = match register_req_res.text().await {
                Ok(text) => text,
                Err(_) => "".to_owned(),
            };

            println!(
                "node: self registration failed status={} response_body={}",
                status, text
            );
            panic!(
                "Wasn't able to register self - Server responded with non-success code: {}",
                text
            )
        }

        let json: CreateNodeResponse = match register_req_res.json().await {
            Ok(json) => json,
            Err(err) => {
                println!(
                    "node: self registration response decode failed error={}",
                    err
                );
                panic!("Wasn't able to register self: {}", err)
            }
        };

        let mut node  = node_lock.write().unwrap();

        node.self_id = json.assigned_id;
        println!(
            "node: self registration succeeded assigned_id={}",
            node.self_id
        );
    } else {
        println!(
            "node: already registered assigned_id={}",
            &node_lock.read().unwrap().self_id
        );
    }
}
