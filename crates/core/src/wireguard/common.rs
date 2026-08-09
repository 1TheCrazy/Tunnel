use std::{fs, process::Command};

use base64::{Engine, engine::general_purpose::STANDARD};
use x25519_dalek::{PublicKey, StaticSecret};

#[cfg(target_os = "windows")]
use crate::util::terminal::WINDOWS_INVISIBLE_TERMIAL;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::state::io_manager::{self, wireguard_path};

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

pub fn add_peer(
    interface: &str,
    public_key: &str,
    allowed_ip: &str,
    endpoint: Option<&str>,
    persistent_keepalive: Option<&str>,
) -> Result<(), ()> {
    let mut command_base: Command;

    #[cfg(target_os = "windows")]
    {
        command_base = Command::new(r"C:\Program Files\WireGuard\wg.exe");
        command_base.creation_flags(WINDOWS_INVISIBLE_TERMIAL);
    }

    #[cfg(target_os = "linux")]
    {
        command_base = Command::new(r"sudo");
        command_base.arg("wg");
    }

    let interface_path = io_manager::wireguard_path().join(format!("{}.conf", interface));
    let mut conf_content = match fs::read(&interface_path) {
        Err(_) => return Err(()),
        Ok(bytes) => match String::from_utf8(bytes) {
            Err(_) => return Err(()),
            Ok(content) => content,
        },
    };

    conf_content.push_str("\n[Peer]\n");
    conf_content.push_str(&format!("PublicKey = {}\n", public_key));
    conf_content.push_str(&format!("AllowedIPs = {}\n", allowed_ip));

    command_base
        .arg("set")
        .arg(interface)
        .arg("peer")
        .arg(public_key)
        .arg("allowed-ips")
        .arg(allowed_ip);

    if let Some(resolved_endpoint) = endpoint {
        command_base.arg("endpoint").arg(resolved_endpoint);

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
        Err(_) => return Err(()),
    };

    if !exit_status.success() {
        return Err(());
    }

    Ok(())
}

pub fn install_service_if_not_already(interface_name: &str) -> Result<(), ()> {
    let service_installed: bool;

    #[cfg(target_os = "windows")]
    {
        let output = match Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args([
                "-NoProfile",
                "-Command",
                "Get-Service 'WireGuardTunnel$*' | Select-Object -ExpandProperty Name",
            ])
            .output()
        {
            Ok(res) => res,
            Err(_) => return Err(()),
        };

        let names = match String::from_utf8(output.stdout) {
            Ok(out) => out,
            Err(_) => return Err(()),
        };

        service_installed = names.contains(&format!("WireGuardTunnel${}", interface_name));
    }

    #[cfg(target_os = "linux")]
    {
        let output = match Command::new("sudo")
            .args(["ls", "/etc/wireguard/*.conf"])
            .output()
        {
            Ok(res) => res,
            Err(_) => return Err(()),
        };

        let names = match String::from_utf8(output.stdout) {
            Ok(out) => out,
            Err(_) => return Err(()),
        };

        service_installed = names.contains(&format!("/etc/wireguard/{}.conf", interface_name));
    }

    if service_installed {
        return Ok(());
    }

    return install_service(interface_name);
}

pub fn install_service(interface_name: &str) -> Result<(), ()> {
    let conf_path = wireguard_path().join(format!("{}.conf", interface_name));
    let conf_path_str = match conf_path.to_str() {
        Some(value) => value,
        None => return Err(()),
    };

    #[cfg(target_os = "windows")]
    let status = Command::new(r"C:\Program Files\WireGuard\wireguard.exe")
        .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
        .arg("/installtunnelservice")
        .arg(format!("{}", conf_path_str))
        .status();

    #[cfg(target_os = "linux")]
    let status = Command::new(r"sudo")
        .arg("cp")
        .arg(format!("{}", conf_path_str))
        .arg(format!(
            "/etc/wireguard/{}",
            conf_path.file_name().and_then(|n| n.to_str()).unwrap()
        ))
        .status();

    let exit = match status {
        Ok(exit) => exit,
        Err(_) => return Err(()),
    };

    if !exit.success() {
        return Err(());
    }

    Ok(())
}

pub fn activate_service(service_name: &str) -> Result<(), ()> {
    deactivate_running_service()?;

    #[cfg(target_os = "windows")]
    {
        match Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args([
                "-NoProfile",
                "-Command",
                &format!("Start-Service 'WireGuardTunnel${}'", service_name),
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

    #[cfg(target_os = "linux")]
    {
        match Command::new("sudo")
            .args([
                "systemctl",
                "start",
                &format!("wg-quick@{}.service", service_name),
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

pub fn get_active_service() -> Option<String> {
    #[cfg(target_os = "windows")]
    let output = match Command::new(r"C:\Program Files\WireGuard\wg.exe")
        .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
        .arg("show")
        .output()
    {
        Ok(out) => out,
        Err(_) => return None,
    };

    #[cfg(target_os = "linux")]
    let output = match Command::new(r"sudo").arg("wg").arg("show").output() {
        Ok(out) => out,
        Err(_) => return None,
    };

    let out_string = match String::from_utf8(output.stdout) {
        Ok(str) => str,
        Err(_) => return None,
    };

    if out_string.is_empty() {
        return None;
    }

    // Parse this output:
    //
    // interface: name
    //   public key: ...=
    //   private key: (hidden)
    //   ...
    let active_service_name = out_string
        .split("\n")
        .next()
        .unwrap()
        .split(" ")
        .nth(1)
        .unwrap()
        .trim()
        .to_owned();

    Some(active_service_name)
}

pub fn deactivate_running_service() -> Result<(), ()> {
    let active_service_name = match get_active_service() {
        Some(value) => value,
        None => return Ok(()), // Assume no service running
    };

    #[cfg(target_os = "windows")]
    {
        match Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args([
                "-NoProfile",
                "-Command",
                &format!("Stop-Service 'WireGuardTunnel${}'", active_service_name),
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

    #[cfg(target_os = "linux")]
    {
        match Command::new("sudo")
            .args([
                "systemctl",
                "stop",
                &format!("wg-quick@{}.service", active_service_name),
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
