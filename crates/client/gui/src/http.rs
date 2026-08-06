use std::{error::Error, fmt, sync::{Arc, Mutex}};

use serde::Deserialize;
use tunnel_core::{
    constants::TUNNEL_SERVICE_PORT,
    net::pinned_tls::{create_pinned_client, get_fingerprint_option},
    structs::{client::{ClientNode, ClientServer}, http::{DiscoverNodeRequest, DiscoverNodeResponse}, server::ServerNode},
};

use crate::state::NodeLocation;

#[derive(Debug)]
pub struct GuiError(pub String);

impl fmt::Display for GuiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for GuiError {}

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
        .map_err(|_| "Error during http call".to_owned())?;
    persist_initial_fingerprint(server, captured_fingerprint);

    if !list_req.status().is_success() {
        return Err(format!(
            "{}: {}",
            list_req.status().as_str(),
            list_req.text().await.unwrap_or(String::new())
        )
        .into());
    }

    let body: Vec<ServerNode> = list_req.json().await?;
    Ok(body
        .iter()
        .map(|n| ClientNode {
            ip: n.ip.to_owned(),
            port: n.port.to_owned(),
            public_key: n.public_key.to_owned(),
            id: n.id.to_owned(),
            discovered: false,
        })
        .collect())
}

pub async fn discover_node(
    id: &str,
    server: &mut ClientServer,
    public_key: &str,
) -> Result<DiscoverNodeResponse, GuiError> {
    let (client, captured_fingerprint) = pinned_client(server)
        .map_err(|error| GuiError(format!("Failed to configure HTTPS client: {error}")))?;
    let req_body = DiscoverNodeRequest {
        id: id.to_owned(),
        public_client_key: public_key.to_owned(),
    };

    let discover_req_res = client
        .post(format!("https://{}:{}/nodes/discover", server.host, TUNNEL_SERVICE_PORT))
        .header("Tunnel-Authorization", &server.password)
        .json(&req_body)
        .send()
        .await
        .map_err(|err| GuiError(format!("Server request error: {}", err)))?;
    persist_initial_fingerprint(server, captured_fingerprint);

    if !discover_req_res.status().is_success() {
        let status = discover_req_res.status().to_string();
        let text = discover_req_res.text().await.unwrap_or(String::new());
        return Err(GuiError(format!("{}: {}", status, text)));
    }

    discover_req_res
        .json()
        .await
        .map_err(|_| GuiError("Malformed Server response".to_owned()))
}

#[derive(Deserialize)]
struct CountryLookup {
    ip: String,
    location: Option<CountryLocation>,
}

#[derive(Deserialize)]
struct CountryLocation {
    latitude: f64,
    longitude: f64,
}

pub async fn get_node_locations(ips: &[String]) -> Result<Vec<(String, NodeLocation)>, GuiError> {
    if ips.is_empty() {
        return Ok(Vec::new());
    }

    let response = reqwest::Client::new()
        .post("https://api.country.is/?fields=location")
        .json(ips)
        .send()
        .await
        .map_err(|err| GuiError(format!("Country lookup request error: {}", err)))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(GuiError(format!(
            "Country lookup failed with {}: {}",
            status, text
        )));
    }

    let lookups: Vec<CountryLookup> = response
        .json()
        .await
        .map_err(|err| GuiError(format!("Malformed country lookup response: {}", err)))?;

    Ok(lookups
        .into_iter()
        .filter_map(|lookup| {
            lookup.location.map(|location| {
                (
                    lookup.ip,
                    NodeLocation {
                        latitude: location.latitude,
                        longitude: location.longitude,
                    },
                )
            })
        })
        .collect())
}
