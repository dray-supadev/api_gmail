use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub app_secret_key: String,
    pub bubble_api_token: String,
    pub widget_api_key: String, // Key exposed in public widget script
    pub allowed_origins: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenv().ok(); // Load .env if present (dev mode)

        let app_secret_key = std::env::var("APP_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("APP_SECRET_KEY is required"))?;
        
        let bubble_api_token = std::env::var("BUBBLE_API_TOKEN")
            .unwrap_or_default();

        // Security: Separate key for public widget.
        let widget_api_key = std::env::var("WIDGET_API_KEY")
            .map_err(|_| anyhow::anyhow!("WIDGET_API_KEY is required for security. Set it to a different value than APP_SECRET_KEY."))?;

        if widget_api_key == app_secret_key {
             tracing::error!("CRITICAL SECURITY RISK: WIDGET_API_KEY is the same as APP_SECRET_KEY. Public widget will expose admin access!");
        }

        let allowed_origins = std::env::var("ALLOWED_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(Self {
            app_secret_key,
            bubble_api_token,
            widget_api_key,
            allowed_origins,
        })
    }
}
