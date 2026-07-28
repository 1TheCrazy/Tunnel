use tunnel_core::constants::TUNNEL_SERVICE_PORT;
use tunnel_core::structs::save::NodeSave;
use tunnel_core::structs::config::NodeConfig;
use tunnel_core::{state::io_manager, structs::node::Node};
use axum::{
    Router,
};
use std::sync::{Arc, RwLock};
use tokio::{net::TcpListener};
use crate::http::state::AppState;
use crate::http::routes;
use crate::http::state::SharedServer;
use crate::util::registration::register_self;

pub trait HttpServer {
    fn from_config() -> SharedServer;
    async fn start(&self);
    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>;
}

impl HttpServer for SharedServer {
    fn from_config() -> SharedServer {
        let config = io_manager::read_config_or_default::<NodeConfig>(&io_manager::NODE_CONFIG_PATH());
        let save = io_manager::read_save_or_default::<NodeSave>(&io_manager::NODE_SAVE_PATH());

        return Arc::new(RwLock::new(Node {
            used_ips: save.used_ips.to_owned(),
            self_id: save.self_id.to_owned(),
            private_key: save.private_key.to_owned(),
            public_key: save.public_key.to_owned(),
            password: config.password.to_owned(),
            vpn_port: config.vpn_port.to_owned(),
            server_host: config.server_host.to_owned()
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

        register_self(&mut self.write().unwrap()).await;

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown)
            .await
            .expect("Failed to serve");
    }

    fn cleanup(&self) -> Result<(), Box<dyn std::error::Error>>{
        let server = self.read().expect("Server lock was poisened");

        let config_to_save = NodeSave {
            used_ips: server.used_ips.to_owned(),
            self_id: server.self_id.to_owned(),
            private_key: server.private_key.to_owned(),
            public_key: server.public_key.to_owned()
        };
        
        io_manager::write_save(&config_to_save, &io_manager::NODE_SAVE_PATH())?;

        Ok(())
    }
}