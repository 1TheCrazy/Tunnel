use sha2::{Digest, Sha256};

pub fn parse_sha256_fingerprint(
    fingerprint: &String,
) -> Result<[u8; 32], String> {
    let normalized: String = fingerprint
        .chars()
        .filter(|character| {
            !matches!(character, ':' | '-' | ' ')
        })
        .collect();

    let bytes = hex::decode(&normalized)
        .map_err(|error| {
            format!("Invalid fingerprint: {error}")
        })?;

    bytes.try_into().map_err(|bytes: Vec<u8>| {
        format!(
            "SHA-256 fingerprint must be 32 bytes, got {}",
            bytes.len()
        )
    })
}

pub fn get_tls_fingerprint(bytes: &[u8]) -> String {
    let sha256: [u8; 32] = Sha256::digest(bytes).into();

    let formatted = sha256
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":");

    return formatted;
}