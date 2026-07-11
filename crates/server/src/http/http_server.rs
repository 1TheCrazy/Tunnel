use tunnel_core::structs::server::Server;
use axum::{
    Router,
};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::RwLock};
use crate::http::state::AppState;
use crate::http::routes;

pub trait HttpServer {
    fn new() -> Server;
    fn new_with_port(port: &str) -> Server;
    async fn start(&self);
}

impl HttpServer for Server {
    fn new() -> Server {
        Server::new_with_port("8000")
    }

    fn new_with_port(port: &str) -> Server {
        Server {
            port: port.to_owned(),
            nodes: vec![],
        }
    }

    async fn start(&self) {
        let state: AppState = AppState {
            nodes: Arc::new(RwLock::new(self.nodes.clone()))
        };

        let app = 
            Router::new()
            .merge(routes::router())
            .with_state(state);

        let listener = 
            TcpListener::bind(format!("0.0.0.0:{}", self.port))
            .await
            .expect("Failed to bind to port");

        axum::serve(listener, app)
            .await
            .expect("Failed to serve");
    }
}