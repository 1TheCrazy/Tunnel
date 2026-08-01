use crate::{
    state::io_manager::{ensure_parent_dir, wireguard_path},
    wireguard::{
        common::{activate_service, install_service_if_not_already},
        util::interface_name_from_node_id,
    },
};
use std::fs;

pub fn create_and_activate_client_conf(
    node_id: &str,
    client_private_key: &str,
    assigned_ip: &str,
    node_public_key: &str,
    endpoint: &str,
) -> Result<(), ()> {
    let interface_name = interface_name_from_node_id(node_id);

    let path = wireguard_path().join(format!("{}.conf", interface_name));
    match ensure_parent_dir(&path) {
        Ok(()) => {}
        Err(_) => return Err(()),
    };

    let mut conf = String::new();
    conf.push_str("[Interface]\n");
    conf.push_str(&format!("PrivateKey = {}\n", client_private_key));
    conf.push_str(&format!("Address = {}/32\n", assigned_ip));
    conf.push_str("DNS = 1.1.1.1");
    conf.push_str("\n[Peer]\n");
    conf.push_str(&format!("PublicKey = {}\n", node_public_key));
    conf.push_str(&format!("Endpoint = {}\n", endpoint));
    // TODO: sync full-/split-tunnel with config
    // TODO: Add ipv6 support
    conf.push_str("AllowedIPs = 0.0.0.0/0\n");
    conf.push_str("PersistentKeepalive = 25\n");

    match fs::write(path, conf) {
        Err(_) => return Err(()),
        Ok(()) => {}
    };

    match install_service_if_not_already(&interface_name) {
        Err(_) => return Err(()),
        Ok(()) => {}
    };

    // Service is automatically activated on Windows, but not on linux
    match activate_service(&interface_name) {
        Err(_) => return Err(()),
        Ok(()) => return Ok(()),
    }
}
