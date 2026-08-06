use std::sync::Arc;

use rustls::ClientConfig;

use crate::{net::fingerprint_verifier::FingerprintVerifier, util::crypto::parse_sha256_fingerprint};

pub fn create_pinned_client(
    fingerprint: &Option<String>,
    server_name: &str,
    initial_fingerprint_callback: Box<dyn Fn(String) + Send + Sync + 'static>,
) -> Result<reqwest::Client, Box<dyn std::error::Error>> {
    let mut tls_config = get_pinned_tls_config(fingerprint, server_name, initial_fingerprint_callback)?;

    // Required HTTP protocol negotiation.
    tls_config.alpn_protocols = vec![
        b"h2".to_vec(),
        b"http/1.1".to_vec(),
    ];

    let client = reqwest::Client::builder()
        .use_preconfigured_tls(tls_config)
        .https_only(true)
        .build()?;

    Ok(client)
}

pub fn get_pinned_tls_config(fingerprint: &Option<String>, server_name: &str, initial_fingerprint_callback: Box<dyn Fn(String) + Send + Sync + 'static>) -> Result<rustls::ClientConfig, Box<dyn std::error::Error>> {
    let expected_sha256 =
        fingerprint
        .as_ref()
        .map(parse_sha256_fingerprint)
        .transpose()
        .map_err(
            |message| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    message,
                )
            }
        )?;
    
    let provider = Arc::new(
        rustls::crypto::ring::default_provider(),
    );

    let verifier = FingerprintVerifier {
        expected_sha256: expected_sha256,
        server_name: server_name.to_owned(),
        blindly_trusted_fingerprint_recieved: initial_fingerprint_callback,
        provider: Arc::clone(&provider)
    };

    let tls_config =
        ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(
                verifier,
            ))
            .with_no_client_auth();

    Ok(tls_config)
}

pub fn get_fingerprint_option(fingerprint: &str) -> Option<String>{
    if fingerprint.is_empty() {
        return None
    } else {
        return Some(String::from(fingerprint));
    }
}