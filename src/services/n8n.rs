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

        let api_key = std::env::var("N8N_API_KEY").unwrap_or_else(|_| "n8n_api_b5f34067cdcd60c1dc6dbcb5d999fdbbad1f9aba10cf475024e6ba9534643dc498c5cb0e11c05d36".to_string());
        let res = self.client.post(&self.webhook_url)
            .header("X-N8N-API-KEY", api_key)
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
