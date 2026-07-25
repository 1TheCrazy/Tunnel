use std::error::Error;

use tunnel_core::structs::{client::{ClientNode, ClientServer}, http::{DiscoverNodeRequest, DiscoverNodeResponse}, server::ServerNode};

use crate::structs::error::GenericError;

pub async fn get_nodes(host: &str, password: &str) -> Result<Vec<ClientNode>, Box<dyn Error>> {
    let client = reqwest::Client::new();
    
    let list_req = match client
        .get(format!("http://{}/nodes/list", host))
        .header("Tunnel-Authorization", password)
        .send()
        .await 
    {
        Ok(res) => res,
        Err(_) => { return Err("Error during http call".to_owned().into()); }
    };

    if !list_req.status().is_success() {
        return Err(format!("{}: {}", list_req.status().as_str(), list_req.text().await.unwrap_or("".to_owned())).into());
    }

    let body: Vec<ServerNode> = list_req.json().await?;
    let resolved = body.iter().map(|n| 
        ClientNode { 
            ip: n.ip.to_owned(), 
            port: n.port.to_owned(), 
            public_key: n.public_key.to_owned(), 
            id: n.id.to_owned(), 
            discovered: false
        }
    ).collect();

    Ok(resolved)
}

pub async fn discover_node(id: &str, server: &ClientServer, public_key: &str) -> Result<DiscoverNodeResponse, GenericError> {
    let client = reqwest::Client::new();

    let req_body: DiscoverNodeRequest = DiscoverNodeRequest { 
        id: id.to_owned(), 
        public_client_key: public_key.to_owned()
    };

    let discover_req_res = match client
        .post(format!("http://{}/nodes/discover", server.host))
        .header("Tunnel-Authorization", &server.password)
        .json(&req_body)
        .send()
        .await 
    {
        Ok(res) => res,
        Err(err) => {
            return Err(GenericError(format!("Server request error: {}", err)))
        }
    };

    if !discover_req_res.status().is_success() {
        let status = discover_req_res.status().to_string();
        let text = discover_req_res.text().await.unwrap_or(String::new());

        return Err(GenericError(format!("{}: {}", status, text)));
    }

    let res_body: DiscoverNodeResponse = match discover_req_res.json().await {
        Ok(body) => body,
        Err(_) => return Err(GenericError(format!("Malformed Server response")))
    };

    Ok(res_body)
}