//! Authentication service

use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::{AppError, AppResult};
use crate::models::{LoginRequest, LoginResponse, UserProfile};
use crate::services::user_service::UserService;

#[derive(Clone)]
pub struct AuthService {
    user_service: UserService,
    jwt_secret: String,
    jwt_expiry_hours: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,  // Subject (user ID)
    exp: usize,   // Expiration time
    iat: usize,   // Issued at
}

impl AuthService {
    pub async fn new(config: &AppConfig, database: Database) -> AppResult<Self> {
        let user_service = UserService::new(config, database).await?;
        Ok(Self {
            user_service,
            jwt_secret: config.auth.jwt_secret.clone(),
            jwt_expiry_hours: config.auth.jwt_expiry_hours,
        })
    }

    /// Login user and return JWT tokens
    pub async fn login(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        // Verify user credentials
        let user = self.user_service.verify_password(&request.email, &request.password).await?;
        
        // Update last login time
        self.user_service.update_last_login(user.id).await?;
        
        // Generate tokens
        let access_token = self.generate_access_token(user.id)?;
        let refresh_token = self.generate_refresh_token(user.id)?;
        
        // Convert to user profile
        let user_profile = UserProfile {
            id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            role: user.role,
            is_admin: user.is_admin,
            permissions: user.permissions,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        };
        
        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in: (self.jwt_expiry_hours * 3600) as i64,
            user: user_profile,
        })
    }

    /// Generate access token
    fn generate_access_token(&self, user_id: Uuid) -> AppResult<String> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::hours(self.jwt_expiry_hours as i64))
            .ok_or_else(|| AppError::Internal("Failed to calculate token expiration".to_string()))?
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(AppError::Jwt)
    }

    /// Generate refresh token (for now, same as access token but with longer expiry)
    fn generate_refresh_token(&self, user_id: Uuid) -> AppResult<String> {
        let expiration = chrono::Utc::now()
            .checked_add_signed(chrono::Duration::days(30))
            .ok_or_else(|| AppError::Internal("Failed to calculate refresh token expiration".to_string()))?
            .timestamp() as usize;

        let claims = Claims {
            sub: user_id.to_string(),
            exp: expiration,
            iat: chrono::Utc::now().timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )
        .map_err(AppError::Jwt)
    }

    /// Verify JWT token
    pub fn verify_token(&self, token: &str) -> AppResult<Uuid> {
        let token_data = jsonwebtoken::decode::<Claims>(
            token,
            &jsonwebtoken::DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &jsonwebtoken::Validation::default(),
        )
        .map_err(AppError::Jwt)?;

        Uuid::parse_str(&token_data.claims.sub)
            .map_err(|e| AppError::Internal(format!("Invalid user ID in token: {}", e)))
    }
}