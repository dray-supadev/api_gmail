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
use crate::services::bubble::BubbleService;
use crate::services::n8n::N8NService;

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
    
    let result: serde_json::Value = provider.list_messages(token, list_params).await?;
    Ok(Json(result).into_response())
}

pub async fn get_message(
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result: super::provider::CleanMessage = provider.get_message(token, &id).await?;
    Ok(Json(result).into_response())
}

pub async fn get_thread(
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result: serde_json::Value = provider.get_thread(token, &id).await?;
    Ok(Json(result).into_response())
}

pub async fn send_message(
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params);
    
    let result: serde_json::Value = provider.send_message(token, payload).await?;
    Ok(Json(result).into_response())
}

// --- Quote Endpoints ---

#[derive(Deserialize)]
pub struct QuotePreviewParams {
    pub quote_id: String,
    pub version: Option<String>,
    pub comment: Option<String>,
}

pub async fn preview_quote(
    Query(params): Query<QuotePreviewParams>,
) -> Result<impl IntoResponse, AppError> {
    let bubble_service = BubbleService::new()?;
    
    // Fetch data
    let quote_data = bubble_service.fetch_quote(params.version.as_deref(), &params.quote_id).await?;
    
    // Generate HTML
    let html = bubble_service.generate_quote_html(&quote_data, params.comment.as_deref());
    
    Ok(Json(json!({ "html": html })).into_response())
}

#[derive(Deserialize)]
pub struct SendQuoteRequest {
    pub quote_id: String,
    pub version: Option<String>,
    pub provider: String, // "gmail" or "outlook"
    pub to: Vec<String>,
    pub subject: String,
    pub thread_id: Option<String>,
    pub comment: Option<String>,
}

pub async fn send_quote_email(
    headers: HeaderMap,
    Json(req): Json<SendQuoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = get_token(&headers)?;
    
    // 1. Setup Services
    let bubble_service = BubbleService::new()?;
    let n8n_service = N8NService::new();
    
    // 2. Fetch Quote Data
    let quote_data = bubble_service.fetch_quote(req.version.as_deref(), &req.quote_id).await?;
    
    // 3. Generate HTML
    let html_body = bubble_service.generate_quote_html(&quote_data, req.comment.as_deref());
    
    // 4. Generate PDF
    let pdf_bytes = n8n_service.generate_pdf(&html_body).await?;
    
    // 5. Select Provider
    let provider_instance: Box<dyn EmailProvider> = match req.provider.as_str() {
        "gmail" => Box::new(GmailProvider::new()),
        "outlook" => Box::new(OutlookProvider::new()),
        _ => return Err(AppError::BadRequest("Invalid provider. Use 'gmail' or 'outlook'".to_string())),
    };
    
    // 6. Send Email
    let attachment = super::provider::Attachment {
        filename: "Quote.pdf".to_string(), 
        content: pdf_bytes,
        mime_type: "application/pdf".to_string(),
    };
    
    let send_req = SendMessageRequest {
        to: req.to,
        subject: req.subject,
        body: html_body, 
        thread_id: req.thread_id,
        attachments: Some(vec![attachment]),
    };
    
    let result = provider_instance.send_message(token, send_req).await?;
    
    Ok(Json(result).into_response())
}
