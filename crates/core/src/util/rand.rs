use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;

pub fn get_128_bit_random() -> String{
    let mut bytes = [0u8; 16];
    rand::rng().fill_bytes(&mut bytes);

    URL_SAFE_NO_PAD.encode(bytes)
}