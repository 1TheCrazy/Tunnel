use tunnel_core::{structs::server::Server, state::save_manager, structs::state::ServerConfig};
use axum::{
    Router,
};
use std::sync::{Arc, RwLock};
use tokio::{net::TcpListener};
use crate::http::state::AppState;
use crate::http::routes;

pub trait HttpServer {
    fn from_config() -> Server;
    async fn start(&self);
    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;

}

impl HttpServer for Server {
    fn from_config() -> Server {
        let config = save_manager::read_config_or_default::<ServerConfig>(&save_manager::SERVER_CONFIG_PATH());

        Server {
            port: config.port,
            nodes: config.nodes,
            password: config.password
        }
    }

    async fn start(&self) {
        let state: AppState = AppState {
            nodes: Arc::new(RwLock::new(self.nodes.clone())),
            password: self.password.clone()
        };

        let app = 
            Router::new()
            .merge(routes::router())
            .with_state(state);

        let listener = 
            TcpListener::bind(format!("0.0.0.0:{}", self.port))
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
        let config_to_save = ServerConfig {
            password: self.password.to_owned(),
            port: self.port.to_owned(),
            nodes: self.nodes.to_owned()
        };
        
        save_manager::write_config(&config_to_save, &save_manager::SERVER_CONFIG_PATH())?;

        Ok(())
    }
}