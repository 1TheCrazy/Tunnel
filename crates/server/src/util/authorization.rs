use axum::http::{HeaderMap, HeaderValue};

pub fn is_request_authorized(password: &str, headers: &HeaderMap) -> bool {
    password.is_empty() || headers.get("X-Tunnel-Authorization").unwrap_or(&HeaderValue::from_static("")) == password
}