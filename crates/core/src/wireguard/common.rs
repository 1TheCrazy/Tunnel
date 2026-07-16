use base64::{engine::general_purpose::STANDARD, Engine};
use x25519_dalek::{PublicKey, StaticSecret};

pub struct KeyPair {
    pub private: String,
    pub public: String,
}

pub fn gen_key_pair() -> KeyPair {
    let private = StaticSecret::random();
    let public = PublicKey::from(&private);

    KeyPair {
        private: STANDARD.encode(private.to_bytes()),
        public: STANDARD.encode(public.as_bytes()),
    }
}