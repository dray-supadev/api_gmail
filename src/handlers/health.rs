use axum::{response::IntoResponse, Json};
use serde_json::json;

pub async fn check() -> impl IntoResponse {
    Json(json!({ "status": "healthy" }))
}
