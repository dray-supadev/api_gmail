use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub app_secret_key: String,
    pub bubble_api_token: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenv().ok(); // Load .env if present (dev mode)

        let app_secret_key = std::env::var("APP_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("APP_SECRET_KEY is required"))?;
        
        let bubble_api_token = std::env::var("BUBBLE_API_TOKEN")
            .unwrap_or_default(); // Optional, but useful for fallback auth

        Ok(Self {
            app_secret_key,
            bubble_api_token,
        })
    }
}
