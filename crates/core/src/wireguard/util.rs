pub fn interface_name_from_node_id(node_id: &str) -> String {
    // Strip to 10 hex digits for linux service name restrictions
    let interface_id: String = node_id.chars().take(10).collect();
    
    format!("t_{}", interface_id)
}