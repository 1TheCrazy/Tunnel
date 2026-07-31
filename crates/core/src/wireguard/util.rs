use std::process::Command;

pub fn interface_name_from_node_id(node_id: &str) -> String {
    // Strip to 10 hex digits for linux service name restrictions
    let interface_id: String = node_id.chars().take(10).collect();
    
    format!("t_{}", interface_id)
}

pub fn get_internet_interface_name() -> Result<String, ()> {
    #[cfg(target_os = "windows")]
    panic!("This action is not supported on Windows");
    
    #[allow(unreachable_code)] // Reachable, but I compile on windows
    let output = Command::new("ip")
        .args([
            "route", 
            "get", 
            "1.1.1.1"
        ])
        .output()
        .map_err(|_| ())?;

    if !output.status.success() {
        return Err(())
    }

    let output = String::from_utf8(output.stdout).map_err(|_| ())?;

    let mut parts = output.split_whitespace();

    let internet_interface = parts
        .find(|part| *part == "dev")
        .and_then(|_| parts.next())
        .map(str::to_owned)
        .ok_or(())?;

    Ok(internet_interface)
}