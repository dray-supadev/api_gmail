use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod middleware;
mod services;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration (Fix Point 5 & 8)
    let config = config::Config::load().expect("Failed to load configuration");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Failed to create reqwest client");
    
    let state = AppState {
        config: config.clone(),
        client,
    };

    // Build application router
    let app = Router::new()
        .route("/health", get(handlers::health::check))
        .route("/api/messages", get(handlers::api::list_messages))
        .route("/api/messages/:id", get(handlers::api::get_message))
        .route("/api/messages/send", post(handlers::api::send_message))
        .route("/api/labels", get(handlers::api::list_labels))
        .route("/api/labels/batch-modify", post(handlers::api::batch_modify_labels))
        .route("/api/profile", get(handlers::api::get_profile))
        .route("/api/quote/preview", post(handlers::api::preview_quote))
        .route("/api/quote/send", post(handlers::api::send_quote_email))
        // Apply Auth Middleware to /api routes (Fix Point 5)
        .route_layer(axum::middleware::from_fn_with_state(state.clone(), middleware::auth::verify_api_key))
        // Explicitly serve embed.js
        .route("/embed.js", get(handlers::api::get_embed_js))
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::compression::CompressionLayer::new())
        // Fix Point 4: More restrictive CORS for production
        .layer(
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any) // Still open for now but can be restricted to specific domains later
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::HeaderName::from_static("x-api-key"), axum::http::header::AUTHORIZATION])
        )
        .fallback_service(
             tower_http::services::ServeDir::new("frontend/dist")
                 .not_found_service(tower_http::services::ServeFile::new("frontend/dist/index.html"))
        )
        .with_state(state);

    // Run server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr_str).await.unwrap();
    
    tracing::info!("listening on {}", addr_str);
    axum::serve(listener, app).await.unwrap();
}
