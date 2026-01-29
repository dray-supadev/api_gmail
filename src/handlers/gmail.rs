use axum::{
    extract::{Path, Query, Json},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use reqwest::Client;
use crate::error::AppError;
use mail_parser::{MessageParser, MimeHeaders};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

// --- DTOs ---

#[derive(Deserialize)]
pub struct ListParams {
    pub label_ids: Option<String>, // Comma separated
    pub max_results: Option<u32>,
    pub q: Option<String>,
    pub page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MessageSummary {
    pub id: String,
    pub thread_id: String,
    pub snippet: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub date: Option<String>,
    pub unread: bool,
    pub has_attachments: bool,
}

#[derive(Serialize)]
pub struct CleanMessage {
    pub id: String,
    pub subject: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub date: Option<String>,
    pub snippet: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentSummary>,
}

#[derive(Serialize)]
pub struct AttachmentSummary {
    pub filename: String,
    pub content_type: String,
    pub size: usize,
    pub id: Option<String>, // Content-ID for inline images
}

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub subject: String,
    pub body: String, // Treat as HTML by default
}

// --- Helpers ---

fn get_google_token(headers: &HeaderMap) -> Result<&str, AppError> {
    headers
        .get("x-google-token")
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::MissingToken)
}

// --- Handlers ---

pub async fn list_messages(
    headers: HeaderMap,
    Query(params): Query<ListParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = get_google_token(&headers)?;
    let client = Client::new();

    let mut url = "https://gmail.googleapis.com/gmail/v1/users/me/messages".to_string();
    
    // Build query params
    let mut query = Vec::new();
    if let Some(max) = params.max_results {
        query.push(format!("maxResults={}", max));
    }
    if let Some(q) = params.q {
        query.push(format!("q={}", urlencoding::encode(&q)));
    }
    if let Some(token_param) = params.page_token {
        query.push(format!("pageToken={}", token_param));
    }
    if let Some(labels) = params.label_ids {
        for label in labels.split(',') {
            query.push(format!("labelIds={}", label.trim()));
        }
    }

    if !query.is_empty() {
        url = format!("{}?{}", url, query.join("&"));
    }

    // Get list of message IDs
    let res = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
    }

    let list_response: serde_json::Value = res.json().await?;
    
    // Extract message IDs
    let messages = list_response["messages"]
        .as_array()
        .map(|arr| arr.to_vec())
        .unwrap_or_default();
    
    if messages.is_empty() {
        return Ok(Json(json!({
            "messages": [],
            "nextPageToken": list_response["nextPageToken"],
            "resultSizeEstimate": 0
        })));
    }

    // Fetch metadata for each message in parallel
    let mut tasks = Vec::new();
    
    for msg in messages {
        let id = msg["id"].as_str().unwrap_or("").to_string();
        let thread_id = msg["threadId"].as_str().unwrap_or("").to_string();
        let client_clone = client.clone();
        let token_clone = token.to_string();
        
        tasks.push(tokio::spawn(async move {
            fetch_message_metadata(&client_clone, &token_clone, &id, &thread_id).await
        }));
    }

    // Wait for all tasks to complete
    let results = futures::future::join_all(tasks).await;
    
    let enriched_messages: Vec<MessageSummary> = results
        .into_iter()
        .filter_map(|r| r.ok().and_then(|m| m.ok()))
        .collect();

    Ok(Json(json!({
        "messages": enriched_messages,
        "nextPageToken": list_response["nextPageToken"],
        "resultSizeEstimate": list_response["resultSizeEstimate"]
    })))
}

pub async fn get_message(
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<CleanMessage>, AppError> {
    let token = get_google_token(&headers)?;
    let client = Client::new();

    // Fetch RAW message (best for parsing fidelity)
    let url = format!("https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=raw", id);
    
    let res = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
    }

    let data: serde_json::Value = res.json().await?;
    
    // Decode Base64Url raw content
    let raw_base64 = data["raw"].as_str().unwrap_or_default();
    let raw_bytes = URL_SAFE_NO_PAD.decode(raw_base64).map_err(|e| anyhow::anyhow!("Base64 Error: {}", e))?;

    // Parse MIME
    let message = MessageParser::default().parse(&raw_bytes).ok_or_else(|| anyhow::anyhow!("Failed to parse email"))?;

    // Convert to Clean JSON
    let clean = CleanMessage {
        id,
        subject: message.subject().map(|s| s.to_string()),
        from: message.from().map(|f| f.first().map(|a| a.name().unwrap_or(a.address().unwrap_or("Unknown"))).unwrap_or("Unknown").to_string()),
        to: message.to().map(|t| t.first().map(|a| a.address().unwrap_or("Unknown")).unwrap_or("Unknown").to_string()), 
        date: message.date().map(|d| d.to_rfc3339()),
        snippet: data["snippet"].as_str().unwrap_or("").to_string(),
        body_text: message.body_text(0).map(|b| b.to_string()),
        body_html: message.body_html(0).map(|b| b.to_string()),
        attachments: message.attachments().map(|a| {
            let filename = a.attachment_name()
                .or_else(|| a.content_type().and_then(|ct| ct.attribute("name")))
                .unwrap_or("unnamed")
                .to_string();
            
            let content_type = a.content_type()
                .map(|ct| format!("{}/{}", ct.c_type, ct.c_subtype.as_ref().unwrap_or(&"octet-stream".into())))
                .unwrap_or_else(|| "application/octet-stream".to_string());
            
            AttachmentSummary {
                filename,
                content_type,
                size: a.contents().len(),
                id: a.content_id().map(|id| id.to_string()),
            }
        }).collect(),
    };

    Ok(Json(clean))
}

pub async fn send_message(
    headers: HeaderMap,
    Json(payload): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let token = get_google_token(&headers)?;
    let client = Client::new();

    // Simple MIME construction
    // For production, consider using 'lettre' crate for complex MIME (attachments, etc)
    let email_content = format!(
        "To: {}\r\nSubject: {}\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{}",
        payload.to, payload.subject, payload.body
    );

    let raw_encoded = URL_SAFE_NO_PAD.encode(email_content.as_bytes());

    let body = json!({
        "raw": raw_encoded
    });

    let res = client
        .post("https://gmail.googleapis.com/gmail/v1/users/me/messages/send")
        .bearer_auth(token)
        .json(&body)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
    }

    let json: serde_json::Value = res.json().await?;
    Ok(Json(json))
}

// Helper function to fetch metadata for a single message
async fn fetch_message_metadata(
    client: &Client,
    token: &str,
    id: &str,
    thread_id: &str,
) -> Result<MessageSummary, AppError> {
    let url = format!(
        "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=metadata&metadataHeaders=Subject&metadataHeaders=From&metadataHeaders=Date",
        id
    );
    
    let res = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await?;
    
    if !res.status().is_success() {
        return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
    }
    
    let data: serde_json::Value = res.json().await?;
    
    // Parse headers
    const EMPTY_ARRAY: &[serde_json::Value] = &[];
    let headers = data["payload"]["headers"].as_array().map_or(EMPTY_ARRAY, |v| v.as_slice());
    
    let subject = headers
        .iter()
        .find(|h| h["name"].as_str() == Some("Subject"))
        .and_then(|h| h["value"].as_str())
        .map(|s| s.to_string());
    
    let from = headers
        .iter()
        .find(|h| h["name"].as_str() == Some("From"))
        .and_then(|h| h["value"].as_str())
        .map(|s| s.to_string());
    
    let date = headers
        .iter()
        .find(|h| h["name"].as_str() == Some("Date"))
        .and_then(|h| h["value"].as_str())
        .map(|s| s.to_string());
    
    // Check if unread (labelIds contains "UNREAD")
    let unread = data["labelIds"]
        .as_array()
        .map(|labels| labels.iter().any(|l| l.as_str() == Some("UNREAD")))
        .unwrap_or(false);
    
    // Check for attachments
    let has_attachments = has_attachments_in_payload(&data["payload"]);
    
    let snippet = data["snippet"].as_str().unwrap_or("").to_string();
    
    Ok(MessageSummary {
        id: id.to_string(),
        thread_id: thread_id.to_string(),
        snippet,
        subject,
        from,
        date,
        unread,
        has_attachments,
    })
}

// Recursively check if payload has attachments
fn has_attachments_in_payload(payload: &serde_json::Value) -> bool {
    if let Some(filename) = payload["filename"].as_str() {
        if !filename.is_empty() {
            return true;
        }
    }
    
    if let Some(parts) = payload["parts"].as_array() {
        for part in parts {
            if has_attachments_in_payload(part) {
                return true;
            }
        }
    }
    
    false
}
