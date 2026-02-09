use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{Response, IntoResponse},
};
use crate::state::AppState;

pub async fn verify_api_key(
    State(state): State<AppState>,
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

    let is_valid = match api_key {
        Some(key) => {
            let matches_app_key = key == state.config.app_secret_key;
            let matches_bubble_key = !state.config.bubble_api_token.is_empty() && key == state.config.bubble_api_token;
            let matches_widget_key = !state.config.widget_api_key.is_empty() && key == state.config.widget_api_key;
            matches_app_key || matches_bubble_key || matches_widget_key
        },
        None => false,
    };

    if is_valid {
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Unauthorized access attempt. Provided key: {:?}", api_key);
        let body = serde_json::json!({
            "error": "Invalid or missing x-api-key header",
            "details": "The application secret key is required for this endpoint."
        });
        Ok((StatusCode::UNAUTHORIZED, axum::Json(body)).into_response())
    }
}
