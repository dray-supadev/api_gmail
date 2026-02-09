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
    pub cc: Option<Vec<String>>,
    pub subject: String,
    pub thread_id: Option<String>,
    pub comment: Option<String>,
    pub pdf_export_settings: Option<Vec<String>>,
    pub html_body: Option<String>,
    pub pdf_base64: Option<String>,
    pub pdf_name: Option<String>,
    pub maildata_identificator: Option<String>,
}

pub async fn send_quote_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SendQuoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = get_token(&headers)?;
    
    // 1. Setup Services
    let bubble_service = BubbleService::new(state.client.clone())?;
    // 2. Fetch/Generate PDF (either from provided base64 or via Bubble Workflow)
    let (pdf_bytes, filename) = if let (Some(base64_content), Some(name)) = (req.pdf_base64, req.pdf_name) {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let bytes = STANDARD.decode(base64_content.trim_start_matches("data:application/pdf;base64,"))
            .map_err(|e| AppError::BadRequest(format!("Invalid PDF base64: {}", e)))?;
        (bytes, name)
    } else {
        bubble_service.generate_pdf_via_workflow(
            &req.quote_id, 
            req.version.as_deref(), 
            req.pdf_export_settings.clone()
        ).await?
    };
    
    // 3. Get HTML Body from Bubble via send_quote workflow
    // The user requested: calls send_quote with quote, pdf, recipients, cc, pdfname, subject, maildata_Identificator
    // And gets HTML back.
    let html_body = bubble_service.send_quote(
        req.version.as_deref(),
        &req.quote_id,
        pdf_bytes.clone(),
        &filename,
        req.to.clone(),
        req.cc.clone().unwrap_or_default(),
        &req.subject,
        req.maildata_identificator.as_deref().unwrap_or(""),
    ).await?;
    
    // 4. Attach PDF
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
        cc: req.cc,
        subject: req.subject,
        body: html_body, 
        thread_id: req.thread_id,
        attachments, 
    };
    
    let result = provider_instance.send_message(token, send_req).await?;
    
    Ok(Json(result).into_response())
}

pub async fn get_embed_js(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let path = "frontend/dist/embed.js";
    let js = match std::fs::read_to_string(path) {
        Ok(content) => content.replace("__API_KEY_PLACEHOLDER__", &state.config.app_secret_key),
        Err(e) => {
            tracing::error!("Failed to read embed.js: {:?}", e);
            "console.error('Widget script not found on server');".to_string()
        }
    };

    axum::response::Response::builder()
        .header("Content-Type", "application/javascript")
        .header("Cache-Control", "no-cache") 
        .body(axum::body::Body::from(js))
        .unwrap()
}
