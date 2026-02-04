use reqwest::Client;
use serde_json::Value;
use crate::error::AppError;
use std::env;

pub struct BubbleService {
    client: Client,
    base_url: String,
    api_token: String,
}

impl BubbleService {
    pub fn new() -> Result<Self, AppError> {
        let api_token = env::var("BUBBLE_API_TOKEN")
            .map_err(|_| AppError::Config("BUBBLE_API_TOKEN must be set".to_string()))?;
        
        // Default to the provided version for now if not overridden, but the method accepts version
        let base_url = "https://app.drayinsight.com".to_string(); 

        Ok(Self {
            client: Client::new(),
            base_url,
            api_token,
        })
    }

    pub async fn fetch_quote(&self, version: Option<&str>, quote_id: &str) -> Result<Value, AppError> {
        let version_path = version.unwrap_or("version-test"); // Default for safety, but should be passed
        let url = format!("{}/{}/api/1.1/obj/Quote/{}", self.base_url, version_path, quote_id); // Dynamic version URL

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

    pub fn generate_quote_html(&self, quote_data: &Value, comment: Option<&str>) -> String {
        // Bubble Get Object response: { "response": { "field": ... } }
        let q = &quote_data["response"];

        // Safe extraction with defaults
        let company_name = q.pointer("/terminal/companyName").and_then(|v| v.as_str()).unwrap_or("Your Company");
        let origin = q.pointer("/saved_origin").and_then(|v| v.as_str()).unwrap_or("-");
        let destination = q.pointer("/saved_destination").and_then(|v| v.as_str()).unwrap_or("-");
        
        let pdf_heading_color = q.pointer("/terminal/pdfHeadingColor").and_then(|v| v.as_str()).unwrap_or("#0056b3"); // Default blue
        let pdf_heading_text_color = q.pointer("/terminal/pdfHeadingTextColor").and_then(|v| v.as_str()).unwrap_or("#ffffff");
        let logo_url = q.pointer("/terminal/logo").and_then(|v| v.as_str()).unwrap_or(""); // Handle empty logo gracefully in HTML
        
        let title = q.pointer("/title").and_then(|v| v.as_str()).unwrap_or("Quote Proposal");
        
        let comment_html = if let Some(c) = comment {
            format!(r#"
              <!-- Comment Box -->
              <div style="
                background-color: #f9f9f9;
                border-left: 2px solid #007bff;
                padding: 10px;
                margin: 10px 0;
                font-size: 14px;
              ">
                <p style="margin: 0; line-height: 1.4;">{}</p>
              </div>
              <br>"#, c)
        } else {
            "".to_string()
        };

        // Simple HTML construction using format! macro. 
        // In a real app, a template engine like Askama or Handlebars is recommended, 
        // but for this specific template, string interpolation is fine.
        
        format!(r##"<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>Quote Proposal</title>
</head>
<body style="margin:0; padding:0; font-family: Inter, sans-serif;">
  <table role="presentation" width="100%" cellpadding="0" cellspacing="0" border="0">
    <tr>
      <td align="center">
        <table role="presentation" width="600" bgcolor="#ffffff" cellpadding="0" cellspacing="0" border="0" style="border-radius: 10px; box-shadow: 0 0 10px rgba(0,0,0,0.1);">
         
          <!-- Header -->
          <!-- <tr>
            <td align="center" bgcolor="{heading_color}" style="border-radius: 10px 10px 0 0; padding: 10px;">
              <h1 style="margin:0; color:{heading_text_color}; font-size: 24px;">Quote Proposal from {company_name}</h1>
            </td>
          </tr> -->

          <!-- Main Content -->
          <tr>
            <td style="padding: 20px; color: #333333;">
              <h2 style="color: #000000; margin-top: 0; font-size: 20px;">{title}</h2>
             
              <p style="font-size: 14px; color: #666666; margin: 0 0 5px 0;">
                <strong>Origin:</strong> {origin}
              </p>
              <p style="font-size: 14px; color: #666666; margin: 0 0 5px 0;">
                <strong>Destination:</strong> {destination}
              </p>

              <br><br>

              <!-- Buttons -->
              <table role="presentation" width="100%" align="center" cellpadding="0" cellspacing="0" border="0">
                <tr>
                  <td align="center">
                    <a href="#" style="display:inline-block; background-color:#0C9C35; color:#ffffff; text-decoration:none; border-radius:5px; font-size:14px; padding:10px 20px;">
                        Accept Quote
                    </a>
                  </td>
                  <td width="20"></td>
                  <td align="center">
                    <a href="#" style="display:inline-block; background-color:#B60909; color:#ffffff; text-decoration:none; border-radius:5px; font-size:14px; padding:10px 20px;">
                        Reject Quote
                    </a>
                  </td>
                </tr>
              </table>

              <br><br>
              
              {comment_section}

              <!-- Company Details -->
              <table role="presentation" width="100%" cellpadding="0" cellspacing="0">
                <tr>
                  <td align="center" style="font-size:14px; color:#666666; padding:20px 0;">
                    {logo_html}
                    <p style="margin:0;"><strong>{company_name}</strong></p>
                  </td>
                </tr>
              </table>
              
              <div style="text-align: center; padding: 10px; font-size: 10px; color: #666666;">
                <p style="margin: 0; color: #A3A3A3;">© Dray Insights. All rights reserved.</p>
              </div>
            </td>
          </tr>
        </table>
      </td>
    </tr>
  </table>
</body>
</html>"##, 
        heading_color=pdf_heading_color,
        heading_text_color=pdf_heading_text_color,
        company_name=company_name,
        title=title,
        origin=origin,
        destination=destination,
        comment_section=comment_html,
        logo_html=if !logo_url.is_empty() { format!(r#"<img src="{}" alt="Company Logo" style="object-fit: contain; max-width: 100px; height: auto; margin-bottom: 10px;">"#, logo_url) } else { "".to_string() }
        )
    }
}
