//! Error handling for the URL Translator backend

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("MeiliSearch error: {0}")]
    MeiliSearch(String),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    
    #[error("Password hashing error: {0}")]
    PasswordHash(String),
    
    #[error("UUID parsing error: {0}")]
    UuidParse(#[from] uuid::Error),
    
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::Validation(_) | AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            AppError::Database(_) 
            | AppError::Redis(_) 
            | AppError::MeiliSearch(_)
            | AppError::Config(_)
            | AppError::Internal(_)
            | AppError::ExternalService(_)
            | AppError::Io(_)
            | AppError::Json(_)
            | AppError::HttpClient(_)
            | AppError::Jwt(_)
            | AppError::PasswordHash(_)
            | AppError::UuidParse(_)
            | AppError::UrlParse(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// Get a user-friendly error message
    pub fn user_message(&self) -> &str {
        match self {
            AppError::Auth(_) => "Authentication failed",
            AppError::Validation(_) => "Invalid input provided",
            AppError::NotFound(_) => "Resource not found",
            AppError::Conflict(_) => "Resource already exists",
            AppError::Forbidden(_) => "Access forbidden",
            AppError::RateLimitExceeded => "Rate limit exceeded. Please try again later",
            AppError::BadRequest(_) => "Bad request",
            AppError::ExternalService(_) => "External service temporarily unavailable",
            _ => "Internal server error",
        }
    }
    
    /// Check if this error should be logged
    pub fn should_log(&self) -> bool {
        !matches!(self, 
            AppError::Auth(_) 
            | AppError::Validation(_) 
            | AppError::NotFound(_) 
            | AppError::BadRequest(_)
            | AppError::Forbidden(_)
            | AppError::RateLimitExceeded
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let user_message = self.user_message();
        
        // Log error if it's internal
        if self.should_log() {
            tracing::error!("Internal error: {}", self);
        } else {
            tracing::warn!("Client error: {}", self);
        }
        
        let body = Json(json!({
            "error": {
                "code": status.as_u16(),
                "message": user_message,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        }));
        
        (status, body).into_response()
    }
}

// Convert validation errors
impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::Validation(format!("Validation failed: {}", err))
    }
}

