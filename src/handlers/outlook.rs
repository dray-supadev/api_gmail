use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::error::AppError;
use super::provider::{EmailProvider, CleanMessage, MessageSummary, AttachmentSummary, SendMessageRequest, ListParams};

pub struct OutlookProvider;

impl OutlookProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl EmailProvider for OutlookProvider {
    async fn list_messages(&self, token: &str, params: ListParams) -> Result<serde_json::Value, AppError> {
        let client = Client::new();
        let mut url = "https://graph.microsoft.com/v1.0/me/messages".to_string();
        
        let mut query = Vec::new();
        query.push("$select=id,subject,from,receivedDateTime,isRead,hasAttachments,bodyPreview,conversationId".to_string());
        query.push("$top=10".to_string()); // Default top
        
        if let Some(max) = params.max_results {
            query.push(format!("$top={}", max));
        }
        
        // Outlook pagination uses $skip or $deltatoken/skiptoken, but simpler to rely on @odata.nextLink provided by API
        // If parameters contain a direct link (from next_page_token hack), use it.
        // For strict page numbers, we might need $skip = (page-1)*limit.
        
        if let Some(page) = params.page_number {
            if page > 1 {
                 let skip = (page - 1) * params.max_results.unwrap_or(10);
                 query.push(format!("$skip={}", skip));
            }
        }

        if let Some(q) = &params.q {
             query.push(format!("$search=\"{}\"", q));
        }

        if !query.is_empty() {
            url = format!("{}?{}", url, query.join("&"));
        }

        let res = client.get(&url)
            .bearer_auth(token)
            .send()
            .await?;

        if !res.status().is_success() {
             return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
        }

        let data: serde_json::Value = res.json().await?;
        
        // Map to standard response format if needed, or return raw for now and let handler map it
        // Check Unified API design: "Unified Message Models".
        // For now returning raw graph response, but we should eventually unify.
        
        Ok(data)
    }

    async fn get_message(&self, token: &str, id: &str) -> Result<CleanMessage, AppError> {
        let client = Client::new();
        let url = format!("https://graph.microsoft.com/v1.0/me/messages/{}", id);
        
        let res = client.get(&url)
            .bearer_auth(token)
            .header("Prefer", "outlook.body-content-type=\"text\"") 
            .send()
            .await?;

        if !res.status().is_success() {
             return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
        }
        
        let data: serde_json::Value = res.json().await?;
        
        // Parse Outlook JSON to CleanMessage
        let subject = data["subject"].as_str().map(|s| s.to_string());
        let from = data["from"]["emailAddress"]["name"].as_str()
            .or_else(|| data["from"]["emailAddress"]["address"].as_str())
            .map(|s| s.to_string());
        let date = data["receivedDateTime"].as_str().map(|s| s.to_string());
        let snippet = data["bodyPreview"].as_str().unwrap_or("").to_string();
        
        // Fetch attachments separately if needed or expand? 
        // Graph API usually requires /attachments endpoint for details.
        // For summary, we can check hasAttachments.
        
        Ok(CleanMessage {
            id: id.to_string(),
            subject,
            from,
            to: None, // TODO extract TO
            date,
            snippet,
            body_text: data["body"]["content"].as_str().map(|s| s.to_string()),
            body_html: None, // Simplified
            attachments: vec![], // TODO fetch attachments
        })
    }
    
    async fn get_thread(&self, token: &str, id: &str) -> Result<serde_json::Value, AppError> {
        // Outlook "conversationId" is not exactly threadId in Gmail sense, but close.
        Ok(json!({}))
    }

    async fn send_message(&self, token: &str, req: SendMessageRequest) -> Result<serde_json::Value, AppError> {
         let client = Client::new();
         let url = "https://graph.microsoft.com/v1.0/me/sendMail";
         
         let body = json!({
             "message": {
                 "subject": req.subject,
                 "body": {
                     "contentType": "HTML",
                     "content": req.body
                 },
                 "toRecipients": [
                     {
                         "emailAddress": {
                             "address": req.to
                         }
                     }
                 ]
             },
             "saveToSentItems": "true"
         });

         let res = client.post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;
            
         if !res.status().is_success() {
             return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
         }
         
         Ok(json!({"status": "sent"}))
    }
}
