use tunnel_core::constants::TUNNEL_SERVICE_PORT;
use tunnel_core::structs::save::ServerSave;
use tunnel_core::{structs::server::Server, state::io_manager, structs::config::ServerConfig};
use axum::{
    Router,
};
use std::net::SocketAddr;
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
        let config = io_manager::read_config_or_default::<ServerConfig>(&io_manager::SERVER_CONFIG_PATH());
        let save = io_manager::read_save_or_default::<ServerSave>(&io_manager::SERVER_SAVE_PATH());

        println!(
            "server: loaded config and save state; known_nodes={}",
            save.nodes.len()
        );

        return Arc::new(RwLock::new(Server {
            nodes: save.nodes.to_owned(),
            password: config.password.to_owned(),
        }));
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

        println!("server: listening on 0.0.0.0:{}", TUNNEL_SERVICE_PORT);

        let shutdown = async { 
            tokio::signal::ctrl_c().
                await
               .expect("Unable to start server, since the shutdown hook cannot be installed");
            println!("server: shutdown signal received");
        };

        axum::serve(
                listener, 
                app.into_make_service_with_connect_info::<SocketAddr>()
            )
            .with_graceful_shutdown(shutdown)
            .await
            .expect("Failed to serve");
    }

    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>{
        let server = self.read().expect("Server lock was poisened");

        let config_to_save = ServerSave {
            nodes: server.nodes.to_owned()
        };

        println!(
            "server: saving state during cleanup; known_nodes={}",
            config_to_save.nodes.len()
        );
        
        io_manager::write_save(&config_to_save, &io_manager::SERVER_SAVE_PATH())?;

        Ok(())
    }
}
