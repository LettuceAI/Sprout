use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::{StatusCode, header::AUTHORIZATION},
    middleware::Next,
    response::Response,
};

use crate::config::Config;

pub async fn require_bearer(
    State(config): State<Arc<Config>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !config.require_auth {
        return Ok(next.run(request).await);
    }

    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    match token {
        Some(token) if constant_time_eq(token.as_bytes(), config.api_key.as_bytes()) => {
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}
