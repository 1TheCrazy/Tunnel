mod http;
mod util;

use crate::http::{http_server::HttpServer, state::SharedServer};
use tunnel_core::structs::server::Server;

#[tokio::main]
async fn main() {
    let http_server = SharedServer::from_config();
    
    // Start server
    http_server.start().await;
    
    // Cleanup after CTRL+C
    match http_server.cleanup(){
        Ok(()) => println!("Server successfully stoppped!"),
        Err(err) => println!("Error during server cleanup: {}", err)
    }
}
