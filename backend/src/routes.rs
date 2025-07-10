//! API routes for the URL Translator backend

use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::handlers;
use crate::services::Services;

/// Create the main application router with all routes
pub fn create_router(services: Services) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health::health_check))
        
        // API routes
        .nest("/api/v1", api_routes())
        
        // Add services as application state
        .with_state(services)
        
        // Add CORS layer
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive()) // Configure as needed
        )
}

/// API v1 routes
fn api_routes() -> Router<Services> {
    Router::new()
        // Authentication routes
        .nest("/auth", auth_routes())
        
        // User management routes
        .nest("/users", user_routes())
        
        // Translation routes
        .nest("/translations", translation_routes())
        
        // Search routes
        .nest("/search", search_routes())
        
        // Project management routes
        .nest("/projects", project_routes())
        
        // System and admin routes
        .nest("/system", system_routes())
}

/// Authentication routes
fn auth_routes() -> Router<Services> {
    use axum::routing::post;
    
    Router::new()
        .route("/register", post(handlers::auth::register))
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh_token))
        .route("/logout", post(handlers::auth::logout))
        .route("/forgot-password", post(handlers::auth::forgot_password))
        .route("/reset-password", post(handlers::auth::reset_password))
}

/// User management routes
fn user_routes() -> Router<Services> {
    use axum::routing::{get, put, delete};
    
    Router::new()
        .route("/profile", get(handlers::users::get_profile))
        .route("/profile", put(handlers::users::update_profile))
        .route("/profile", delete(handlers::users::delete_profile))
        .route("/config", get(handlers::users::get_config))
        .route("/config", put(handlers::users::update_config))
        .route("/stats", get(handlers::users::get_stats))
}

/// Translation routes
fn translation_routes() -> Router<Services> {
    use axum::routing::{get, post, delete};
    
    Router::new()
        // Single translation
        .route("/translate", post(handlers::translations::translate_url))
        
        // Translation history
        .route("/history", get(handlers::translations::get_history))
        .route("/history/:id", get(handlers::translations::get_translation))
        .route("/history/:id", delete(handlers::translations::delete_translation))
        .route("/history/:id/download", get(handlers::translations::download_translation))
        
        // Batch operations
        .route("/batch", post(handlers::translations::start_batch_translation))
        .route("/batch/:id/status", get(handlers::translations::get_batch_status))
        .route("/batch/:id/cancel", post(handlers::translations::cancel_batch))
}

/// Search routes
fn search_routes() -> Router<Services> {
    use axum::routing::{get, post};
    
    Router::new()
        .route("/", get(handlers::search::search_translations))
        .route("/suggestions", get(handlers::search::get_search_suggestions))
        .route("/history", get(handlers::search::get_search_history))
        .route("/reindex", post(handlers::search::reindex_content))
}

/// Project management routes
fn project_routes() -> Router<Services> {
    use axum::routing::{get, post, put, delete};
    
    Router::new()
        .route("/", get(handlers::projects::list_projects))
        .route("/", post(handlers::projects::create_project))
        .route("/:id", get(handlers::projects::get_project))
        .route("/:id", put(handlers::projects::update_project))
        .route("/:id", delete(handlers::projects::delete_project))
        .route("/:id/urls", get(handlers::projects::get_project_urls))
        .route("/:id/urls", post(handlers::projects::add_urls_to_project))
        .route("/:id/export", get(handlers::projects::export_project))
}

/// System and admin routes
fn system_routes() -> Router<Services> {
    use axum::routing::{get, post};
    
    Router::new()
        .route("/config", get(handlers::system::get_system_config))
        .route("/config", post(handlers::system::update_system_config))
        .route("/stats", get(handlers::system::get_system_stats))
        .route("/metrics", get(handlers::system::get_metrics))
        .route("/cache/clear", post(handlers::system::clear_cache))
}