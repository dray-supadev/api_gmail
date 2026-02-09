use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub app_secret_key: String,
    pub bubble_api_token: String,
    pub widget_api_key: String, // Key exposed in public widget script
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenv().ok(); // Load .env if present (dev mode)

        let app_secret_key = std::env::var("APP_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("APP_SECRET_KEY is required"))?;
        
        let bubble_api_token = std::env::var("BUBBLE_API_TOKEN")
            .unwrap_or_default(); // Optional, but useful for fallback auth

        // Security: Separate key for public widget. Fallback to app_secret_key for backward compatibility,
        // but it is HIGHLY recommended to set a separate WIDGET_API_KEY.
        let widget_api_key = std::env::var("WIDGET_API_KEY")
            .unwrap_or_else(|_| {
                tracing::warn!("WIDGET_API_KEY not set. Using APP_SECRET_KEY for widget. This is insecure for public widgets.");
                app_secret_key.clone()
            });

        Ok(Self {
            app_secret_key,
            bubble_api_token,
            widget_api_key,
        })
    }
}
