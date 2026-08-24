use crate::{
    state::io_manager::{ensure_parent_dir, wireguard_path},
    wireguard::{
        common::{activate_service, install_service_if_not_already},
        util::interface_name_from_node_id,
    },
};
use std::{fs, /*net::Ipv4Addr,*/ process::Command};

#[cfg(target_os = "windows")]
use crate::util::terminal::WINDOWS_INVISIBLE_TERMIAL;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
    // When client and node are on the same network and the latest windows update, this kills the pc
    // You'd have to pass the local address 192.168.x.x , but I'm not doing this
    conf.push_str(&format!("Endpoint = {}\n", endpoint));
    // TODO: sync full-/split-tunnel with config
    // TODO: Add ipv6 support
    // On Windows, a literal /0 enables WireGuard's kill-switch firewall rules.
    // Two /1 routes provide the same IPv4 full-tunnel coverage without that
    // special routing/firewall path, which also keeps the peer endpoint usable.
    conf.push_str("AllowedIPs = 0.0.0.0/1, 128.0.0.0/1\n");
    conf.push_str("PersistentKeepalive = 25\n");

    match fs::write(path, conf) {
        Err(_) => return Err(()),
        Ok(()) => {}
    };

    activate_client_connection(&interface_name, node_public_key, endpoint)
}

pub fn update_and_activate_client_conf(
    node_id: &str,
    node_public_key: &str,
    endpoint: &str,
) -> Result<(), ()> {
    let interface_name = interface_name_from_node_id(node_id);
    let path = wireguard_path().join(format!("{}.conf", interface_name));
    let conf = fs::read_to_string(&path).map_err(|_| ())?;
    let updated_conf = replace_peer_endpoint(&conf, node_public_key, endpoint).ok_or(())?;
    fs::write(path, updated_conf).map_err(|_| ())?;

    activate_client_connection(&interface_name, node_public_key, endpoint)
}

fn activate_client_connection(
    interface_name: &str,
    node_public_key: &str,
    endpoint: &str,
) -> Result<(), ()> {
    //let node_ip = endpoint.rsplit_once(':').map(|(ip, _)| ip).ok_or(())?;
    //install_endpoint_bypass_route(node_ip)?; // Remove when Windows/WireGuard fixes this (never remove)
    install_service_if_not_already(interface_name)?;
    activate_service(interface_name)?;
    set_active_peer_endpoint(interface_name, node_public_key, endpoint)
}

fn replace_peer_endpoint(conf: &str, node_public_key: &str, endpoint: &str) -> Option<String> {
    let mut result = String::with_capacity(conf.len() + endpoint.len());
    let mut in_target_peer = false;
    let mut replaced = false;

    for line in conf.lines() {
        if line.trim() == "[Peer]" {
            in_target_peer = false;
        } else if line.trim() == format!("PublicKey = {node_public_key}") {
            in_target_peer = true;
        }

        if in_target_peer && line.trim_start().starts_with("Endpoint =") {
            result.push_str(&format!("Endpoint = {endpoint}\n"));
            replaced = true;
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    replaced.then_some(result)
}

fn set_active_peer_endpoint(
    interface_name: &str,
    node_public_key: &str,
    endpoint: &str,
) -> Result<(), ()> {
    #[cfg(target_os = "windows")]
    let mut command = Command::new(r"C:\Program Files\WireGuard\wg.exe");
    #[cfg(target_os = "windows")]
    command.creation_flags(WINDOWS_INVISIBLE_TERMIAL);

    #[cfg(target_os = "linux")]
    let mut command = {
        let mut command = Command::new("sudo");
        command.arg("wg");
        command
    };

    command
        .args([
            "set",
            interface_name,
            "peer",
            node_public_key,
            "endpoint",
            endpoint,
        ])
        .status()
        .map_err(|_| ())?
        .success()
        .then_some(())
        .ok_or(())
}

/*
// Ensures packets to the WireGuard peer use the pre-tunnel default route.
// The route only lives in Windows' ActiveStore and is recreated before every connection. 
fn install_endpoint_bypass_route(node_ip: &str) -> Result<(), ()> {
    let node_ip: Ipv4Addr = node_ip.parse().map_err(|_| ())?;

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "$destination = '{node_ip}/32'; \
             $defaultRoute = Get-NetRoute -AddressFamily IPv4 -DestinationPrefix '0.0.0.0/0' | Sort-Object @{{ Expression = {{ $_.RouteMetric + $_.InterfaceMetric }} }} | Select-Object -First 1; \
             if ($null -eq $defaultRoute) {{ exit 1 }}; \
             $existingRoute = Get-NetRoute -AddressFamily IPv4 -DestinationPrefix $destination -ErrorAction SilentlyContinue | Where-Object {{ $_.InterfaceIndex -eq $defaultRoute.InterfaceIndex -and $_.NextHop -eq $defaultRoute.NextHop }} | Select-Object -First 1; \
             if ($null -eq $existingRoute) {{ New-NetRoute -DestinationPrefix $destination -InterfaceIndex $defaultRoute.InterfaceIndex -NextHop $defaultRoute.NextHop -RouteMetric 1 -PolicyStore ActiveStore -ErrorAction Stop | Out-Null }}"
        );
        let status = Command::new("powershell")
            .creation_flags(WINDOWS_INVISIBLE_TERMIAL)
            .args(["-NoProfile", "-Command", &script])
            .status()
            .map_err(|_| ())?;
        return status.success().then_some(()).ok_or(());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = node_ip;
        Ok(())
    }
}
    */
