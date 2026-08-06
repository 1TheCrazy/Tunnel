use std::time::Duration;

use tokio::{sync::mpsc, time};
use tunnel_core::structs::http::{NodeToServerMessage, UpdateNodeRequest};

use crate::net::state::SharedState;

/// Starts the periodic node update task.
///
/// Messages are sent over a channel so the websocket owner remains the only
/// task that writes to the socket.
pub fn register_updating(
    period: Duration,
    node_state: SharedState,
    sender: mpsc::Sender<NodeToServerMessage>,
) {
    tokio::spawn(async move {
        let mut interval = time::interval(period);

        loop {
            interval.tick().await;

            let id = node_state.read().unwrap().self_id.clone();
            if id.is_empty() {
                continue;
            }

            if sender
                .send(NodeToServerMessage::Update(UpdateNodeRequest { id }))
                .await
                .is_err()
            {
                println!("node: update scheduler stopped because websocket closed");
                break;
            }
        }
    });
}
