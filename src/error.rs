use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Gmail API error: {0}")]
    GmailApi(#[from] reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Missing Google Token")]
    MissingToken,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::GmailApi(e) => {
                if let Some(reqwest_status) = e.status() {
                    // Convert reqwest::StatusCode (http 0.2) to axum::http::StatusCode (http 1.0)
                    let status_code = StatusCode::from_u16(reqwest_status.as_u16())
                        .unwrap_or(StatusCode::BAD_GATEWAY);
                    
                    if status_code == StatusCode::UNAUTHORIZED {
                        (StatusCode::UNAUTHORIZED, "Invalid or expired Google Token")
                    } else {
                        (status_code, "Gmail API returned an error")
                    }
                } else {
                    (StatusCode::BAD_GATEWAY, "Failed to reach Gmail API")
                }
            },
            AppError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing x-google-token header"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string()
        }));

        (status, body).into_response()
    }
}
