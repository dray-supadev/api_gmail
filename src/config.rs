use std::sync::Arc;
use dotenvy::dotenv;
use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct Config {
    pub app_secret_key: String,
}

impl Config {
    pub fn load() -> Result<Self, anyhow::Error> {
        dotenv().ok(); // Load .env if present (dev mode)

        // In production (Easypanel), these come from actual env vars
        let app_secret_key = std::env::var("APP_SECRET_KEY")
            .map_err(|_| anyhow::anyhow!("APP_SECRET_KEY is required"))?;

        Ok(Self {
            app_secret_key,
        })
    }
}
