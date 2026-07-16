use reqwest::Client;
use tunnel_core::{structs::{http::{CreateNodeRequest, CreateNodeResponse}, node::Node}, wireguard::common::gen_key_pair};


pub async fn register_self(self_server: &mut Node) {
    if self_server.self_id.is_empty() ||  self_server.private_key.is_empty() {
        let keys = gen_key_pair();

        let client = Client::new();
        let req_body = CreateNodeRequest {
            port: self_server.vpn_port.to_owned(),
            public_key: keys.public
        };

        let register_req_res = match client
            .post(format!("http://{}/nodes/register", &self_server.server_host))
            .header("Tunnel-Authorization", &self_server.password)
            .json(&req_body)
            .send()
            .await 
        {
            Ok(res) => res,
            Err(err) => panic!("Wasn't able to register self: {}", err)
        };

        if !register_req_res.status().is_success(){
            let text = match register_req_res.text().await {
                Ok(text) => text,
                Err(_) => "".to_owned()
            };

            panic!("Wasn't able to register self - Server responded with non-success code: {}", text)
        }

        let json: CreateNodeResponse = match register_req_res.json().await {
            Ok(json) => json,
            Err(err) => panic!("Wasn't able to register self: {}", err)
        };

        self_server.self_id = json.assigned_id;
        self_server.private_key = keys.private

    }
}