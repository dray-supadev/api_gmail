use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("API error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Gmail API error: {0}")]
    GmailApi(reqwest::Error),
    #[error("Outlook API error: {0}")]
    OutlookApi(reqwest::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Missing Token")]
    MissingToken,
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Bubble API error: {0}")]
    BubbleApi(reqwest::Error),
    #[error("Bad Gateway: {0}")]
    BadGateway(String),
    #[error("Forbidden: {0}")]
    Forbidden(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::Reqwest(ref _e) => (StatusCode::BAD_GATEWAY, "Network or API error"),
            AppError::GmailApi(ref e) => {
                if let Some(reqwest_status) = e.status() {
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
            AppError::OutlookApi(ref e) => {
                if let Some(reqwest_status) = e.status() {
                    let status_code = StatusCode::from_u16(reqwest_status.as_u16())
                        .unwrap_or(StatusCode::BAD_GATEWAY);
                    
                    if status_code == StatusCode::UNAUTHORIZED {
                        (StatusCode::UNAUTHORIZED, "Invalid or expired Microsoft Token")
                    } else {
                        (status_code, "Outlook API returned an error")
                    }
                } else {
                    (StatusCode::BAD_GATEWAY, "Failed to reach Microsoft Graph API")
                }
            },
            AppError::MissingToken => (StatusCode::UNAUTHORIZED, "Missing Authorization header"),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::Config(ref msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::BubbleApi(ref e) => {
                // If the Bubble API returns 401, it means OUR token is wrong/expired
                if let Some(reqwest_status) = e.status() {
                    let status_code = StatusCode::from_u16(reqwest_status.as_u16())
                        .unwrap_or(StatusCode::BAD_GATEWAY);

                    if status_code == StatusCode::UNAUTHORIZED {
                        (StatusCode::INTERNAL_SERVER_ERROR, "Bubble API Token Invalid/Expired")
                    } else if status_code == StatusCode::NOT_FOUND {
                        (StatusCode::NOT_FOUND, "Quote ID not found in Bubble")
                    } else {
                        (StatusCode::BAD_GATEWAY, "Bubble API returned an error")
                    }
                } else {
                    (StatusCode::BAD_GATEWAY, "Failed to reach Bubble API")
                }
            },
            AppError::BadGateway(ref msg) => (StatusCode::BAD_GATEWAY, msg.as_str()),
            AppError::Forbidden(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": error_message,
            "details": self.to_string()
        }));

        (status, body).into_response()
    }
}
