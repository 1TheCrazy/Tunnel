use std::{fs, sync::RwLockWriteGuard};
use std::process::Command;
use crate::structs::node::Node;
use crate::{state::io_manager::{NODE_WG_CONFIG_PATH, ensure_parent_dir}, wireguard::common};

pub fn register_client(client_public_key: &str, server: &mut RwLockWriteGuard<'_, Node>) -> Option<String> {
    let proposed_ip_part = (2u8..=254).find(|candidate| {
        !server
            .used_ips
            .iter()
            .any(|ip| ip.ends_with(&format!(".{candidate}")))
    });

    let assigned_ip = match proposed_ip_part {
        Some(ip) => format!("10.8.0.{}", ip.to_string()),
        None => {
            // TODO: fix the logging
            // TODO: add ipv6 support
            println!("Ran out of ips to assign");
            return None
        }
    };

    println!("adding peer with ip: {}", assigned_ip);

    match add_peer(client_public_key, &assigned_ip) {
        Ok(()) => {},
        Err(()) => return None
    };

    server.used_ips.push(assigned_ip.clone());

    Some(assigned_ip)
}

pub fn create_default_node_conf_if_not_exist(node_private_key: &str, port: &str) -> Result<(), ()>{
    let path = NODE_WG_CONFIG_PATH();

    if path.exists() {
        return Ok(());
    }

    match create_default_node_conf(node_private_key, port){
        Err(_) => return Err(()),
        Ok(()) => return Ok(())
    };
}

pub fn create_default_node_conf(node_private_key: &str, port: &str) -> Result<(), ()>{
    let path = NODE_WG_CONFIG_PATH();

    match ensure_parent_dir(&path) {
        Ok(()) => { },
        Err(_) => return Err(())
    };

    let mut conf = String::new();
    conf.push_str("[Interface]\n");
    conf.push_str(&format!("PrivateKey = {}\n", node_private_key));
    // TODO: make this ipv6 ?
    // TODO: make the range/ip configurable
    conf.push_str("Address = 10.8.0.1/24\n");
    conf.push_str(&format!("ListenPort = {}\n", port));

    // Add NAT translations
    conf.push_str("\n# Enable automatic forwarding\n");
    conf.push_str("PostUp = sysctl -w net.ipv4.ip_forward=1\n");
    conf.push_str("PostUp = iptables -t nat -A POSTROUTING -o $(ip route get 1.1.1.1 | awk '{print $5; exit}') -j MASQUERADE\n\n");

    conf.push_str("PostDown = iptables -t nat -D POSTROUTING -o $(ip route get 1.1.1.1 | awk '{print $5; exit}') -j MASQUERADE\n");

    match fs::write(path, conf) {
        Err(_) => return Err(()),
        Ok(()) => return Ok(())
    };
}

pub fn install_service_if_not_already() -> Result<(), ()> {
    // This is ok, since the Windows-Service is started automtically on reboot
    #[cfg(target_os = "windows")]
    let out = Command::new(r"C:\Program Files\WireGuard\wg.exe")
        .arg("show")
        .output();
    
    #[cfg(not(target_os = "windows"))]
    let out = Command::new(r"sudo")
        .arg("wg")
        .arg("show")
        .output();

    let output = match out {
        Ok(exit) => exit,
        Err(_) => return Err(()) 
    };

    if !output.status.success() {
        return Err(());
    }

    let output_str = match String::from_utf8(output.stdout) {
        Ok(value) => value,
        Err(_) => return Err(())
    };

    if output_str.starts_with("interface: tunnel_0_node") {
        return Ok(());
    }

    return install_service();
}

pub fn install_service() -> Result<(), ()>{
    let conf_path = NODE_WG_CONFIG_PATH();
    let conf_path_str = match conf_path.to_str() {
        Some(value) => value,
        None => return Err(())
    };

    #[cfg(target_os = "windows")]
    let status = Command::new(r"C:\Program Files\WireGuard\wireguard.exe")
        .arg("/installtunnelservice")
        .arg(format!("{}", conf_path_str))
        .status();
    
    #[cfg(not(target_os = "windows"))]
    let status = Command::new(r"sudo")
        .arg("wg-quick")
        .arg("up")
        .arg(format!("{}", conf_path_str))
        .status();

    let exit = match status {
        Ok(exit) => exit,
        Err(_) => return Err(()) 
    };

    if !exit.success() {
        return Err(());
    }

    Ok(())
}

fn add_peer(client_public_key: &str, assigned_ip: &str) -> Result<(), ()>{
    common::add_peer("tunnel_0_node", client_public_key, assigned_ip, None, None)
}