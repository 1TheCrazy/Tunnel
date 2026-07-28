use std::{fs, process::Command};

use base64::{engine::general_purpose::STANDARD, Engine};
use x25519_dalek::{PublicKey, StaticSecret};

use crate::state::io_manager;

pub struct KeyPair {
    pub private: String,
    pub public: String,
}

pub fn gen_key_pair() -> KeyPair {
    let private = StaticSecret::random();
    let public = PublicKey::from(&private);

    KeyPair {
        private: STANDARD.encode(private.to_bytes()),
        public: STANDARD.encode(public.as_bytes()),
    }
}

pub fn add_peer(interface: &str, public_key: &str, allowed_ip: &str, endpoint: Option<&str>, persistent_keepalive: Option<&str>) -> Result<(), ()> {
    let mut command_base: Command;
    
    #[cfg(target_os = "windows")]
    {
        command_base = Command::new(r"C:\Program Files\WireGuard\wg.exe");
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        command_base = Command::new(r"sudo");
        command_base
            .arg("wg");
    }
    
    let interface_path = io_manager::wireguard_path().join(format!("{}.conf", interface));
    let mut conf_content = match fs::read(&interface_path) {
        Err(_) => return Err(()),
        Ok(bytes) => match String::from_utf8(bytes) {
            Err(_) => return Err(()),
            Ok(content) => content
        }
    };
    conf_content.push_str("\n\n");
    conf_content.push_str(&format!("PublicKey = {}", public_key));
    conf_content.push_str(&format!("AllowedIPs = {}", allowed_ip));

    command_base
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(allowed_ip);

    if let Some(resolved_endpoint) = endpoint {
        command_base
            .arg("endpoint")
            .arg(resolved_endpoint);

        conf_content.push_str(&format!("Endpoint = {}", resolved_endpoint));
    }

    if let Some(resolved_keepalive) = persistent_keepalive {
        command_base
            .arg("persistent-keepalive")
            .arg(resolved_keepalive);

        conf_content.push_str(&format!("PersistentKeepalive = {}", resolved_keepalive));
    }

    match fs::write(&interface_path, conf_content) {
        Err(_) => return Err(()),
        Ok(()) => {}
    };

    let exit_status = match command_base.status() {
        Ok(status) => status,
        Err(_) => return Err(())
    };

    if !exit_status.success() {
        return Err(());
    }

    Ok(())
}