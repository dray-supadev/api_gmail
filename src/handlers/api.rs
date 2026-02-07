use axum::{
    extract::{Path, Query, Json},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use crate::error::AppError;
use crate::state::AppState;
use super::provider::{EmailProvider, ListParams, SendMessageRequest, BatchModifyRequest};
use super::gmail::GmailProvider;
use super::outlook::OutlookProvider;
use crate::services::bubble::BubbleService;
use html_escape::encode_safe;
use axum::extract::State;

#[derive(Deserialize)]
pub struct ProviderParams {
    pub provider: Option<String>,
}

fn get_token(headers: &HeaderMap) -> Result<&str, AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .map(|t| t.trim_start_matches("Bearer "))
        .or_else(|| {
             // Fallback for existing Gmail integration using x-google-token
             headers.get("x-google-token").and_then(|h| h.to_str().ok())
        })
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .ok_or(AppError::MissingToken)?;
    
    Ok(token)
}

fn get_provider(params: &ProviderParams, client: reqwest::Client) -> Box<dyn EmailProvider> {
    match params.provider.as_deref() {
        Some("outlook") | Some("microsoft") => Box::new(OutlookProvider::new(client)),
        _ => Box::new(GmailProvider::new(client)), // Default to Gmail
    }
}

pub async fn list_messages(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
    Query(list_params): Query<ListParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params, state.client.clone());
    
    let result: serde_json::Value = provider.list_messages(token, list_params).await?;
    Ok(Json(result).into_response())
}

pub async fn get_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params, state.client.clone());
    
    let result: super::provider::CleanMessage = provider.get_message(token, &id).await?;
    Ok(Json(result).into_response())
}

pub async fn send_message(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params, state.client.clone());
    
    let result: serde_json::Value = provider.send_message(token, payload).await?;
    Ok(Json(result).into_response())
}

pub async fn list_labels(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params, state.client.clone());
    
    let result = provider.list_labels(token).await?;
    Ok(Json(result).into_response())
}

pub async fn batch_modify_labels(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
    Json(payload): Json<BatchModifyRequest>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params, state.client.clone());
    
    provider.batch_modify_labels(token, payload).await?;
    Ok(Json(json!({"status": "ok"})).into_response())
}

// --- Quote Endpoints ---

#[derive(Deserialize)]
pub struct QuotePreviewParams {
    pub quote_id: String,
    pub version: Option<String>,
    pub comment: Option<String>,
    pub pdf_export_settings: Option<Vec<String>>,
}

pub async fn preview_quote(
    State(state): State<AppState>,
    Json(params): Json<QuotePreviewParams>,
) -> Result<impl IntoResponse, AppError> {
    let bubble_service = BubbleService::new(state.client.clone())?;
    // Old logic: fetch raw data + local HTML generation
    // New logic: fetch HTML + Body from Bubble Workflow
    
    let (html, body) = bubble_service.fetch_quote_preview(
        &params.quote_id, 
        params.version.as_deref(), 
        params.pdf_export_settings
    ).await?;
    
    Ok(Json(json!({ 
        "html": html,
        "body": body 
    })).into_response())
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
    pub pdf_export_settings: Option<Vec<String>>,
    pub html_body: Option<String>,
}

pub async fn send_quote_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SendQuoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = get_token(&headers)?;
    
    // 1. Setup Services
    let bubble_service = BubbleService::new(state.client.clone())?;
    // 2. Fetch/Generate PDF via Bubble Workflow
    let (pdf_bytes, filename) = bubble_service.generate_pdf_via_workflow(
        &req.quote_id, 
        req.version.as_deref(), 
        req.pdf_export_settings.clone()
    ).await?;
    
    // 3. Determine HTML Body
    // If frontend passed the preview HTML, use it. Otherwise fall back to simple message.
    let html_body = if let Some(body) = req.html_body {
        body
    } else if let Some(comment) = &req.comment {
        // Fix Point 1: XSS protection
        let escaped_comment = encode_safe(comment).replace("\n", "<br>");
        format!("<p>{}</p>", escaped_comment)
    } else {
        "<p>Please find the attached quote proposal.</p>".to_string()
    };
    
    // 4. Attach PDF (Fix Point 9)
    let attachments = Some(vec![super::provider::Attachment {
        filename,
        content: pdf_bytes,
        mime_type: "application/pdf".to_string(),
    }]);

    // 5. Select Provider
    let provider_instance: Box<dyn EmailProvider> = match req.provider.as_str() {
        "gmail" => Box::new(GmailProvider::new(state.client.clone())),
        "outlook" => Box::new(OutlookProvider::new(state.client.clone())),
        _ => return Err(AppError::BadRequest("Invalid provider. Use 'gmail' or 'outlook'".to_string())),
    };
    
    let send_req = SendMessageRequest {
        to: req.to,
        subject: req.subject,
        body: html_body, 
        thread_id: req.thread_id,
        attachments, 
    };
    
    let result = provider_instance.send_message(token, send_req).await?;
    
    Ok(Json(result).into_response())
}
