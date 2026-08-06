use crate::net::state::SharedState;
use crate::util::registration::register_self;
use crate::util::update::register_updating;
use crate::net::handlers::discover::discover;
use axum::http::HeaderValue;
use tokio::sync::mpsc;
use tokio_tungstenite::Connector;
use tokio_tungstenite::connect_async_tls_with_config;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tunnel_core::constants::TUNNEL_SERVICE_PORT;
use tunnel_core::net::pinned_tls::get_fingerprint_option;
use tunnel_core::net::pinned_tls::get_pinned_tls_config;
use tunnel_core::structs::http::{NodeToServerMessage, ServerToNodeMessage};
use std::sync::{Arc, RwLock};
use tunnel_core::structs::config::NodeConfig;
use tunnel_core::structs::save::NodeSave;
use tunnel_core::wireguard::common::deactivate_running_service;
use tunnel_core::wireguard::node::uninstall_nat;
use tunnel_core::{state::io_manager, structs::node::Node};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};

pub trait WebSocketProvider {
    fn from_config() -> SharedState;
    async fn connect(&self);
    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl WebSocketProvider for SharedState {
    fn from_config() -> SharedState {
        let config =
            io_manager::read_config_or_default::<NodeConfig>(&io_manager::NODE_CONFIG_PATH());
        let save = io_manager::read_save_or_default::<NodeSave>(&io_manager::NODE_SAVE_PATH());
        
        let fingerprint = 
        if !config.host_fingerprint.is_empty() {
            config.host_fingerprint
        } else {
            save.host_fingerprint
        };

        if fingerprint.is_empty() && !config.blindly_trust_host {
            panic!("Invalid config: there is no fingerprint available but the node should also not trust the host.")
        }

        println!(
            "node: loaded config and save state; registered={} used_ips={}",
            !save.self_id.is_empty(),
            save.used_ips.len()
        );

        return Arc::new(RwLock::new(Node {
            used_ips: save.used_ips.to_owned(),
            self_id: save.self_id.to_owned(),
            private_key: save.private_key.to_owned(),
            public_key: save.public_key.to_owned(),
            password: config.password.to_owned(),
            vpn_port: config.vpn_port.to_owned(),
            server_host: config.server_host.to_owned(),
            update_period: config.update_period.to_owned(),
            blindly_trust_host: config.blindly_trust_host,
            host_fingerprint: fingerprint.to_owned()
        }));
    }

    async fn connect(&self) {
        // Registration may establish the certificate fingerprint and assign the
        // node ID, so it must complete before the websocket settings are read.
        register_self(self.clone()).await;

        let (host_fingerprint, server_host, password, update_period) = {
            let state = self.read().unwrap();
            (
                state.host_fingerprint.clone(),
                state.server_host.clone(),
                state.password.clone(),
                state.update_period,
            )
        };
        let fingerprint = get_fingerprint_option(&host_fingerprint);

        let tls_config = get_pinned_tls_config(
            &fingerprint,
            &server_host,
            Box::new(|_| {
                panic!("encountered certificate that should've already been known") // This shouldn't happen since initial fingerprint discovery is never on the websocket, but on register
            })
        ).expect("Wasn't able to create TLs config");

        let connector = Connector::Rustls(Arc::new(tls_config));

        let mut request =
            format!("wss://{}:{}/nodes/websocket", server_host, TUNNEL_SERVICE_PORT)
            .into_client_request()
            .expect("Wasn't able to create websocket request"); 

        request.headers_mut().insert(
            "Tunnel-Authorization",
            HeaderValue::from_str(&password).expect("Wasn't able to forge HeaderValue from string"),
        );

        // Connect websocket
        let (mut websocket, _response) =
            connect_async_tls_with_config(
                request,
                None, // Default WebSocketConfig
                false, // Keep Nagle enabled
                Some(connector),
            )
            .await
            .expect("Wasnt able to connect to websocket");

        let (webhook_sender, mut webhook_receiver) = mpsc::channel(16);
        let node_id = self.read().unwrap().self_id.clone();
        webhook_sender
            .send(NodeToServerMessage::Connected { node_id })
            .await
            .expect("Failed to queue websocket registration");
        register_updating(update_period, self.clone(), webhook_sender.clone());

        loop {
            tokio::select! {
                Some(message) = webhook_receiver.recv() => {
                    let body = match serde_json::to_string(&message) {
                        Ok(body) => body,
                        Err(error) => {
                            println!("node: failed to serialize websocket message error={error}");
                            continue;
                        }
                    };

                    if let Err(error) = websocket.send(Message::Text(body.into())).await {
                        println!("node: failed to send webhook message error={error}");
                        break;
                    }
                }

                result = websocket.next() => match result {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ServerToNodeMessage>(&text) {
                            Ok(ServerToNodeMessage::DiscoverRequest { request_id, request }) => {
                                tokio::spawn(discover(self.clone(), request_id, request, webhook_sender.clone()));
                            }
                            Err(error) => println!("node: invalid websocket message error={error}"),
                        }
                    }

                    Some(Ok(Message::Close(frame))) => {
                        println!("Connection closed: {frame:?}");
                        break;
                    }

                    Some(Err(error)) => {
                        println!("node: websocket receive failed error={error}");
                        break;
                    }

                    Some(Ok(Message::Ping(data))) => {
                        if let Err(error) = websocket.send(Message::Pong(data)).await {
                            println!("node: failed to send websocket pong error={error}");
                            break;
                        }
                    }

                    Some(_) => {},

                    None => break,
                },
            }
        }
    }

    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let state = self.read().expect("State lock was poisened");

        let config_to_save = NodeSave {
            used_ips: state.used_ips.to_owned(),
            self_id: state.self_id.to_owned(),
            private_key: state.private_key.to_owned(),
            public_key: state.public_key.to_owned(),
            host_fingerprint: state.host_fingerprint.to_owned(),
        };

        println!(
            "node: saving state during cleanup; registered={} used_ips={}",
            !config_to_save.self_id.is_empty(),
            config_to_save.used_ips.len()
        );

        io_manager::write_save(&config_to_save, &io_manager::NODE_SAVE_PATH())?;

        match deactivate_running_service() {
            Ok(()) => {
                println!("node: deactivated Tunnel service");
            }
            Err(_) => {
                println!("node: wasn't able to deactivate Tunnel service")
            }
        }

        match uninstall_nat() {
            Ok(()) => {
                println!("node: uninstalled NAT");
            }
            Err(_) => {
                println!("node: wasn't able to uninstall NAT");
            }
        }

        Ok(())
    }
}
