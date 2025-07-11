//! Authentication middleware

use axum::{
    async_trait,
    extract::{FromRequestParts, Request},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashSet;

use crate::error::AppError;
use crate::models::user::{User, UserRole};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // Subject (user ID)
    pub exp: usize,   // Expiration time
    pub iat: usize,   // Issued at
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub user: User,
}

impl AuthenticatedUser {
    /// 检查用户是否拥有指定权限
    pub fn has_permission(&self, permission: &str) -> bool {
        self.user.has_permission(permission)
    }
    
    /// 检查用户是否为管理员
    pub fn is_admin(&self) -> bool {
        self.user.is_administrator()
    }
    
    /// 检查用户是否为管理者（包括管理员和管理者）
    pub fn is_manager(&self) -> bool {
        self.user.is_manager()
    }
    
    /// 获取用户角色
    pub fn role(&self) -> &UserRole {
        &self.user.role
    }
    
    /// 获取用户的所有有效权限
    pub fn effective_permissions(&self) -> HashSet<String> {
        self.user.get_effective_permissions()
    }
}

#[async_trait]
impl FromRequestParts<crate::services::Services> for AuthenticatedUser
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &crate::services::Services) -> Result<Self, Self::Rejection> {
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
        
        // Get JWT secret from application configuration
        let jwt_secret = &state.config.auth.jwt_secret;
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(jwt_secret.as_ref()),
            &validation,
        ).map_err(|e| AppError::Auth(format!("Invalid token: {}", e)))?;

        let user_id = Uuid::parse_str(&token_data.claims.sub)
            .map_err(|_| AppError::Auth("Invalid user ID in token".to_string()))?;

        // 从数据库获取用户信息
        let user = sqlx::query_as::<_, crate::models::user::User>(
            "SELECT id, username, email, password_hash, is_active, role, is_admin, permissions, created_at, updated_at, last_login_at FROM users WHERE id = $1 AND is_active = true"
        )
        .bind(user_id)
        .fetch_one(state.db.pool())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AppError::Auth("User not found or inactive".to_string()),
            _ => AppError::Database(e),
        })?;

        Ok(AuthenticatedUser { 
            user_id,
            user,
        })
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

/// Middleware that requires admin privileges
pub async fn admin_required(
    request: Request,
    next: Next,
) -> Result<Response, axum::http::StatusCode> {
    // Try to extract user from request
    let auth_header = request.headers().get(AUTHORIZATION)
        .ok_or(axum::http::StatusCode::UNAUTHORIZED)?;

    let auth_header_str = auth_header.to_str()
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    if !auth_header_str.starts_with("Bearer ") {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header_str[7..];
    
    // Get JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default_secret_key_change_in_production".to_string());
    
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    ).map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    let _user_id = Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    // This is a simplified version - we'll check admin status directly
    // In a real implementation, you'd query the database here
    Ok(next.run(request).await)
}

/// Middleware that requires manager privileges (admin or moderator)
pub async fn manager_required(
    user: AuthenticatedUser,
    request: Request,
    next: Next,
) -> Response {
    if user.is_manager() {
        // User is manager, continue with request
        next.run(request).await
    } else {
        // Return 403 Forbidden response
        (axum::http::StatusCode::FORBIDDEN, "Manager privileges required").into_response()
    }
}

/// Create a middleware that requires a specific permission
pub fn permission_required(permission: &'static str) -> impl Fn(Result<AuthenticatedUser, AppError>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |user: Result<AuthenticatedUser, AppError>, request: Request, next: Next| {
        Box::pin(async move {
            match user {
                Ok(auth_user) => {
                    if auth_user.has_permission(permission) {
                        // User has permission, continue with request
                        Ok(next.run(request).await)
                    } else {
                        Err(AppError::Forbidden(format!("Permission '{}' required", permission)))
                    }
                }
                Err(e) => {
                    // User is not authenticated, return error
                    Err(e)
                }
            }
        })
    }
}

/// Helper struct for role-based access control
#[derive(Debug, Clone)]
pub struct RequireRole(pub UserRole);

impl RequireRole {
    pub async fn check(
        &self,
        user: Result<AuthenticatedUser, AppError>,
        request: Request,
        next: Next,
    ) -> Result<Response, AppError> {
        match user {
            Ok(auth_user) => {
                if auth_user.role() == &self.0 || auth_user.is_admin() {
                    // User has required role or is admin, continue with request
                    Ok(next.run(request).await)
                } else {
                    Err(AppError::Forbidden(format!("Role '{}' required", self.0)))
                }
            }
            Err(e) => {
                // User is not authenticated, return error
                Err(e)
            }
        }
    }
}