use std::error::Error;

use tunnel_core::structs::{client::ClientNode, server::ServerNode};

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