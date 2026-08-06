use tunnel_core::{state::io_manager, util::crypto::get_tls_fingerprint};
use rcgen::{generate_simple_self_signed, CertifiedKey};
use std::fs;

pub fn create_pem_files_if_not_already(hostname: &str) -> Result<(), ()>{
    let cert_path = io_manager::TLS_CERT_PATH();
    let key_path = io_manager::TLS_KEY_PATH();
    
    if !(cert_path.exists() && key_path.exists()) {
        return create_pem_files(hostname);
    }

    Ok(())
}

pub fn create_pem_files(hostname: &str) -> Result<(), ()> {
    let cert_path = io_manager::TLS_CERT_PATH();
    let key_path = io_manager::TLS_KEY_PATH();

    if let Some(parent) = cert_path.parent() {
        match fs::create_dir_all(parent) {
            Ok(()) => {},
            Err(_) => return Err(())
        };
    }

    if let Some(parent) = key_path.parent() {
        match fs::create_dir_all(parent) {
            Ok(()) => {},
            Err(_) => return Err(())
        };
    }

    let CertifiedKey { cert, signing_key } = match generate_simple_self_signed(vec![hostname.to_owned()]) {
        Ok(res) => res,
        Err(_) => return Err(())
    };

    let fingerprint = get_tls_fingerprint(cert.der().as_ref());

    println!("server: created TLS certificate with fingerprint: {}", fingerprint);

    match fs::write(&cert_path, cert.pem()) {
        Ok(()) => {},
        Err(_) => return Err(())
    };

    match fs::write(&key_path, signing_key.serialize_pem()) {
        Ok(()) => {},
        Err(_) => return Err(())
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        match fs::set_permissions(
            &key_path,
            fs::Permissions::from_mode(0o600),
        ) {
            Ok(()) => {},
            Err(_) => println!("Failed to restrict cert and key access.")
        };
    }

    println!(
        "server: generated TLS certificate at {}",
        cert_path.display()
    );

    Ok(())
}