use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use crate::error::AppError;
use super::provider::{EmailProvider, CleanMessage, MessageSummary, SendMessageRequest, ListParams, Label, BatchModifyRequest};

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
        let mut url = if let Some(label_id) = params.label_ids.as_deref() {
            if label_id == "INBOX" {
                "https://graph.microsoft.com/v1.0/me/mailFolders/inbox/messages".to_string()
            } else if label_id == "SENT" {
                "https://graph.microsoft.com/v1.0/me/mailFolders/sentitems/messages".to_string()
            } else if label_id == "DRAFT" {
                "https://graph.microsoft.com/v1.0/me/mailFolders/drafts/messages".to_string()
            } else if label_id == "TRASH" {
                "https://graph.microsoft.com/v1.0/me/mailFolders/deleteditems/messages".to_string()
            } else {
                format!("https://graph.microsoft.com/v1.0/me/mailFolders/{}/messages", label_id)
            }
        } else {
            "https://graph.microsoft.com/v1.0/me/messages".to_string()
        };

        let mut query = Vec::new();
        query.push("$select=id,subject,from,receivedDateTime,isRead,hasAttachments,bodyPreview,conversationId".to_string());
        
        // Fixed Point 12: Avoid duplicate $top
        let top = params.max_results.unwrap_or(10);
        query.push(format!("$top={}", top));
        
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
        
        let messages_raw = data["value"].as_array().ok_or_else(|| anyhow::anyhow!("Messages not found in response"))?;
        
        let summaries: Vec<MessageSummary> = messages_raw.iter().map(|m| {
            MessageSummary {
                id: m["id"].as_str().unwrap_or("").to_string(),
                thread_id: m["conversationId"].as_str().unwrap_or("").to_string(),
                snippet: m["bodyPreview"].as_str().unwrap_or("").to_string(),
                subject: m["subject"].as_str().map(|s| s.to_string()),
                from: m["from"]["emailAddress"]["address"].as_str()
                    .or_else(|| m["from"]["emailAddress"]["name"].as_str())
                    .map(|s| s.to_string()),
                date: m["receivedDateTime"].as_str().map(|s| s.to_string()),
                unread: !m["isRead"].as_bool().unwrap_or(true),
                has_attachments: m["hasAttachments"].as_bool().unwrap_or(false),
                messages_in_thread: None,
            }
        }).collect();
        
        Ok(json!({
            "messages": summaries,
            "@odata.nextLink": data["@odata.nextLink"]
        }))
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
        
        // Prioritize address for "from" if it's for an email field
        let from = data["from"]["emailAddress"]["address"].as_str()
            .or_else(|| data["from"]["emailAddress"]["name"].as_str())
            .map(|s| s.to_string());
            
        let date = data["receivedDateTime"].as_str().map(|s| s.to_string());
        let snippet = data["bodyPreview"].as_str().unwrap_or("").to_string();
        
        // Extract recipients
        let to = data["toRecipients"].as_array().map(|recipients| {
            recipients.iter()
                .filter_map(|r| r["emailAddress"]["address"].as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        });

        Ok(CleanMessage {
            id: id.to_string(),
            subject,
            from,
            to,
            date,
            snippet,
            body_text: data["body"]["content"].as_str().map(|s| s.to_string()),
            body_html: None, 
            attachments: vec![],
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

         let cc_recipients: Vec<serde_json::Value> = req.cc.unwrap_or_default().iter().map(|email| {
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
                 "ccRecipients": cc_recipients,
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
        
        // Moving to folder in Outlook is done per-message via POST /messages/{id}/move
        // We only support moving to a SINGLE folder (the first one in add_label_ids)
        // We clone getting the first folder ID to avoid borrowing issues while iterating.
        let target_folder = req.add_label_ids.as_ref().and_then(|ids| ids.first()).cloned();
        
        if let Some(folder_id) = target_folder {
            let mut tasks = Vec::new();
            
            for message_id in req.ids {
                let client = self.client.clone();
                let token = token.to_string();
                let folder_id = folder_id.clone();
                
                tasks.push(tokio::spawn(async move {
                    let url = format!("https://graph.microsoft.com/v1.0/me/messages/{}/move", message_id);
                    let body = json!({
                        "destinationId": folder_id
                    });

                    client.post(&url)
                        .bearer_auth(token)
                        .json(&body)
                        .send()
                        .await
                }));
            }

            // Execute all move requests in parallel
            let results = futures::future::join_all(tasks).await;

            // Check for errors
            for res in results {
                match res {
                    Ok(Ok(response)) => {
                        if !response.status().is_success() {
                            return Err(AppError::OutlookApi(response.error_for_status().unwrap_err()));
                        }
                    },
                    Ok(Err(e)) => return Err(AppError::OutlookApi(e)), // Reqwest error
                    Err(e) => return Err(AppError::Internal(anyhow::anyhow!("Task join error: {}", e))), // Join error
                }
            }
        }

        Ok(())
    }
}
