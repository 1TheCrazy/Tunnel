use tunnel_core::wireguard::client::deactivate_running_service;

use crate::write_line;


pub fn disconnect() -> Result<(), ()> {

    match deactivate_running_service() {
        Ok(()) => return Ok(()),
        Err(_) => {
            write_line!("Wasn't able to end the running service...");
        }
    }

    Err(())
}