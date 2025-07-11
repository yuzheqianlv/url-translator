//! Authentication handlers

use axum::{extract::State, response::Json};
use serde_json::{json, Value};

use crate::error::{AppError, AppResult};
use crate::models::{CreateUserRequest, LoginRequest, LoginResponse};
use validator::Validate;
use crate::services::Services;

/// Register a new user
pub async fn register(
    State(services): State<Services>,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Json<Value>> {
    // Validate input
    request.validate()?;
    
    // Create user through service
    let user = services.user_service.create_user(request).await?;
    
    Ok(Json(json!({
        "message": "User created successfully",
        "user": user
    })))
}

/// Login user
pub async fn login(
    State(services): State<Services>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    // Validate input
    request.validate()?;
    
    // Authenticate user
    let response = services.auth_service.login(request).await?;
    
    Ok(Json(response))
}

/// Refresh access token
pub async fn refresh_token(
    State(_services): State<Services>,
) -> AppResult<Json<Value>> {
    // TODO: Implement token refresh logic
    Err(AppError::Internal("Token refresh not implemented".to_string()))
}

/// Logout user
pub async fn logout(
    State(_services): State<Services>,
) -> AppResult<Json<Value>> {
    // TODO: Implement logout logic (invalidate token)
    Ok(Json(json!({
        "message": "Logged out successfully"
    })))
}

/// Forgot password
pub async fn forgot_password(
    State(_services): State<Services>,
) -> AppResult<Json<Value>> {
    // TODO: Implement forgot password logic
    Err(AppError::Internal("Forgot password not implemented".to_string()))
}

/// Reset password
pub async fn reset_password(
    State(_services): State<Services>,
) -> AppResult<Json<Value>> {
    // TODO: Implement reset password logic
    Err(AppError::Internal("Reset password not implemented".to_string()))
}