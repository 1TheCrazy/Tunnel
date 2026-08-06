use crate::SharedServer;
use crate::net::routes;
use crate::net::state::AppState;
use crate::util::tls::create_pem_files_if_not_already;
use axum::Router;
use axum_server::Handle;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::Duration;
use tunnel_core::constants::TUNNEL_SERVICE_PORT;
use tunnel_core::structs::save::ServerSave;
use tunnel_core::{state::io_manager, structs::config::ServerConfig, structs::server::Server};

pub trait HttpServer {
    fn from_config() -> SharedServer;
    async fn start(&self);
    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl HttpServer for SharedServer {
    fn from_config() -> SharedServer {
        let config = io_manager::read_config_or_default::<ServerConfig>(&io_manager::SERVER_CONFIG_PATH());
        let save = io_manager::read_save_or_default::<ServerSave>(&io_manager::SERVER_SAVE_PATH());

        create_pem_files_if_not_already(&config.self_hostname)
            .expect("Failed to create TLS pem files");

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
            node_connections: Arc::new(RwLock::new(HashMap::new())),
            pending_discoveries: Arc::new(RwLock::new(HashMap::new())),
        };

        let app = 
            Router::new()
            .merge(routes::router())
            .with_state(state);

        let address: SocketAddr = format!("0.0.0.0:{}", TUNNEL_SERVICE_PORT)
            .parse()
            .expect("Invalid server adress");

        let tls_config = 
            RustlsConfig::from_pem_file(io_manager::TLS_CERT_PATH(), io_manager::TLS_KEY_PATH())
            .await
            .expect("Failed to load TLS certificate or private key");

        let handle: Handle<SocketAddr> = Handle::new();
        let shutdown_handle = handle.clone();

        tokio::spawn(async move {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install shutdown hook");

            println!("server: shutdown signal received");

            shutdown_handle.graceful_shutdown(
                Some(Duration::from_secs(10)),
            );
        });        

        axum_server::bind_rustls(address, tls_config)
        .handle(handle)
        .serve(
            app.into_make_service_with_connect_info::<SocketAddr>()
        )
        .await
        .expect("Failed to serve");
    }

    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let server = self.read().expect("Server lock was poisened");

        let config_to_save = ServerSave {
            nodes: server.nodes.to_owned(),
        };

        println!(
            "server: saving state during cleanup; known_nodes={}",
            config_to_save.nodes.len()
        );

        io_manager::write_save(&config_to_save, &io_manager::SERVER_SAVE_PATH())?;

        Ok(())
    }
}
