//! Authentication service

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::{AppError, AppResult};
use crate::models::{LoginRequest, LoginResponse, UserProfile};

#[derive(Clone)]
pub struct AuthService {
    database: Database,
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
        Ok(Self {
            database,
            jwt_secret: config.auth.jwt_secret.clone(),
            jwt_expiry_hours: config.auth.jwt_expiry_hours,
        })
    }

    /// Login user and return JWT tokens
    pub async fn login(&self, request: LoginRequest) -> AppResult<LoginResponse> {
        // TODO: Implement user lookup from database
        // let user = self.find_user_by_email(&request.email).await?;
        
        // TODO: Verify password
        // self.verify_password(&request.password, &user.password_hash)?;
        
        // For now, return a mock response
        let user_id = Uuid::new_v4();
        let access_token = self.generate_access_token(user_id)?;
        let refresh_token = self.generate_refresh_token(user_id)?;
        
        // Mock user profile
        let user = UserProfile {
            id: user_id,
            username: "test_user".to_string(),
            email: request.email,
            is_active: true,
            created_at: chrono::Utc::now(),
            last_login_at: Some(chrono::Utc::now()),
        };
        
        Ok(LoginResponse {
            access_token,
            refresh_token,
            expires_in: (self.jwt_expiry_hours * 3600) as i64,
            user,
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

    /// Hash password using Argon2
    pub fn hash_password(&self, password: &str) -> AppResult<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::PasswordHash(e.to_string()))?
            .to_string();
            
        Ok(password_hash)
    }

    /// Verify password against hash
    pub fn verify_password(&self, password: &str, hash: &str) -> AppResult<()> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::PasswordHash(e.to_string()))?;
        
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| AppError::Auth(format!("Invalid password: {}", e)))?;
            
        Ok(())
    }
}