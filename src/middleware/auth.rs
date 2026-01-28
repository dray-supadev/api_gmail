use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use crate::config::Config;

pub async fn verify_api_key(
    State(config): State<Config>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());

    match api_key {
        Some(key) if key == config.app_secret_key => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
