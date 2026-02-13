use super::provider::{EmailProvider, ListParams, SendMessageRequest, BatchModifyRequest, CleanMessage, UserProfile, Label};
use crate::error::AppError;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

pub struct PostmarkProvider {
    client: Client,
    server_token: String,
    company: String,
}

impl PostmarkProvider {
    pub fn new(client: Client, company: String) -> Self {
        let token = std::env::var("POSTMARK_API_TOKEN").expect("POSTMARK_API_TOKEN environment variable must be set");
            
        Self {
            client,
            server_token: token,
            company,
        }
    }
}

#[async_trait]
impl EmailProvider for PostmarkProvider {
    async fn list_messages(&self, _token: &str, _params: ListParams) -> Result<serde_json::Value, AppError> {
        // Postmark in this context is send-only. Return empty list.
        Ok(json!({
            "messages": [],
            "nextPageToken": null,
            "resultSizeEstimate": 0
        }))
    }

    async fn get_message(&self, _token: &str, _id: &str) -> Result<CleanMessage, AppError> {
        Err(AppError::BadRequest("Message viewing not supported for Postmark".to_string()))
    }

    async fn send_message(&self, _token: &str, req: SendMessageRequest) -> Result<serde_json::Value, AppError> {
        let url = "https://api.postmarkapp.com/email";
        
        let from_address = format!("{}@drayinsight.com", self.company.to_lowercase().replace(" ", ""));

        // Convert attachments to Postmark format
        let attachments: Vec<serde_json::Value> = req.attachments.unwrap_or_default().into_iter().map(|att| {
            // Encode content to base64
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let content_base64 = STANDARD.encode(&att.content);
            
            json!({
                "Name": att.filename,
                "Content": content_base64,
                "ContentType": att.mime_type,
            })
        }).collect();

        // Join recipients
        let to = req.to.join(",");
        let cc = req.cc.map(|c| c.join(","));

        // Construct body
        // Note: 'body' in SendMessageRequest is expected to be HTML for our app
        let mut body_json = json!({
            "From": from_address,
            "To": to,
            "Subject": req.subject,
            "HtmlBody": req.body,
            "Attachments": attachments
        });

        if let Some(cc_val) = cc {
            body_json["Cc"] = json!(cc_val);
        }

        let res = self.client.post(url)
            .header("X-Postmark-Server-Token", if _token.is_empty() { &self.server_token } else { _token })
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&body_json)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            tracing::error!("Postmark API error: {}", error_text);
            return Err(AppError::BadRequest(format!("Postmark error: {}", error_text)));
        }

        let data: serde_json::Value = res.json().await?;
        Ok(data)
    }

    async fn list_labels(&self, _token: &str) -> Result<Vec<Label>, AppError> {
        // No labels for Postmark
        Ok(vec![])
    }

    async fn batch_modify_labels(&self, _token: &str, _req: BatchModifyRequest) -> Result<(), AppError> {
        // Not supported
        Ok(())
    }

    async fn get_profile(&self, _token: &str) -> Result<UserProfile, AppError> {
        Ok(UserProfile {
            email: format!("{}@drayinsight.com", self.company.to_lowercase().replace(" ", "")),
            name: Some(self.company.clone()),
            picture: None,
        })
    }
}
