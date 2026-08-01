use std::time::Duration;

use reqwest::Client;
use tokio::time;
use tunnel_core::structs::{http::UpdateNodeRequest, node::Node};

pub fn register_updating(period: Duration, node: Node){
    tokio::spawn( async move {
        let mut interval = time::interval(period);

        loop {
            interval.tick().await;
            update_node(&node).await;
        }
    });
}

async fn update_node(node: &Node) {
    let client = Client::new();
    let req_body = UpdateNodeRequest {
        id: node.self_id.to_owned()
    };

    let update_req_res = match client
        .post(format!("http://{}/nodes/update", &node.server_host))
        .header("Tunnel-Authorization", &node.password)
        .json(&req_body)
        .send()
        .await 
    {
        Ok(res) => res,
        Err(err) => {
            println!("node: self update request failed error={}", err);
            return;
        }
    };

    if !update_req_res.status().is_success(){
        let status = update_req_res.status();
        let text = match update_req_res.text().await {
            Ok(text) => text,
            Err(_) => "".to_owned()
        };

        println!(
            "node: self update failed status={} response_body={}",
            status,
            text
        );
        return;
    }

    println!("node: updated self");

}