use tunnel_core::constants::TUNNEL_SERVICE_PORT;
use tunnel_core::structs::state::NodeConfig;
use tunnel_core::{state::save_manager, structs::node::Node};
use axum::{
    Router,
};
use std::sync::{Arc, RwLock};
use tokio::{net::TcpListener};
use crate::http::state::AppState;
use crate::http::routes;
use crate::http::state::SharedServer;

pub trait HttpServer {
    fn from_config() -> SharedServer;
    async fn start(&self);
    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl HttpServer for SharedServer {
    fn from_config() -> SharedServer {
        let config = save_manager::read_config_or_default::<NodeConfig>(&save_manager::SERVER_CONFIG_PATH());

        return Arc::new(RwLock::new(Node {
            used_ips: config.used_ips.to_owned(),
            password: config.password.to_owned()
        }))
    }

    async fn start(&self) {
        let state: AppState = AppState {
            server: Arc::clone(&self),
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

        let config_to_save = NodeConfig {
            password: server.password.to_owned(),
            used_ips: server.used_ips.to_owned()
        };
        
        save_manager::write_config(&config_to_save, &save_manager::SERVER_CONFIG_PATH())?;

        Ok(())
    }
}