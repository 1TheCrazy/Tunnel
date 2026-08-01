use reqwest::Client;
use tunnel_core::structs::{
    http::{CreateNodeRequest, CreateNodeResponse},
    node::Node,
};

pub async fn register_self(self_server: &mut Node) {
    if self_server.self_id.is_empty() {
        println!(
            "node: registering with server host={} vpn_port={}",
            self_server.server_host, self_server.vpn_port
        );

        let client = Client::new();
        let req_body = CreateNodeRequest {
            port: self_server.vpn_port.to_owned(),
            public_key: self_server.public_key.to_owned(),
        };

        let register_req_res = match client
            .post(format!(
                "http://{}/nodes/register",
                &self_server.server_host
            ))
            .header("Tunnel-Authorization", &self_server.password)
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

        self_server.self_id = json.assigned_id;
        println!(
            "node: self registration succeeded assigned_id={}",
            self_server.self_id
        );
    } else {
        println!(
            "node: already registered assigned_id={}",
            self_server.self_id
        );
    }
}
