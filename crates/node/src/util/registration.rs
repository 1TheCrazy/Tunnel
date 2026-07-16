use reqwest::Client;
use tunnel_core::{constants::{IS_DEV, TUNNEL_SERVICE_PORT}, structs::{http::{CreateNodeRequest, CreateNodeResponse}, node::Node}, wireguard::common::gen_key_pair};


pub async fn register_self(self_server: &mut Node) {
    if self_server.self_id.is_empty() {
        let client = Client::new();
        let server_host = match IS_DEV {
            true => "localhost",
            false => "" // TODO: implement cfg here
        };

        let req_body = CreateNodeRequest {
            port: "1234".to_owned(), // TODO: implement cfg here
            public_key: gen_key_pair()
        };

        let register_req_res = match client
            .post(format!("http://{}:{}/nodes/register", server_host, TUNNEL_SERVICE_PORT))
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

    }
}