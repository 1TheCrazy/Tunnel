use std::{collections::HashMap, process::Command};

use serde::Serialize;
use tauri::Manager;
use tunnel_core::{
    structs::client::{ClientNode, ClientServer},
    wireguard::{
        client::create_and_activate_client_conf,
        common::{activate_service, deactivate_running_service, get_active_service},
        util::interface_name_from_node_id,
    },
};

use crate::{
    http::{discover_node, get_node_locations, get_nodes},
    state::{
        get_active_server, get_mut_active_server, get_mut_save,
        get_node_locations as get_saved_node_locations, get_ref_save, write_node_locations,
    },
};

#[derive(Serialize)]
pub struct ServerRow {
    pub name: String,
    pub host: String,
    pub nodes: usize,
    pub active: bool,
}

#[derive(Serialize)]
pub struct NodeRow {
    pub id: String,
    pub ip: String,
    pub discovered: bool,
    pub connected: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize)]
pub struct NetworkStats {
    pub interface: String,
    pub received_bytes: u64,
    pub sent_bytes: u64,
}

#[tauri::command]
pub fn list_servers() -> Vec<ServerRow> {
    let state = get_ref_save();
    state
        .servers
        .iter()
        .enumerate()
        .map(|(index, server)| ServerRow {
            name: server.name.to_owned(),
            host: server.host.to_owned(),
            nodes: server.nodes.len(),
            active: index as i32 == state.active_server_index,
        })
        .collect()
}

#[tauri::command]
pub fn connection_active() -> bool {
    get_active_service().is_some()
}

#[tauri::command]
pub fn set_connection_icon(app: tauri::AppHandle, connected: bool) -> Result<(), String> {
    let icon_bytes = if connected {
        include_bytes!("../icons/icon_connected.ico")
    } else {
        include_bytes!("../icons/icon.ico")
    };
    let icon = tauri::image::Image::from_bytes(icon_bytes)
        .map_err(|error| format!("Couldn't load the application icon: {error}"))?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Couldn't find the main application window".to_owned())?;

    window
        .set_icon(icon)
        .map_err(|error| format!("Couldn't update the application icon: {error}"))
}

#[tauri::command]
pub fn network_stats() -> Result<Option<NetworkStats>, String> {
    let interface = match get_active_service() {
        Some(interface) => interface,
        None => return Ok(None),
    };

    #[cfg(target_os = "windows")]
    let command = Command::new(r"C:\Program Files\WireGuard\wg.exe")
        .arg("show")
        .output();
    #[cfg(not(target_os = "windows"))]
    let command = Command::new("sudo").args(["wg", "show"]).output();

    let output = command.map_err(|error| format!("Couldn't run wg show: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "wg show failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }

    parse_network_stats(&String::from_utf8_lossy(&output.stdout), interface).map(Some)
}

fn parse_network_stats(output: &str, interface: String) -> Result<NetworkStats, String> {
    let mut received_bytes = 0;
    let mut sent_bytes = 0;
    let mut found_transfer = false;

    for line in output.lines().map(str::trim) {
        let Some(transfer) = line.strip_prefix("transfer:") else {
            continue;
        };
        let (received, sent) = transfer
            .trim()
            .split_once(',')
            .ok_or_else(|| "Couldn't parse wg transfer counters".to_owned())?;
        let received = received
            .trim()
            .strip_suffix(" received")
            .ok_or_else(|| "Couldn't parse received counter".to_owned())?;
        let sent = sent
            .trim()
            .strip_suffix(" sent")
            .ok_or_else(|| "Couldn't parse sent counter".to_owned())?;

        received_bytes += parse_wireguard_bytes(received)?;
        sent_bytes += parse_wireguard_bytes(sent)?;
        found_transfer = true;
    }

    if !found_transfer {
        return Err("wg show did not include transfer counters".to_owned());
    }

    Ok(NetworkStats {
        interface,
        received_bytes,
        sent_bytes,
    })
}

fn parse_wireguard_bytes(value: &str) -> Result<u64, String> {
    let mut parts = value.split_whitespace();
    let amount: f64 = parts
        .next()
        .ok_or_else(|| "Missing transfer amount".to_owned())?
        .parse()
        .map_err(|_| "Invalid transfer amount".to_owned())?;
    let unit = parts
        .next()
        .ok_or_else(|| "Missing transfer unit".to_owned())?;
    let multiplier = match unit {
        "B" => 1,
        "KiB" => 1024,
        "MiB" => 1024 * 1024,
        "GiB" => 1024 * 1024 * 1024,
        "TiB" => 1024_u64.pow(4),
        _ => return Err(format!("Unsupported transfer unit '{unit}'")),
    };

    Ok((amount * multiplier as f64).round() as u64)
}

#[tauri::command]
pub fn list_nodes() -> Result<Vec<NodeRow>, String> {
    let state = get_ref_save();
    let locations = get_saved_node_locations();
    let active_service = get_active_service();
    let server = get_active_server(&state).ok_or_else(|| {
        "There was no selected server. To list nodes, select a server using 'server set <NAME>' or see '--help'".to_owned()
    })?;

    Ok(server
        .nodes
        .iter()
        .map(|node| NodeRow {
            id: node.id.to_owned(),
            ip: node.ip.to_owned(),
            discovered: node.discovered,
            connected: active_service
                .as_deref()
                .is_some_and(|service_name| service_name == interface_name_from_node_id(&node.id)),
            latitude: locations
                .locations
                .get(&node.id)
                .map(|location| location.latitude),
            longitude: locations
                .locations
                .get(&node.id)
                .map(|location| location.longitude),
        })
        .collect())
}

#[tauri::command]
pub async fn refresh_node_locations() -> Result<(), String> {
    let state = get_ref_save();
    let mut node_ids_by_ip = HashMap::new();

    for node in state.servers.iter().flat_map(|server| &server.nodes) {
        node_ids_by_ip
            .entry(node.ip.clone())
            .or_insert_with(Vec::new)
            .push(node.id.clone());
    }

    let ips: Vec<_> = node_ids_by_ip.keys().cloned().collect();
    let mut locations = get_saved_node_locations();

    for batch in ips.chunks(100) {
        let resolved = get_node_locations(batch)
            .await
            .map_err(|err| format!("Couldn't refresh node locations: {}", err))?;

        for (ip, location) in resolved {
            if let Some(node_ids) = node_ids_by_ip.get(&ip) {
                for node_id in node_ids {
                    locations
                        .locations
                        .insert(node_id.clone(), location.clone());
                }
            }
        }
    }

    write_node_locations(&locations).map_err(|err| format!("Couldn't save node locations: {}", err))
}

#[tauri::command]
pub async fn refresh() -> Result<(), String> {
    let mut state = get_mut_save();
    let server = get_mut_active_server(&mut state).ok_or_else(|| {
        "There was no selected server. To refresh nodes, select a server 'server set <NAME>' or see '--help'".to_owned()
    })?;

    let nodes = get_nodes(&server.host, &server.password)
        .await
        .map_err(|err| format!("Encountered an error while refreshing nodes: \n{}", err))?;

    let mut existing_nodes: HashMap<_, _> = std::mem::take(&mut server.nodes)
        .into_iter()
        .map(|node| (node.id.clone(), node))
        .collect();

    server.nodes = nodes
        .into_iter()
        .map(|node| match existing_nodes.remove(&node.id) {
            Some(existing_node) => ClientNode {
                ip: node.ip,
                id: existing_node.id,
                port: node.port,
                public_key: existing_node.public_key,
                discovered: existing_node.discovered,
            },
            None => node,
        })
        .collect();

    Ok(())
}

#[tauri::command]
pub async fn server_add(
    name: String,
    host: String,
    password: Option<String>,
) -> Result<(), String> {
    let mut state = get_mut_save();

    if state.servers.iter().any(|server| server.name == name) {
        return Err("Server name already in use. To modify a server see '--help'".to_owned());
    }

    if let Some(server) = state.servers.iter().find(|server| server.host == host) {
        return Err(format!(
            "A server with the same host is already present ('{}')",
            server.name
        ));
    }

    let pw = password.unwrap_or_default();
    let node_list = get_nodes(&host, &pw)
        .await
        .map_err(|err| format!("Wasn't able to add the server: \n{}", err))?;

    state.servers.push(ClientServer {
        host,
        name,
        password: pw,
        nodes: node_list,
    });

    if state.active_server_index == -1 {
        state.active_server_index = state.servers.len() as i32 - 1;
    }

    Ok(())
}

#[tauri::command]
pub fn server_remove(name: String) -> Result<(), String> {
    let mut state = get_mut_save();
    let target_server_index = state
        .servers
        .iter()
        .position(|s| s.name == name)
        .ok_or_else(|| format!("There was no server with the name '{}'", name))?;

    state.servers.remove(target_server_index);

    if target_server_index as i32 == state.active_server_index {
        state.active_server_index = -1;
    }

    if target_server_index as i32 <= state.active_server_index {
        state.active_server_index -= 1;
    }

    Ok(())
}

#[tauri::command]
pub fn server_set(name: String) -> Result<(), String> {
    let mut state = get_mut_save();
    let target_server_index = state
        .servers
        .iter()
        .position(|s| s.name == name)
        .ok_or_else(|| format!("There was no server with the name '{}'", name))?;

    state.active_server_index = target_server_index as i32;
    Ok(())
}

#[tauri::command]
pub async fn connect(id: String) -> Result<(), String> {
    let mut state = get_mut_save();
    let server = get_mut_active_server(&mut state).ok_or_else(|| {
        "There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help".to_owned()
    })?;

    let discovered = server
        .nodes
        .iter()
        .find(|n| n.id == id)
        .ok_or_else(|| "The selected server didn't contian a node with the given id".to_owned())?
        .discovered;

    drop(state);

    if discovered {
        connect_known(&id)
    } else {
        connect_fresh(&id).await
    }
}

fn connect_known(id: &str) -> Result<(), String> {
    let service_name = interface_name_from_node_id(id);

    activate_service(&service_name).map_err(|_| "Wasn't able to activate service.".to_owned())
}

async fn connect_fresh(id: &str) -> Result<(), String> {
    let mut state = get_mut_save();
    let ref_state = get_ref_save();
    let server = get_mut_active_server(&mut state).ok_or_else(|| {
        "There was no selected server. Select a server before connecting to a node that owns it. See '--help' for help".to_owned()
    })?;

    if !server.nodes.iter().any(|n| n.id == id) {
        return Err("The selected server didn't contian anode with the given id".to_owned());
    }

    let res = discover_node(id, server, &ref_state.public_key)
        .await
        .map_err(|err| format!("Wasn't able to discover node: \n{}", err))?;

    let node = server
        .nodes
        .iter_mut()
        .find(|n| n.id == id)
        .ok_or_else(|| "The selected server didn't contian anode with the given id".to_owned())?;

    node.discovered = true;
    let endpoint = format!("{}:{}", node.ip, node.port);

    create_and_activate_client_conf(
        id,
        &ref_state.private_key,
        &res.assigned_vpn_ip,
        &node.public_key,
        &endpoint,
    )
    .map_err(|_| "Failed to create or start wireguard service".to_owned())
}

#[tauri::command]
pub fn disconnect() -> Result<(), String> {
    deactivate_running_service().map_err(|_| "Wasn't able to end the running service...".to_owned())
}

#[cfg(test)]
mod tests {
    use super::parse_network_stats;

    #[test]
    fn parses_wireguard_transfer_counters() {
        let output = "interface: t_EL_GRWh4F7\npeer: test\n  transfer: 14.00 MiB received, 504.95 KiB sent\n";
        let stats = parse_network_stats(output, "t_EL_GRWh4F7".to_owned()).unwrap();

        assert_eq!(stats.received_bytes, 14 * 1024 * 1024);
        assert_eq!(stats.sent_bytes, (504.95_f64 * 1024.0).round() as u64);
    }
}
