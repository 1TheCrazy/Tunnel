use std::sync::{Arc, RwLock};

use tunnel_core::structs::node::Node;

pub type SharedState = Arc<RwLock<Node>>;