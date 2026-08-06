use std::{error::Error, sync::{Arc, Mutex}};

use tunnel_core::{
    constants::TUNNEL_SERVICE_PORT,
    net::pinned_tls::{create_pinned_client, get_fingerprint_option},
    structs::{
        client::{ClientNode, ClientServer},
        http::{DiscoverNodeRequest, DiscoverNodeResponse},
        server::ServerNode,
    },
};

use crate::structs::error::GenericError;

fn pinned_client(server: &ClientServer) -> Result<(reqwest::Client, Arc<Mutex<Option<String>>>), Box<dyn Error>> {
    let captured_fingerprint = Arc::new(Mutex::new(None));
    let callback_fingerprint = Arc::clone(&captured_fingerprint);
    let client = create_pinned_client(
        &get_fingerprint_option(&server.host_fingerprint),
        &server.host,
        Box::new(move |fingerprint| *callback_fingerprint.lock().unwrap() = Some(fingerprint)),
    )?;
    Ok((client, captured_fingerprint))
}

fn persist_initial_fingerprint(server: &mut ClientServer, captured: Arc<Mutex<Option<String>>>) {
    if server.host_fingerprint.is_empty() {
        if let Some(fingerprint) = captured.lock().unwrap().take() {
            println!("client: trusted initial server certificate fingerprint={fingerprint}");
            server.host_fingerprint = fingerprint;
        }
    }
}

pub async fn get_nodes(server: &mut ClientServer) -> Result<Vec<ClientNode>, Box<dyn Error>> {
    let (client, captured_fingerprint) = pinned_client(server)?;
    let list_req = client
        .get(format!("https://{}:{}/nodes/list", server.host, TUNNEL_SERVICE_PORT))
        .header("Tunnel-Authorization", &server.password)
        .send()
        .await
        .map_err(|_| "Error during HTTPS call".to_owned())?;
    persist_initial_fingerprint(server, captured_fingerprint);

    if !list_req.status().is_success() {
        return Err(format!("{}: {}", list_req.status().as_str(), list_req.text().await.unwrap_or_default()).into());
    }

    let body: Vec<ServerNode> = list_req.json().await?;
    Ok(body.into_iter().map(|n| ClientNode {
        name: n.name,
        ip: n.ip,
        port: n.port,
        public_key: n.public_key,
        id: n.id,
        discovered: false,
    }).collect())
}

pub async fn discover_node(
    id: &str,
    server: &mut ClientServer,
    public_key: &str,
) -> Result<DiscoverNodeResponse, GenericError> {
    let (client, captured_fingerprint) = pinned_client(server)
        .map_err(|error| GenericError(format!("Failed to configure HTTPS client: {error}")))?;
    let req_body = DiscoverNodeRequest { id: id.to_owned(), public_client_key: public_key.to_owned() };
    let response = client
        .post(format!("https://{}:{}/nodes/discover", server.host, TUNNEL_SERVICE_PORT))
        .header("Tunnel-Authorization", &server.password)
        .json(&req_body)
        .send()
        .await
        .map_err(|error| GenericError(format!("Server request error: {error}")))?;
    persist_initial_fingerprint(server, captured_fingerprint);

    if !response.status().is_success() {
        return Err(GenericError(format!("{}: {}", response.status(), response.text().await.unwrap_or_default())));
    }
    response.json().await.map_err(|_| GenericError("Malformed Server response".to_owned()))
}
