use std::sync::OnceLock;

pub static QUIET: OnceLock<bool> = OnceLock::new();
