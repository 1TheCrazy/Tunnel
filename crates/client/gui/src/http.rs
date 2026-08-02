use std::{error::Error, fmt};

use serde::Deserialize;
use tunnel_core::structs::{
    client::{ClientNode, ClientServer},
    http::{DiscoverNodeRequest, DiscoverNodeResponse},
    server::ServerNode,
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

pub async fn get_nodes(host: &str, password: &str) -> Result<Vec<ClientNode>, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let list_req = client
        .get(format!("http://{}/nodes/list", host))
        .header("Tunnel-Authorization", password)
        .send()
        .await
        .map_err(|_| "Error during http call".to_owned())?;

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
    server: &ClientServer,
    public_key: &str,
) -> Result<DiscoverNodeResponse, GuiError> {
    let client = reqwest::Client::new();
    let req_body = DiscoverNodeRequest {
        id: id.to_owned(),
        public_client_key: public_key.to_owned(),
    };

    let discover_req_res = client
        .post(format!("http://{}/nodes/discover", server.host))
        .header("Tunnel-Authorization", &server.password)
        .json(&req_body)
        .send()
        .await
        .map_err(|err| GuiError(format!("Server request error: {}", err)))?;

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
