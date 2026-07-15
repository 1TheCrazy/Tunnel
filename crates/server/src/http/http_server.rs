use tunnel_core::constants::TUNNEL_SERVICE_PORT;
use tunnel_core::{structs::server::Server, state::save_manager, structs::state::ServerConfig};
use axum::{
    Router,
};
use std::sync::{Arc, RwLock};
use tokio::{net::TcpListener};
use crate::http::state::AppState;
use crate::http::routes;
use crate::SharedServer;

pub trait HttpServer {
    fn from_config() -> SharedServer;
    async fn start(&self);
    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl HttpServer for SharedServer {
    fn from_config() -> SharedServer {
        let config = save_manager::read_config_or_default::<ServerConfig>(&save_manager::SERVER_CONFIG_PATH());

        return Arc::new(RwLock::new(Server {
            nodes: config.nodes,
            password: config.password,
        }))
    }

    async fn start(&self) {
        let state: AppState = AppState {
            server: Arc::clone(&self),
            http_client: reqwest::Client::new()
        };

        let app = 
            Router::new()
            .merge(routes::router())
            .with_state(state);

        let listener = 
            TcpListener::bind(format!("0.0.0.0:{}", TUNNEL_SERVICE_PORT))
            .await
            .expect("Failed to bind to port");

        let shutdown = async { 
            tokio::signal::ctrl_c().
                await
               .expect("Unable to start server, since the shutdown hook cannot be installed");
        };

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
            .expect("Failed to serve");
    }

    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>{
        let server = self.read().expect("Server lock was poisened");

        let config_to_save = ServerConfig {
            password: server.password.to_owned(),
            nodes: server.nodes.to_owned()
        };
        
        save_manager::write_config(&config_to_save, &save_manager::SERVER_CONFIG_PATH())?;

        Ok(())
    }
}