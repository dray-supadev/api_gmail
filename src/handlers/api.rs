use axum::{
    extract::{Path, Query, Json, State},
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
use crate::handlers::postmark::PostmarkProvider;
use crate::services::bubble::BubbleService;

#[derive(Deserialize)]
pub struct ProviderParams {
    pub provider: Option<String>,
    pub company: Option<String>,
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
        .filter(|t| !t.is_empty());

    // Allow empty token for Postmark provider, as it uses server-side token
    // But get_token returns Result<&str>, so we need to handle this caller side or allow it here?
    // Actually, get_provider is called *after* get_token.
    // Making get_token optional or handling Postmark specifically.
    // For now, let's keep get_token strict but maybe pass a dummy token from frontend for Postmark?
    // Or we handle it here: if no token found, check if provider param says postmark?
    // But we don't have access to params here easily without extracting it.
    // Let's stick to requiring a token, frontend can send "postmark-token" or similar dummy.
    // Wait, the user said "Postmark Token - ...". It is server token.
    // So client doesn't have a user token.
    
    if let Some(t) = token {
        Ok(t)
    } else {
        // We defer error to the provider usage if needed, but the current sig returns Result.
        // Let's return a specific error or handle "postmark" exception at call site?
        // Actually, let's allow it to be optional in get_token logic if we change signature?
        // No, that breaks too much.
        // Be pragmatic: Client MUST send a token. For Postmark, client can send "dummy".
        Err(AppError::MissingToken)
    }
}

fn get_provider(params: &ProviderParams, client: reqwest::Client) -> Box<dyn EmailProvider> {
    match params.provider.as_deref() {
        Some("outlook") | Some("microsoft") => Box::new(OutlookProvider::new(client)),
        Some("postmark") => Box::new(PostmarkProvider::new(client, params.company.clone().unwrap_or("Unknown".to_string()))),
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

pub async fn get_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(provider_params): Query<ProviderParams>,
) -> Result<Response, AppError> {
    let token = get_token(&headers)?;
    let provider = get_provider(&provider_params, state.client.clone());
    
    let result = provider.get_profile(token).await?;
    Ok(Json(result).into_response())
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
    pub company: Option<String>,
    pub trigger_reminder: Option<bool>,
}

pub async fn send_quote_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<SendQuoteRequest>,
) -> Result<impl IntoResponse, AppError> {
    let token = get_token(&headers)?;
    
    // 1. Setup Services
    let bubble_service = BubbleService::new(state.client.clone())?;
    // 2. Fetch/Generate PDF (either from provided base64/URL or via Bubble Workflow)
    // We need both bytes (for email attachment) and potentially URL (for Bubble WF)
    let (pdf_bytes, filename, pdf_url_for_bubble) = if let (Some(content), Some(name)) = (req.pdf_base64, req.pdf_name) {
        // Check if it's a URL
        if content.starts_with("http") || content.starts_with("//") {
            let url = if content.starts_with("//") {
                format!("https:{}", content)
            } else {
                content.clone()
            };
            
            // Download bytes for email attachment
            let res = state.client.get(&url).send().await
                .map_err(|e| AppError::BadGateway(format!("Failed to download PDF from URL: {}", e)))?;
                
            if !res.status().is_success() {
                return Err(AppError::BadGateway(format!("Failed to download PDF from URL. Status: {}", res.status())));
            }
            
            let bytes = res.bytes().await
                .map_err(|e| AppError::BadGateway(format!("Failed to read PDF bytes: {}", e)))?
                .to_vec();
                
            // Key change: We pass the ORIGINAL URL to Bubble but keep bytes for email attachment
            // Bubble will take the URL in the 'pdf' field as text
            (bytes, name, Some(content)) 
        } else {
            // Assume Base64
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            // Clean up potentially messy base64 strings (data URI prefix)
            let clean_base64 = if let Some(idx) = content.find(',') {
                &content[idx+1..]
            } else {
                &content
            };
            
            let bytes = STANDARD.decode(clean_base64.trim())
                .map_err(|e| AppError::BadRequest(format!("Invalid PDF base64: {}", e)))?;
            // If we have base64, we don't have a URL.
            // But we can't send text URL if we don't have one.
            (bytes, name, None)
        }
    } else {
        let (bytes, name, url) = bubble_service.generate_pdf_via_workflow(
            &req.quote_id, 
            req.version.as_deref(), 
            req.pdf_export_settings.clone()
        ).await?;
        (bytes, name, Some(url))
    };
    
    // 3. Get HTML Body from Bubble via send_quote workflow
    // Use the URL found or generated. If we generated via workflow, 'pdf_url_for_bubble' is None.
    // BUT the requirement says "pass only text, specifically url".
    // If we only have bytes (from base64), we CANNOT pass a URL to Bubble unless we upload it somewhere first.
    // However, the user said "pass only text, specifically url".
    // This implies that either:
    // a) The client ALWAYS sends a URL (pdf_base64 is actually a URL?) -> The code handles this in line 168.
    // b) If we have bytes, we can't fulfill the requirement.
    // But wait, line 189: (bytes, name, Some(content)). Content IS the URL.
    // Line 210: (bytes, name, None). Here we generated PDF via Buffer workflow.
    // Does 'generate_pdf_via_workflow' return a URL?
    // Looking at 'bubble.rs': 'generate_pdf_via_workflow' returns (Vec<u8>, String). It downloads the PDF bytes.
    // We need it to return the URL too if we want to pass it back to 'send_quote'.
    // Let's assume for now valid usage provides a URL or we need to refactor 'generate_pdf_via_workflow' to return URL.
    // Refactor step needed: Update bubble.rs `generate_pdf_via_workflow` to return (Vec<u8>, String, String) -> (bytes, name, url).
    
    // WAIT. I cannot change bubble.rs again in this single tool call block easily without risking sync issues.
    // I will check if I can modify `generate_pdf_via_workflow` in a separate step or if I should just use the URL if available.
    // If I generated the PDF, I *downloaded* it from a URL. I should return that URL.
    
    // For this specific 'api.rs' edit, I will assume `bubble_service.send_quote` needs a String.
    // I need to get that URL.
    
    // Let's STOP and rethink. I need to modify `api.rs`. I also need `bubble.rs` to return the URL.
    // I already modified `bubble.rs` `send_quote`. I did NOT modify `generate_pdf_via_workflow`.
    // I should probably have modified `generate_pdf_via_workflow` too.
    // I will do that in a subsequent step if needed.
    // For now, in `api.rs`, I will try to pass `pdf_url_for_bubble.unwrap_or_default()`. 
    // If it's empty, and we only have bytes, the Bubble WF will likely fail or receive empty string.
    // This highlights a potential logic gap: if user sends Base64, we have no URL.
    // But the user constraint says "pass only text, specifically url". This implies use cases providing Base64 are either invalid OR Base64 is not used for Bubble, only for Email Attachment?
    // "send_quote" workflow in Bubble sends code to the User. It needs the PDF to include it in the email sent FROM Bubble?
    // If we are sending email from Rust (Gmail/Outlook), maybe we don't need Bubble to send the email?
    // Ah, `bubble_service.send_quote` returns HTML body. It seems it *generates* the email body.
    // Does it *send* the email? The method name is `send_quote`. The endpoint is `wf/send_quote`.
    // But we are using the result `html_body` to send via Gmail/Outlook.
    // So Bubble generates the body. Does the body *contain* the link to PDF?
    // If so, it needs the URL.
    // If we only have Base64, we can't give a URL.
    // I will assume for now we have a URL or the user accepts that Base64 flows might be broken for the "link in body" feature.
    
    // Back to code:
    let pdf_url_to_pass = pdf_url_for_bubble.ok_or_else(|| AppError::BadRequest("PDF URL is required for Bubble template (Base64 not supported for this flow)".to_string()))?;

    let html_body = bubble_service.send_quote(
        &req.quote_id,
        req.version.as_deref(),
        &filename,
        req.to.clone(),
        req.cc.clone().unwrap_or_default(),
        &req.subject,
        req.maildata_identificator.as_deref().unwrap_or(""),
        req.pdf_export_settings.clone().unwrap_or_default(),
        pdf_url_to_pass,
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
        "postmark" => Box::new(PostmarkProvider::new(state.client.clone(), req.company.clone().unwrap_or("Unknown".to_string()))),
        _ => return Err(AppError::BadRequest("Invalid provider. Use 'gmail', 'outlook', or 'postmark'".to_string())),
    };
    
    let send_req = SendMessageRequest {
        to: req.to,
        cc: req.cc,
        subject: req.subject,
        body: html_body, 
        thread_id: req.thread_id,
        attachments, 
    };
    
    let result: serde_json::Value = provider_instance.send_message(token, send_req).await?;
    
    // 6. Trigger reminder on Bubble if requested (only once)
    if req.trigger_reminder.unwrap_or(false) {
        if let Err(e) = bubble_service.send_remember(&req.quote_id, req.version.as_deref()).await {
             tracing::error!("Failed to trigger Bubble reminder: {:?}", e);
             // We don't fail the whole request because the email was already sent
        }
    }

    Ok(Json(result).into_response())
}

#[derive(Deserialize)]
pub struct ReminderWebhookRequest {
    pub content: String,
    pub subject: String,
    pub cc: Option<Vec<String>>,
    pub recipients: Vec<String>,
    pub identificator: Option<String>,
    pub file: String, // URL or base64
    pub file_name: String,
    pub platform: String,
    pub keys: Option<String>, // Token or API Key
    pub company: Option<String>,
}

pub async fn reminder_webhook(
    State(state): State<AppState>,
    Json(req): Json<ReminderWebhookRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Get Token (from keys field or fallback to headers if needed, but keys is priority for webhooks)
    let token = req.keys.as_deref().ok_or_else(|| AppError::BadRequest("API Key (keys) is required for reminder webhook".to_string()))?;

    // 2. Download/Prepare Attachment
    let (file_bytes, filename) = if req.file.starts_with("http") || req.file.starts_with("//") {
        let url = if req.file.starts_with("//") {
            format!("https:{}", req.file)
        } else {
            req.file.clone()
        };
        
        let res = state.client.get(&url).send().await
            .map_err(|e| AppError::BadGateway(format!("Failed to download file from URL: {}", e)))?;
            
        if !res.status().is_success() {
            return Err(AppError::BadGateway(format!("Failed to download file from URL. Status: {}", res.status())));
        }
        
        let bytes = res.bytes().await
            .map_err(|e| AppError::BadGateway(format!("Failed to read file bytes: {}", e)))?
            .to_vec();
            
        (bytes, req.file_name)
    } else {
        // Assume Base64
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let clean_base64 = if let Some(idx) = req.file.find(',') {
            &req.file[idx+1..]
        } else {
            &req.file
        };
        
        let bytes = STANDARD.decode(clean_base64.trim())
            .map_err(|e| AppError::BadRequest(format!("Invalid file base64: {}", e)))?;
        (bytes, req.file_name)
    };

    let attachments = Some(vec![super::provider::Attachment {
        filename,
        content: file_bytes,
        mime_type: "application/pdf".to_string(), // Defaulting to PDF as it's the context, or we could detect
    }]);

    // 3. Select Provider
    let provider_params = ProviderParams {
        provider: Some(req.platform.clone()),
        company: req.company.clone(),
    };
    
    let provider_instance: Box<dyn EmailProvider> = get_provider(&provider_params, state.client.clone());

    // 4. Send Message
    let content_len = req.content.len();
    tracing::info!("Reminder webhook: receiving HTML content ({} bytes)", content_len);

    let send_req = SendMessageRequest {
        to: req.recipients,
        cc: req.cc,
        subject: req.subject,
        body: req.content, // This will be treated as HTML by the provider
        thread_id: None,
        attachments,
    };

    let result: serde_json::Value = provider_instance.send_message(token, send_req).await?;

    Ok(Json(result).into_response())
}

pub async fn get_embed_js(
    State(state): State<AppState>,
) -> impl IntoResponse {
    use tokio::sync::OnceCell;
    static EMBED_JS_CACHE: OnceCell<String> = OnceCell::const_new();

    let js_template = EMBED_JS_CACHE.get_or_init(|| async {
        let path = "frontend/dist/embed.js";
        match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(e) => {
                tracing::error!("Failed to read embed.js: {:?}", e);
                "console.error('Widget script not found on server');".to_string()
            }
        }
    }).await;

    // Inject API key into cached template
    let js = js_template.replace("__API_KEY_PLACEHOLDER__", &state.config.widget_api_key);

    axum::response::Response::builder()
        .header("Content-Type", "application/javascript")
        // Cache for 1 hour in browser, validate with server occasionally if needed but max-age is good for speed
        .header("Cache-Control", "public, max-age=3600") 
        .body(axum::body::Body::from(js))
        .unwrap()
}
