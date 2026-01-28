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
    let config = config::Config::load().expect("Failed to load configuration");

    // Build application router
    let app = Router::new()
        .route("/health", get(handlers::health::check))
        .route("/api/messages", get(handlers::gmail::list_messages))
        .route("/api/messages/:id", get(handlers::gmail::get_message))
        .route("/api/messages/send", post(handlers::gmail::send_message))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()) // Customize this for production security
        .layer(axum::middleware::from_fn_with_state(
            config.clone(),
            middleware::auth::verify_api_key,
        ));

    // Run server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr_str = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr_str).await.unwrap();
    
    tracing::info!("listening on {}", addr_str);
    axum::serve(listener, app).await.unwrap();
}
