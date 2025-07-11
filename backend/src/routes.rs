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
        
        // Admin management routes
        .nest("/admin", admin_routes())
        
        // Migration routes (temporary)
        .nest("/migrate", migration_routes())
        
        // Translation routes
        .nest("/translations", translation_routes())
        
        // Search routes
        .nest("/search", search_routes())
        
        // Project management routes
        .nest("/projects", project_routes())
        
        // System and admin routes
        .nest("/system", system_routes())
        
        // WebSocket routes
        .nest("/ws", websocket_routes())
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
        // Single translation (synchronous)
        .route("/translate", post(handlers::translations::translate_url))
        
        // Async translation task endpoints
        .route("/tasks/submit", post(handlers::translations::submit_translation_task))
        .route("/tasks/:task_id/status", get(handlers::translations::get_task_status))
        .route("/tasks/:task_id/cancel", post(handlers::translations::cancel_translation_task))
        .route("/tasks", get(handlers::translations::get_user_tasks))
        
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

/// Admin management routes (requires admin privileges)
fn admin_routes() -> Router<Services> {
    use axum::routing::{get, post, put, delete};
    
    Router::new()
        // User management - admin privileges checked in handlers
        .route("/users", post(handlers::admin::create_admin_user))
        .route("/users", get(handlers::admin::list_users))
        .route("/users/:id/role", put(handlers::admin::update_user_role))
        .route("/users/:id/permissions/:permission", get(handlers::admin::check_user_permission))
        .route("/users/:id", delete(handlers::admin::delete_user))
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
        // Admin privileges checked in handlers
}

/// Migration routes (temporary, for database setup)
fn migration_routes() -> Router<Services> {
    use axum::routing::{get, post};
    
    Router::new()
        .route("/status", get(handlers::migration::check_migration_status))
        .route("/user-roles", post(handlers::migration::migrate_user_roles))
        .route("/upgrade-admin", post(handlers::migration::upgrade_user_to_admin))
}

/// WebSocket routes
fn websocket_routes() -> Router<Services> {
    use axum::routing::get;
    
    Router::new()
        .route("/", get(handlers::websocket::websocket_handler))
}