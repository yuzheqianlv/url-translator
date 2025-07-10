//! Authentication middleware

use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .ok_or_else(|| AppError::Auth("Missing authorization header".to_string()))?;

        let auth_header_str = auth_header
            .to_str()
            .map_err(|_| AppError::Auth("Invalid authorization header".to_string()))?;

        if !auth_header_str.starts_with("Bearer ") {
            return Err(AppError::Auth("Invalid authorization header format".to_string()));
        }

        let token = &auth_header_str[7..];
        
        // TODO: Get JWT secret from application state
        let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_secret_key_change_in_production".to_string());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &validation,
        ).map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Invalid user ID in token".to_string()))?;

        Ok(AuthenticatedUser { user_id })
    }
}

/// Middleware that requires authentication
pub async fn auth_required(
    user: Result<AuthenticatedUser, AppError>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    match user {
        Ok(_) => {
            // User is authenticated, continue with request
            Ok(next.run(request).await)
        }
        Err(e) => {
            // User is not authenticated, return error
            Err(e)
        }
    }
}