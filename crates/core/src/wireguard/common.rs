use wireguard_control::KeyPair;

pub fn gen_key_pair() -> KeyPair {
    KeyPair::generate()
}