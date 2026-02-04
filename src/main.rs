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

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let _config = config::Config::load().expect("Failed to load configuration");

    // Build application router
    let app = Router::new()
        .route("/health", get(handlers::health::check))
        .route("/api/messages", get(handlers::api::list_messages))
        .route("/api/messages/:id", get(handlers::api::get_message))
        .route("/api/messages/send", post(handlers::api::send_message))
        .route("/api/threads/:thread_id", get(handlers::api::get_thread))
        .route("/api/quote/preview", get(handlers::api::preview_quote))
        .route("/api/quote/send", post(handlers::api::send_quote_email))
        // Explicitly serve embed.js to diagnose ServeDir issues
        .route("/embed.js", axum::routing::get_service(tower_http::services::ServeFile::new("frontend/dist/embed.js")))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()) // Customize this for production security
        .fallback_service(
             tower_http::services::ServeDir::new("frontend/dist")
                 .not_found_service(tower_http::services::ServeFile::new("frontend/dist/index.html"))
        );

    // Run server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr_str).await.unwrap();
    
    tracing::info!("listening on {}", addr_str);
    axum::serve(listener, app).await.unwrap();
}
