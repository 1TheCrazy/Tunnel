use std::{error::Error, fmt::format};

use tunnel_core::structs::server::ServerNode;

pub async fn get_nodes(host: &str, password: &str) -> Result<Vec<ServerNode>, Box<dyn Error>> {
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

    Ok(body)
}