mod net;
mod util;

use crate::net::{https_server::HttpServer, state::SharedServer};

#[tokio::main]
async fn main() {
    let https_server = SharedServer::from_config();

    // Start server
    https_server.start().await;

    // Cleanup after CTRL+C
    match https_server.cleanup() {
        Ok(()) => println!("Server successfully stoppped!"),
        Err(err) => println!("Error during server cleanup: {}", err),
    }
}
