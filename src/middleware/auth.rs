use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{Response, IntoResponse},
};
use crate::state::AppState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthLevel {
    Admin,
    Widget,
}

pub async fn verify_api_key(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    let api_key = headers
        .get("x-api-key")
        .and_then(|value| value.to_str().ok());

    let auth_level = match api_key {
        Some(key) => {
            if key == state.config.app_secret_key {
                Some(AuthLevel::Admin)
            } else if !state.config.bubble_api_token.is_empty() && key == state.config.bubble_api_token {
                Some(AuthLevel::Admin) // Bubble token is also admin
            } else if !state.config.widget_api_key.is_empty() && key == state.config.widget_api_key {
                Some(AuthLevel::Widget)
            } else {
                None
            }
        },
        None => None,
    };

    if let Some(level) = auth_level {
        request.extensions_mut().insert(level);
        Ok(next.run(request).await)
    } else {
        tracing::warn!("Unauthorized access attempt from path: {}", request.uri().path());
        let body = serde_json::json!({
            "error": "Invalid or missing x-api-key header",
            "details": "The application secret key is required for this endpoint."
        });
        Ok((StatusCode::UNAUTHORIZED, axum::Json(body)).into_response())
    }
}
