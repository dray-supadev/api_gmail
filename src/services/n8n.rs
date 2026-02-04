use reqwest::Client;
use serde_json::json;
use crate::error::AppError;

pub struct N8NService {
    client: Client,
    webhook_url: String,
}

impl N8NService {
    pub fn new() -> Self {
        // Hardcoded for now based on user request, or could be env var
        let webhook_url = "https://n8n-n8n.jyohlh.easypanel.host/webhook/render-pdf".to_string();
        
        Self {
            client: Client::new(),
            webhook_url,
        }
    }

    pub async fn generate_pdf(&self, html_content: &str) -> Result<Vec<u8>, AppError> {
        let payload = json!({
            "html": html_content
        });

        let res = self.client.post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
             return Err(AppError::GmailApi(res.error_for_status().unwrap_err()));
        }

        let pdf_bytes = res.bytes().await?;
        Ok(pdf_bytes.to_vec())
    }
}
