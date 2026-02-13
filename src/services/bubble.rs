use reqwest::{Client, multipart};
use serde_json::Value;
use crate::error::AppError;
use std::env;

pub struct BubbleService {
    client: Client,
    base_url: String,
    api_token: String,
}

impl BubbleService {
    pub fn new(client: Client) -> Result<Self, AppError> {
        let api_token = env::var("BUBBLE_API_TOKEN")
            .map_err(|_| AppError::Config("BUBBLE_API_TOKEN must be set".to_string()))?;
        
        let base_url = env::var("BUBBLE_APP_URL").unwrap_or_else(|_| "https://revup-22775.bubbleapps.io".to_string()); 

        Ok(Self {
            client,
            base_url,
            api_token,
        })
    }

    pub async fn fetch_quote(&self, version: Option<&str>, quote_id: &str) -> Result<Value, AppError> {
        let version_path = version.unwrap_or("version-test"); // Default for safety, but should be passed
        let url = format!("{}/{}/api/1.1/obj/quote_details/{}", self.base_url, version_path, quote_id); // Dynamic version URL

        let res = self.client.get(&url)
            .bearer_auth(&self.api_token)
            .send()
            .await?;

        if !res.status().is_success() {
             return Err(AppError::BubbleApi(res.error_for_status().unwrap_err()));
        }

        let body: Value = res.json().await?;
        Ok(body)
    }

    pub async fn generate_pdf_via_workflow(&self, quote_id: &str, version: Option<&str>, settings: Option<Vec<String>>) -> Result<(Vec<u8>, String, String), AppError> {
        let version_path = version.unwrap_or("version-test");
        let url = format!("{}/{}/api/1.1/wf/get_quote_json", self.base_url, version_path);
        
        let settings_list = settings.unwrap_or_default();

        let payload = serde_json::json!({
            "quote": quote_id,
            "PDFExportSettings": settings_list
        });

        let res = self.client.post(&url)
            .bearer_auth(&self.api_token)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
             let error_text = res.text().await.unwrap_or_default();
             return Err(AppError::BadGateway(format!("Bubble Workflow failed: {}", error_text)));
        }

        // Response expected: { "response": { "pdfFile": "https://...", "pdfName": "Quote.pdf" } }
        let body: Value = res.json().await?;
        let response_data = &body["response"];
        
        // Handle cases where Bubble returns "http:" but we prefer "https:" or raw URLs
        let pdf_url = response_data["pdfFile"].as_str()
            .ok_or(AppError::Config("Missing pdfFile in Bubble response".to_string()))?;
            
        // Correct protocol if needed (Bubble sometimes returns //s3...)
        let pdf_url_fixed = if pdf_url.starts_with("//") {
            format!("https:{}", pdf_url)
        } else {
            pdf_url.to_string()
        };
            
        let pdf_name = response_data["pdfName"].as_str().unwrap_or("Quote.pdf").to_string();

        // Download PDF
        let pdf_res = self.client.get(&pdf_url_fixed).send().await?;
        if !pdf_res.status().is_success() {
            return Err(AppError::BadGateway("Failed to download PDF from Bubble".to_string()));
        }
        let pdf_bytes = pdf_res.bytes().await?.to_vec();

        Ok((pdf_bytes, pdf_name, pdf_url_fixed))
    }

    pub async fn fetch_quote_preview(&self, quote_id: &str, version: Option<&str>, settings: Option<Vec<String>>) -> Result<(String, Option<String>), AppError> {
        let version_path = version.unwrap_or("version-test");
        let url = format!("{}/{}/api/1.1/wf/get_quote_preview", self.base_url, version_path);
        
        let settings_list = settings.unwrap_or_default();

        let payload = serde_json::json!({
            "quote": quote_id,
            "PDFExportSettings": settings_list
        });

        let res = self.client.post(&url)
            .bearer_auth(&self.api_token)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
             let status = res.status();
             let error_text = res.text().await.unwrap_or_default();
             
             // Log the error for debugging
             tracing::error!("Bubble API Error ({}): {}", status, error_text);

             if status.is_client_error() {
                 return Err(AppError::BadRequest(format!("Bubble Validation Error: {}", error_text)));
             }
             
             return Err(AppError::BadGateway(format!("Bubble Preview WF failed: {}", error_text)));
        }

        let body: Value = res.json().await?;
        let response = &body["response"];
        
        let html = response["html"].as_str().unwrap_or("").to_string();
        let email_body = response["body"].as_str().map(|s| s.to_string());

        Ok((html, email_body))
    }

    pub async fn send_quote(
        &self,
        quote_id: &str,
        version: Option<&str>,
        pdf_name: &str,
        recipients: Vec<String>,
        cc: Vec<String>,
        subject: &str,
        maildata_identificator: &str,
        pdf_export_settings: Vec<String>,
        pdf_url: String, // Changed from Option<String> + Option<Vec<u8>> to just String (URL)
    ) -> Result<String, AppError> {
        let version_path = version.unwrap_or("version-test");
        let url = format!("{}/{}/api/1.1/wf/send_quote", self.base_url, version_path);

        // Convert lists to JSON strings
        let recipients_json = serde_json::to_string(&recipients).unwrap_or_else(|_| "[]".to_string());
        let cc_json = serde_json::to_string(&cc).unwrap_or_else(|_| "[]".to_string());
        let settings_json = serde_json::to_string(&pdf_export_settings).unwrap_or_else(|_| "[]".to_string());
        
        // User Requirement 5: Pass only text (URL) for PDF
        // Ensure proper protocol
        let final_pdf_url = if pdf_url.starts_with("//") {
            format!("https:{}", pdf_url)
        } else {
            pdf_url.to_string()
        };

        let form = multipart::Form::new()
            .text("quote", quote_id.to_string())
            .text("recipients", recipients_json)
            .text("cc", cc_json)
            .text("pdfname", pdf_name.to_string())
            .text("subject", subject.to_string())
            .text("maildata_Identificator", maildata_identificator.to_string())
            .text("PDFExportSettings", settings_json)
            .text("pdf", final_pdf_url); // Only URL as text

        let res = self.client.post(&url)
            .bearer_auth(&self.api_token)
            .multipart(form)
            .send()
            .await?;

        if !res.status().is_success() {
             let status = res.status();
             let error_text = res.text().await.unwrap_or_default();
             tracing::error!("Bubble Send Quote WF Error ({}): {}", status, error_text);
             return Err(AppError::BadGateway(format!("Bubble Send Quote WF failed: {}", error_text)));
        }

        let body: Value = res.json().await?;
        
        let html_content = body["response"]["html"].as_str()
            .ok_or_else(|| AppError::BadGateway("Bubble did not return html content in response".to_string()))?
            .to_string();

        Ok(html_content)
    }

    pub async fn send_remember(&self, quote_id: &str, version: Option<&str>) -> Result<(), AppError> {
        let version_path = version.unwrap_or("version-test");
        let url = format!("{}/{}/api/1.1/wf/send_remember", self.base_url, version_path);

        let payload = serde_json::json!({
            "quote": quote_id,
        });

        let res = self.client.post(&url)
            .bearer_auth(&self.api_token)
            .json(&payload)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_default();
            tracing::error!("Bubble Send Remember WF Error ({}): {}", status, error_text);
            return Err(AppError::BadGateway(format!("Bubble Send Remember WF failed: {}", error_text)));
        }

        Ok(())
    }
}
