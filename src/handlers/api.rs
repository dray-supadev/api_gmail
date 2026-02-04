use axum::{
    extract::{Path, Query, Json},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use crate::error::AppError;
use super::provider::{EmailProvider, ListParams, SendMessageRequest};
use super::gmail::GmailProvider;
use super::outlook::OutlookProvider;

#[derive(Deserialize)]
pub struct ProviderParams {
    pub provider: Option<String>,
}

fn get_token(headers: &HeaderMap) -> Result<&str, AppError> {
    headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|t| t.trim_start_matches("Bearer "))
        .or_else(|| {
             // Fallback for existing Gmail integration using x-google-token
             headers.get("x-google-token").and_then(|h| h.to_str().ok())
        })
        .map(|t| t.trim())
        .ok_or(AppError::MissingToken)
}

fn get_provider(params: &ProviderParams) -> Box<dyn EmailProvider> {
    match params.provider.as_deref() {
        Some("outlook") | Some("microsoft") => Box::new(OutlookProvider::new()),
        _ => Box::new(GmailProvider::new()), // Default to Gmail
    }
}

pub async fn list_messages(
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
    Query(list_params): Query<ListParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result = provider.list_messages(token, list_params).await?;
    Ok(Json(result).into_response())
}

pub async fn get_message(
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result = provider.get_message(token, &id).await?;
    Ok(Json(result).into_response())
}

pub async fn get_thread(
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result = provider.get_thread(token, &id).await?;
    Ok(Json(result).into_response())
}

pub async fn send_message(
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result = provider.send_message(token, payload).await?;
    Ok(Json(result).into_response())
}
