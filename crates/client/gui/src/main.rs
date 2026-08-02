#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod http;
mod state;

fn main() {
    #[cfg(target_os = "macos")]
    panic!("MacOS does is currently not supported by Tunnel::Node");
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_servers,
            commands::connection_active,
            commands::set_connection_icon,
            commands::network_stats,
            commands::list_nodes,
            commands::refresh,
            commands::refresh_node_locations,
            commands::server_add,
            commands::server_remove,
            commands::server_set,
            commands::connect,
            commands::disconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
