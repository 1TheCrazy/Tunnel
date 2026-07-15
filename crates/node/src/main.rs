mod http;

use crate::http::{http_server::HttpServer, state::SharedServer};

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
