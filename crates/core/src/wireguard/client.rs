use std::fs;
use std::process::Command;
use crate::{state::io_manager::{NODE_WG_CONFIG_PATH, ensure_parent_dir, wireguard_path}};

// Note:
// I've left a note here for all the poor souls that willingly read and try to understand this code.
// Wireguard does not expose a platform-agnostic API for installing/starting/stopping services
// Therefore Tunnel ALWAYS uses a different approach for Windows and Linux (I sadly use Windows myself on my home PC, so windows support is needed)
// Here is a good diagram sourced from the hells of ChatGPT:
/*
+-------------------------------------------+---------------------------------------------+-----------------------+
| Windows                                   | Linux                                       | Function              |
+-------------------------------------------+---------------------------------------------+-----------------------+
| wireguard.exe /installtunnelservice       | Copy config to /etc/wireguard/              | Install Service       |
|                                           | (no installation step beyond that)          |                       |
+-------------------------------------------+---------------------------------------------+-----------------------+
| Start-Service                             | systemctl start                             | Start Service         |
| WireGuardTunnel$<name>                    | wg-quick@<name>                             |                       |
+-------------------------------------------+---------------------------------------------+-----------------------+
| Stop-Service                              | systemctl stop                              | Stop Service          |
| WireGuardTunnel$<name>                    | wg-quick@<name>                             |                       |
+-------------------------------------------+---------------------------------------------+-----------------------+
| wireguard.exe /uninstalltunnelservice     | Remove /etc/wireguard/<name>.conf           | Remove Service        |
|                                           | (and optionally disable the service)        |                       |
+-------------------------------------------+---------------------------------------------+-----------------------+
*/
// I don't know of a reliable way to start a Wireguard Interface/Service beside the GUI (on Windows) and `wg-quick up` on Linux (which isn't available on Windows)
// `wg` only provides config/interface updates/inspects and is therefore not viable for bringing up or creating services.
// That's why we ALWAYS have to go the hacky route and use `systemctl start` (Linux) and Powershell's `Start-Service` (Windows)
// Actually not, on the node we use `node-quick up` because there we don't have to shut-down service since - ideally - it should run indefinetly
//
// Why am I writing this?
//
// See this as a plea to the Wireguard maintainers to pweeeeeeease add unified wrapper for this (since this project would have been SO much easier)
// (and to inform the coming generations of the horrors I've seen)
// Also randomly deciding to document random parts of this project is an uncontrollable urge (where leaving out key documentation is just as intriguing)

pub fn create_and_activate_client_conf(node_id: &str, client_private_key: &str, assigned_ip: &str, node_public_key: &str, endpoint: &str) -> Result<(), ()>{
    // Strip to 10 hex digits for linux service name restrictions
    let interface_id: String = node_id.chars().take(10).collect();
    let interface_name = format!("t_{}", interface_id);

    let path = wireguard_path().join(format!("{}.conf", interface_name));
    match ensure_parent_dir(&path) {
        Ok(()) => { },
        Err(_) => return Err(())
    };

    let mut conf = String::new();
    conf.push_str("[Interface]\n");
    conf.push_str(&format!("PrivateKey = {}\n", client_private_key));
    conf.push_str(&format!("Address = {}/32\n", assigned_ip));
    conf.push_str("\n[Peer]\n");
    conf.push_str(&format!("PublicKey = {}", node_public_key));
    conf.push_str(&format!("Endpoint = {}", endpoint));
    // TODO: sync with conf
    conf.push_str("AllowedIps = 10.8.0.0/24");
    conf.push_str("PersistentKeepalive = 25");

    match fs::write(path, conf) {
        Err(_) => return Err(()),
        Ok(()) => {}
    };

    match install_service_if_not_already(&interface_name) {
        Err(_) => return Err(()),
        Ok(()) => return Ok(())
    };
}

pub fn install_service_if_not_already(interface_name: &str) -> Result<(), ()> {
    let service_installed: bool;

    #[cfg(target_os = "windows")]
    {
        let output = match Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-Service 'WireGuardTunnel$*' | Select-Object -ExpandProperty Name",
        ])
        .output() {
            Ok(res) => res,
            Err(_) => return Err(())
        };

        let names = match String::from_utf8(output.stdout) {
            Ok(out) => out,
            Err(_) =>  return Err(())
        };

        service_installed = names.contains(&format!("WireGuardTunnel${}", interface_name));
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let output = match Command::new("sudo")
        .args([
            "ls",
            "/etc/wireguard/*.conf",
        ])
        .output() {
            Ok(res) => res,
            Err(_) => return Err(())
        };

        let names = match String::from_utf8(output.stdout) {
            Ok(out) => out,
            Err(_) =>  return Err(())
        };

        service_installed = names.contains(&format!("/etc/wireguard/{}.conf", interface_name));
    }

    if service_installed {
        return Ok(());
    }

    return install_service(interface_name);
}

pub fn install_service(interface_name: &str) -> Result<(), ()>{
    let conf_path = NODE_WG_CONFIG_PATH().join(format!("{}.conf", interface_name));
    let conf_path_str = match conf_path.to_str() {
        Some(value) => value,
        None => return Err(())
    };

    println!("{}", conf_path_str);

    #[cfg(target_os = "windows")]
    let status = Command::new(r"C:\Program Files\WireGuard\wireguard.exe")
        .arg("/installtunnelservice")
        .arg(format!("{}", conf_path_str))
        .status();
    
    #[cfg(not(target_os = "windows"))]
    let status = Command::new(r"sudo")
        .arg("cp")
        .arg(format!("{}", conf_path_str))
        .arg(format!("/etc/wireguard/{}", conf_path.file_name().and_then(|n| n.to_str()).unwrap()))
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

pub fn activate_service(service_name: &str) -> Result<(), ()>{
    deactivate_running_service()?;

    #[cfg(target_os = "windows")]
    {
        match Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Start-Service 'WireGuardTunnel${}'", service_name),
        ])
        .status() {
            Ok(status) =>  {
                if status.success() {
                    return Ok(())
                } else { 
                    return Err(())
                }
            },
            Err(_) => return Err(())
        };
    }

    #[cfg(not(target_os = "windows"))]
    {
        match Command::new("sudo")
        .args([
            "systemctl",
            "start",
            &format!("wg-quick@{}.service", service_name),
        ])
        .status() {
            Ok(status) =>  {
                if status.success() {
                    return Ok(())
                } else { 
                    return Err(())
                }
            },
            Err(_) => return Err(())
        };
    }
}

fn deactivate_running_service() -> Result<(), ()>{
    #[cfg(target_os = "windows")]
    let output = match Command::new(r"C:\Program Files\WireGuard\wg.exe")
        .arg("show")
        .output() {
            Ok(out) => out,
            Err(_) => return Err(())
        };
    
    #[cfg(not(target_os = "windows"))]
    let output = match Command::new(r"sudo")
        .arg("wg")
        .arg("show")
        .output() {
            Ok(out) => out,
            Err(_) => return Err(())
        };

    let out_string = match String::from_utf8( output.stdout) {
        Ok(str) => str,
        Err(_) => return Err(())
    };

    // Parse this output:
    //
    // interface: name
    //   public key: ...=
    //   private key: (hidden)
    //   ...
    let active_service_name = out_string.split("\n").next().unwrap().split(" ").nth(1).unwrap();

    #[cfg(target_os = "windows")]
    {
        match Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            &format!("Stop-Service 'WireGuardTunnel${}'", active_service_name),
        ])
        .status() {
            Ok(status) =>  {
                if status.success() {
                    return Ok(())
                } else { 
                    return Err(())
                }
            },
            Err(_) => return Err(())
        };
    }

    #[cfg(not(target_os = "windows"))]
    {
        match Command::new("sudo")
            .args([
                "systemctl", 
                "stop", 
                &format!("wg-quick@{}.service", active_service_name)
            ])
            .status()
        {
            Ok(status) => {
                if status.success() {
                    return Ok(());
                } else {
                    return Err(());
                }
            }
            Err(_) => return Err(()),
        };
    }
}