use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::error::AppError;
use super::provider::{EmailProvider, CleanMessage, SendMessageRequest, ListParams, Label, BatchModifyRequest};

pub struct OutlookProvider {
    client: Client,
}

impl OutlookProvider {
    pub fn new(client: Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl EmailProvider for OutlookProvider {
    async fn list_messages(&self, token: &str, params: ListParams) -> Result<serde_json::Value, AppError> {
        let mut url = "https://graph.microsoft.com/v1.0/me/messages".to_string();
        let mut query = Vec::new();
        query.push("$select=id,subject,from,receivedDateTime,isRead,hasAttachments,bodyPreview,conversationId".to_string());
        
        // Fixed Point 12: Avoid duplicate $top
        let top = params.max_results.unwrap_or(10);
        query.push(format!("$top={}", top));
        
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

        let res = self.client.get(&url)
            .bearer_auth(token)
            .send()
            .await?;

        if !res.status().is_success() {
             // Fixed Point 10: Specific Outlook error
             return Err(AppError::OutlookApi(res.error_for_status().unwrap_err()));
        }

        let data: serde_json::Value = res.json().await?;
        
        // Map to standard response format if needed, or return raw for now and let handler map it
        // Check Unified API design: "Unified Message Models".
        // For now returning raw graph response, but we should eventually unify.
        
        Ok(data)
    }

    async fn get_message(&self, token: &str, id: &str) -> Result<CleanMessage, AppError> {
        let url = format!("https://graph.microsoft.com/v1.0/me/messages/{}", id);
        
        let res = self.client.get(&url)
            .bearer_auth(token)
            .header("Prefer", "outlook.body-content-type=\"text\"") 
            .send()
            .await?;

        if !res.status().is_success() {
             // Fixed Point 10
             return Err(AppError::OutlookApi(res.error_for_status().unwrap_err()));
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
    
    async fn send_message(&self, token: &str, req: SendMessageRequest) -> Result<serde_json::Value, AppError> {
         let url = "https://graph.microsoft.com/v1.0/me/sendMail";
         
         let recipients: Vec<serde_json::Value> = req.to.iter().map(|email| {
             json!({
                 "emailAddress": {
                     "address": email
                 }
             })
         }).collect();

         let mut attachments_json = vec![];
         if let Some(attachments) = req.attachments {
             use base64::{Engine as _, engine::general_purpose};
             for att in attachments {
                 attachments_json.push(json!({
                     "@odata.type": "#microsoft.graph.fileAttachment",
                     "name": att.filename,
                     "contentType": att.mime_type,
                     "contentBytes": general_purpose::STANDARD.encode(&att.content) 
                 }));
             }
         }

         let body = json!({
             "message": {
                 "subject": req.subject,
                 "body": {
                     "contentType": "HTML",
                     "content": req.body
                 },
                 "toRecipients": recipients,
                 "attachments": attachments_json
             },
             "saveToSentItems": "true"
         });

         let res = self.client.post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await?;
            
         if !res.status().is_success() {
             // Fixed Point 10
             return Err(AppError::OutlookApi(res.error_for_status().unwrap_err()));
         }
         
         Ok(json!({"status": "sent"}))
    }

    async fn list_labels(&self, token: &str) -> Result<Vec<Label>, AppError> {
        let url = "https://graph.microsoft.com/v1.0/me/mailFolders?$top=99";

        let res = self.client
            .get(url)
            .bearer_auth(token)
            .send()
            .await?;

        if !res.status().is_success() {
            // Fixed Point 10
            return Err(AppError::OutlookApi(res.error_for_status().unwrap_err()));
        }

        let data: serde_json::Value = res.json().await?;
        let folders = data["value"].as_array().ok_or_else(|| anyhow::anyhow!("Folders not found"))?;

        let labels = folders
            .iter()
            .map(|f| Label {
                id: f["id"].as_str().unwrap_or("").to_string(),
                name: f["displayName"].as_str().unwrap_or("").to_string(),
                label_type: Some("user".to_string()), // Simplified
            })
            .collect();

        Ok(labels)
    }

    async fn batch_modify_labels(&self, token: &str, req: BatchModifyRequest) -> Result<(), AppError> {
        let client = Client::new();
        
        // Moving to folder in Outlook is done per-message via POST /messages/{id}/move
        // We only support moving to a SINGLE folder (the first one in add_label_ids)
        let target_folder = req.add_label_ids.and_then(|ids| ids.into_iter().next());
        
        if let Some(folder_id) = target_folder {
            for message_id in req.ids {
                let url = format!("https://graph.microsoft.com/v1.0/me/messages/{}/move", message_id);
                let body = json!({
                    "destinationId": folder_id
                });

                let res = self.client
                    .post(&url)
                    .bearer_auth(token)
                    .json(&body)
                    .send()
                    .await?;

                if !res.status().is_success() {
                    // Fixed Point 10
                    return Err(AppError::OutlookApi(res.error_for_status().unwrap_err()));
                }
            }
        }

        Ok(())
    }
}
