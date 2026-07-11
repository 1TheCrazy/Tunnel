mod http;

use crate::http::http_server::{HttpServer};
use tunnel_core::structs::server::Server;

#[tokio::main]
async fn main() {
    let http_server = Server::new();
    http_server.start().await
}
