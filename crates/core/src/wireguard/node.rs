use crate::structs::node::Node;
#[cfg(target_os = "linux")]
use crate::wireguard::util::get_internet_interface_name;
use crate::{
    state::io_manager::{NODE_WG_CONFIG_PATH, ensure_parent_dir},
    wireguard::common,
};
use std::process::Command;
use std::{fs, sync::RwLockWriteGuard};
#[cfg(target_os = "windows")]
use crate::util::terminal::WINDOWS_INVISIBLE_TERMIAL;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub fn register_client(
    client_public_key: &str,
    node: &mut RwLockWriteGuard<'_, Node>,
) -> Option<String> {
    let proposed_ip_part = (2u8..=254).find(|candidate| {
        !node
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
            return None;
        }
    };

    println!("adding peer with ip: {}", assigned_ip);

    match add_peer(client_public_key, &assigned_ip) {
        Ok(()) => {}
        Err(()) => return None,
    };

    node.used_ips.push(assigned_ip.clone());

    Some(assigned_ip)
}

pub fn create_default_node_conf_if_not_exist(node_private_key: &str, port: &str) -> Result<(), ()> {
    let path = NODE_WG_CONFIG_PATH();

    if path.exists() {
        return Ok(());
    }

    match create_default_node_conf(node_private_key, port) {
        Err(_) => return Err(()),
        Ok(()) => return Ok(()),
    };
}

pub fn create_default_node_conf(node_private_key: &str, port: &str) -> Result<(), ()> {
    let path = NODE_WG_CONFIG_PATH();

    match ensure_parent_dir(&path) {
        Ok(()) => {}
        Err(_) => return Err(()),
    };

    let mut conf = String::new();
    conf.push_str("[Interface]\n");
    conf.push_str(&format!("PrivateKey = {}\n", node_private_key));
    // TODO: make this ipv6 ?
    // TODO: make the range/ip configurable
    conf.push_str("Address = 10.8.0.1/24\n");
    conf.push_str(&format!("ListenPort = {}\n", port));

    match fs::write(path, conf) {
        Err(_) => return Err(()),
        Ok(()) => return Ok(()),
    };
}

pub fn install_nat() -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        // Ready IP forwarding
        match Command::new("sudo")
            .args(["sysctl", "-w", "net.ipv4.ip_forward=1"])
            .status()
        {
            Ok(exit) => {
                if !exit.success() {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        };

        // Ready NAT mapping
        let internet_interface = get_internet_interface_name()?;

        match Command::new("sudo")
            .args([
                "iptables",
                "-t",
                "nat",
                "-A",
                "POSTROUTING",
                "-o",
                &internet_interface,
                "-j",
                "MASQUERADE",
            ])
            .status()
        {
            Ok(exit) => {
                if !exit.success() {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }
    }

    #[cfg(target_os = "windows")]
    {
        match Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args([
                "Set-ItemProperty",
                "-Path",
                "\"HKLM:\\SYSTEM\\CurrentControlSet\\Services\\Tcpip\\Parameters\"",
                "-Name",
                "IPEnableRouter",
                "-Value",
                "1",
            ])
            .status()
        {
            Ok(exit) => {
                if !exit.success() {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }

        match Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args([
                "New-NetNat",
                "-Name",
                "TunnelNat",
                "-InternalIPInterfaceAddressPrefix",
                "10.8.0.0/24",
            ])
            .status()
        {
            Ok(exit) => {
                if !exit.success() {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }
    }

    Ok(())
}

pub fn uninstall_nat() -> Result<(), ()> {
    #[cfg(target_os = "linux")]
    {
        let internet_interface = get_internet_interface_name()?;

        // Uninstall NAT rule (this isn't really neccessary, but whatever)
        match Command::new("sudo")
            .args([
                "iptables",
                "-t",
                "nat",
                "-D",
                "POSTROUTING",
                "-o",
                &internet_interface, // This assumes a stable internet interface
                "-j",
                "MASQUERADE",
            ])
            .status()
        {
            Ok(exit) => {
                if !exit.success() {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Uninstall NAT rule (this isn't really neccessary, but whatever)
        match Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args(["Remove-NetNat", "-Name", "TunnelNat", "-Confirm:$false"])
            .status()
        {
            Ok(exit) => {
                if !exit.success() {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        }
    }

    Ok(())
}

fn add_peer(client_public_key: &str, assigned_ip: &str) -> Result<(), ()> {
    common::add_peer("tunnel_0_node", client_public_key, assigned_ip, None, None)
}
