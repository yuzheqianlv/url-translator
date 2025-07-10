//! URL Translator Backend API Server
//!
//! This is the main entry point for the URL Translator backend service.
//! It provides RESTful APIs for:
//! - User authentication and management
//! - Translation history and projects
//! - Search functionality via MeiliSearch
//! - Caching via Redis
//! - Data persistence via PostgreSQL

mod config;
mod database;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;
mod utils;

use axum::Server;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::AppResult;
use crate::routes::create_router;

#[tokio::main]
async fn main() -> AppResult<()> {
    // Initialize tracing
    init_tracing();

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("Starting URL Translator Backend v{}", env!("CARGO_PKG_VERSION"));
    info!("Server will listen on port {}", config.server.port);

    // Initialize database
    let database = Database::new(&config.database).await?;
    info!("Database connection established");

    // Run database migrations
    database.migrate().await?;
    info!("Database migrations completed");

    // Initialize services
    let services = services::Services::new(&config, database.clone()).await?;
    info!("Services initialized successfully");

    // Create router with middleware
    let app = create_router(services)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()) // Configure CORS as needed
                .layer(middleware::request_id::RequestIdLayer)
                .layer(middleware::rate_limit::create_rate_limit_layer(&config))
        );

    // Create server address
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("Server starting on {}", addr);

    // Start server
    match Server::bind(&addr).serve(app.into_make_service()).await {
        Ok(()) => {
            info!("Server stopped gracefully");
            Ok(())
        }
        Err(e) => {
            warn!("Server error: {}", e);
            Err(e.into())
        }
    }
}

/// Initialize tracing/logging
fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "url_translator_backend=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}