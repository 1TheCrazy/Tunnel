mod http;

use crate::http::http_server::{HttpServer};
use tunnel_core::structs::server::Server;

#[tokio::main]
async fn main() {
    let http_server = Server::from_config();
    
    // Start server
    http_server.start().await;
    
    // Cleanup after CTRL+C
    match http_server.cleanup(){
        Ok(()) => println!("Server successfully stoppped!"),
        Err(err) => println!("Error during server cleanup: {}", err)
    }
}
