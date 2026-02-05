use axum::{
    // extract::{Json},
    // response::{IntoResponse},
};
use serde_json::json;
use reqwest::Client;
use crate::error::AppError;
use mail_parser::{MessageParser, MimeHeaders};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
// --- Global Pagination Cache ---
use std::sync::{Mutex};
use std::collections::HashMap;
use std::sync::OnceLock;

use super::provider::{EmailProvider, CleanMessage, MessageSummary, AttachmentSummary, SendMessageRequest, ListParams};

// Key for the cache: (Google Token Hash + Query Params Hash) -> Page Number -> Gmail Token
// We use a simple string key: "{token_hash}_{query}_{labels}_{max}_{page}"
static PAGINATION_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_cache() -> &'static Mutex<HashMap<String, String>> {
    PAGINATION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct GmailProvider;

impl GmailProvider {
    pub fn new() -> Self {
        Self
    }

    // Helper to fetch and parse a single message fully
    async fn fetch_and_parse_message(
        &self,
        client: &Client,
        token: &str,
        id: &str,
    ) -> Result<CleanMessage, AppError> {
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
        let sanitized_base64 = raw_base64.trim_end_matches('=');
        let raw_bytes = URL_SAFE_NO_PAD.decode(sanitized_base64).map_err(|e| anyhow::anyhow!("Base64 Error: {} (len: {})", e, raw_base64.len()))?;

        // Parse MIME
        let message = MessageParser::default().parse(&raw_bytes).ok_or_else(|| anyhow::anyhow!("Failed to parse email"))?;

        // Convert to Clean JSON
        let clean = CleanMessage {
            id: id.to_string(),
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

        Ok(clean)
    }

    // Helper function to fetch metadata for a single message
    async fn fetch_message_metadata(
        &self,
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
            messages_in_thread: None, // Not set for individual message fetch
        })
    }
}

#[async_trait::async_trait]
impl EmailProvider for GmailProvider {
    async fn list_messages(
        &self,
        token: &str,
        params: ListParams,
    ) -> Result<serde_json::Value, AppError> {
        let client = Client::new();

        let mut url = "https://gmail.googleapis.com/gmail/v1/users/me/messages".to_string();
        
        // Pagination Logic
        let page_num = params.page_number.unwrap_or(1);
        
        // Determine the actual Gmail Token to use
        let actual_token = if page_num <= 1 {
            // Page 1 always has no token
            None 
        } else if let Some(manual_token) = &params.page_token {
            // User provided explicit token (overrides page number)
            Some(manual_token.clone())
        } else {
            // Look up token for this page number in Cache
            // Create a unique cache key for this user's specific query
            let cache_key_prefix = format!(
                "{}_{}_{}_{}",
                simple_hash(token),
                params.q.as_deref().unwrap_or(""),
                params.label_ids.as_deref().unwrap_or(""),
                params.max_results.unwrap_or(10)
            );
            let key = format!("{}_{}", cache_key_prefix, page_num);
            
            // Try to get token from cache
            let cache = get_cache().lock().unwrap();
            match cache.get(&key) {
                Some(t) => Some(t.clone()),
                None => {
                    if page_num > 1 {
                         return Ok(json!({
                            "messages": [],
                            "resultSizeEstimate": 0,
                            "warning": "Page token not found. Please navigate sequentially from Page 1."
                        }));
                    }
                    None
                }
            }
        };

        // Build query params
        let mut query = Vec::new();
        if let Some(max) = params.max_results {
            query.push(format!("maxResults={}", max));
        }
        if let Some(q) = &params.q {
            query.push(format!("q={}", urlencoding::encode(q)));
        }
        
        // Use the resolved token
        if let Some(t) = actual_token {
            query.push(format!("pageToken={}", t));
        }
        
        if let Some(labels) = &params.label_ids {
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
        
        // CACHE UPDATE: Save the 'nextPageToken' for the NEXT page (current + 1)
        if let Some(next_token) = list_response["nextPageToken"].as_str() {
            let cache_key_prefix = format!(
                "{}_{}_{}_{}",
                simple_hash(token),
                params.q.as_deref().unwrap_or(""),
                params.label_ids.as_deref().unwrap_or(""),
                params.max_results.unwrap_or(10)
            );
            let next_page_num = page_num + 1;
            let key = format!("{}_{}", cache_key_prefix, next_page_num);
            
            let mut cache = get_cache().lock().unwrap();
            cache.insert(key, next_token.to_string());
        }
        
        // Extract message IDs
        let messages_raw = list_response["messages"]
            .as_array()
            .map(|arr| arr.to_vec())
            .unwrap_or_default();
        
        if messages_raw.is_empty() {
            return Ok(json!({
                "messages": [],
                "nextPageToken": list_response["nextPageToken"],
                "page": page_num,
                "resultSizeEstimate": 0
            }));
        }

        // Fetch metadata for each message in parallel
        let mut tasks = Vec::new();
        
        for msg in messages_raw {
            let id = msg["id"].as_str().unwrap_or("").to_string();
            let thread_id = msg["threadId"].as_str().unwrap_or("").to_string();
            let client_clone = client.clone();
            let token_clone = token.to_string();
            // Need to clone self to move into async block, but self is reference.
            // Actually, we can just use the helper method logic or make helper static/function.
            // Making helper a method on &self makes it hard to spawn.
            // For now, let's clone the provider if it was cheap (it is ZST).
            // Better: just move the logic into an async block or Arc<Self>.
            // Since GmailProvider is ZST (Zero Sized Type), we can just construct it inside or make methods standalone.
            // Let's make `fetch_message_metadata` a standalone function or associate it with the implementation.
            tasks.push(tokio::spawn(async move {
                // HACK: Re-instantiating provider here or just copying logic?
                // The helper function uses `has_attachments_in_payload` which is standalone.
                // Let's just make the helper function NOT a method of self, or just static.
                // For this refactor, I'll assume `fetch_message_metadata` is moved out of impl or we Clone.
                // Since `GmailProvider` is ZST, we can create a new one.
                let provider = GmailProvider::new();
                provider.fetch_message_metadata(&client_clone, &token_clone, &id, &thread_id).await
            }));
        }

        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;
        
        let mut enriched_messages: Vec<MessageSummary> = results
            .into_iter()
            .filter_map(|r| r.ok().and_then(|m| m.ok()))
            .collect();

        // If collapse_threads is enabled, group by thread_id and keep only the latest message
        if params.collapse_threads.unwrap_or(false) {
            use std::collections::HashMap;
            
            let mut threads: HashMap<String, Vec<MessageSummary>> = HashMap::new();
            
            // Group messages by thread_id
            for msg in enriched_messages {
                threads.entry(msg.thread_id.clone())
                    .or_insert_with(Vec::new)
                    .push(msg);
            }
            
            // For each thread, keep only the latest message and add count
            enriched_messages = threads
                .into_iter()
                .map(|(_thread_id, mut msgs)| {
                    let count = msgs.len() as u32;
                    // Sort by date (newest first) - use date string comparison as fallback
                    msgs.sort_by(|a, b| b.date.cmp(&a.date));
                    
                    // Take the latest message and add thread count
                    let mut latest = msgs.into_iter().next().unwrap();
                    latest.messages_in_thread = Some(count);
                    latest
                })
                .collect();
            
            // Sort final results by date (newest first)
            enriched_messages.sort_by(|a, b| b.date.cmp(&a.date));
        }

        Ok(json!({
            "messages": enriched_messages,
            "nextPageToken": list_response["nextPageToken"],
            "page": page_num,
            "next_page": page_num + 1,
            "resultSizeEstimate": list_response["resultSizeEstimate"]
        }))
    }

    async fn get_message(&self, token: &str, id: &str) -> Result<CleanMessage, AppError> {
        let client = Client::new();
        self.fetch_and_parse_message(&client, token, id).await
    }

    async fn get_thread(&self, token: &str, thread_id: &str) -> Result<serde_json::Value, AppError> {
        let client = Client::new();

        // 1. Fetch thread details (minimal format) just to get message IDs
        let url = format!(
            "https://gmail.googleapis.com/gmail/v1/users/me/threads/{}?format=minimal",
            thread_id
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
        
        // Extract message IDs
        const EMPTY_ARRAY: &[serde_json::Value] = &[];
        let messages_data = data["messages"].as_array().map_or(EMPTY_ARRAY, |v| v.as_slice());
        
        // 2. Fetch and parse each message in parallel
        let mut tasks = Vec::new();

        for msg_data in messages_data {
            let id = msg_data["id"].as_str().unwrap_or("").to_string();
            let client_clone = client.clone();
            let token_clone = token.to_string();
            
            tasks.push(tokio::spawn(async move {
                let provider = GmailProvider::new();
                provider.fetch_and_parse_message(&client_clone, &token_clone, &id).await
            }));
        }
        
        // Wait for all tasks to complete
        let results = futures::future::join_all(tasks).await;
        
        let mut messages: Vec<CleanMessage> = results
            .into_iter()
            .filter_map(|r| r.ok().and_then(|m| m.ok()))
            .collect();

        // 3. Sort by date (oldest first for thread view - chronological order)
        messages.sort_by(|a, b| {
            let date_a = a.date.as_deref().unwrap_or("");
            let date_b = b.date.as_deref().unwrap_or("");
            date_a.cmp(date_b)
        });
        
        Ok(json!({
            "thread_id": thread_id,
            "message_count": messages.len(),
            "messages": messages
        }))
    }

    async fn send_message(&self, token: &str, req: SendMessageRequest) -> Result<serde_json::Value, AppError> {
        let client = Client::new();

        let to_header = req.to.join(", ");
        let boundary = "boundary_1234567890"; // Simple static boundary

        let mut email_content = String::new();
        email_content.push_str(&format!("To: {}\r\n", to_header));
        email_content.push_str(&format!("Subject: {}\r\n", req.subject));
        
        let has_attachments = req.attachments.as_ref().map_or(false, |atts| !atts.is_empty());

        if has_attachments {
            email_content.push_str("MIME-Version: 1.0\r\n");
            email_content.push_str(&format!("Content-Type: multipart/mixed; boundary=\"{}\"\r\n\r\n", boundary));
            
            // HTML Part
            email_content.push_str(&format!("--{}\r\n", boundary));
            email_content.push_str("Content-Type: text/html; charset=utf-8\r\n");
            email_content.push_str("Content-Disposition: inline\r\n\r\n");
            email_content.push_str(&req.body);
            email_content.push_str("\r\n\r\n");

            // Attachments
            if let Some(attachments) = &req.attachments {
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                for att in attachments {
                    email_content.push_str(&format!("--{}\r\n", boundary));
                    email_content.push_str(&format!("Content-Type: {}; name=\"{}\"\r\n", att.mime_type, att.filename));
                    email_content.push_str(&format!("Content-Disposition: attachment; filename=\"{}\"\r\n", att.filename));
                    email_content.push_str("Content-Transfer-Encoding: base64\r\n\r\n");
                    
                    let encoded = STANDARD.encode(&att.content);
                    email_content.push_str(&encoded);
                    email_content.push_str("\r\n\r\n");
                }
            }
            email_content.push_str(&format!("--{}--", boundary));
        } else {
             email_content.push_str("Content-Type: text/html; charset=utf-8\r\n\r\n");
             email_content.push_str(&req.body);
        }

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
        Ok(json)
    }
}

// Simple hash for cache keys
fn simple_hash(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish().to_string()
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
