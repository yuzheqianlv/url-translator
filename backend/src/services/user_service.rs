//! User management service

use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::Utc;
use sqlx::Row;

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::{AppError, AppResult};
use crate::models::{CreateUserRequest, UserProfile, User};

#[derive(Clone)]
pub struct UserService {
    database: Database,
}

impl UserService {
    pub async fn new(_config: &AppConfig, database: Database) -> AppResult<Self> {
        Ok(Self { database })
    }

    /// Create a new user
    pub async fn create_user(&self, request: CreateUserRequest) -> AppResult<UserProfile> {
        // Check if user already exists
        if self.user_exists_by_email(&request.email).await? {
            return Err(AppError::Conflict("用户邮箱已存在".to_string()));
        }

        if self.user_exists_by_username(&request.username).await? {
            return Err(AppError::Conflict("用户名已存在".to_string()));
        }

        // Hash password
        let password_hash = hash(&request.password, DEFAULT_COST)
            .map_err(|e| AppError::Internal(format!("密码哈希失败: {}", e)))?;

        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert user into database
        let query = r#"
            INSERT INTO users (id, username, email, password_hash, is_active, role, is_admin, permissions, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#;

        sqlx::query(query)
            .bind(&user_id)
            .bind(&request.username)
            .bind(&request.email)
            .bind(&password_hash)
            .bind(true)
            .bind(&crate::models::user::UserRole::User)
            .bind(false)
            .bind(&Vec::<String>::new())
            .bind(&now)
            .bind(&now)
            .execute(self.database.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        // Create default user configuration
        self.create_default_user_config(user_id).await?;

        Ok(UserProfile {
            id: user_id,
            username: request.username,
            email: request.email,
            is_active: true,
            role: crate::models::user::UserRole::User,
            is_admin: false,
            permissions: vec![],
            created_at: now,
            last_login_at: None,
        })
    }

    /// Get user by ID
    pub async fn get_user(&self, _user_id: Uuid) -> AppResult<UserProfile> {
        // TODO: Implement database lookup
        Err(crate::error::AppError::Internal("Get user not implemented".to_string()))
    }

    /// Update user profile
    pub async fn update_user(&self, _user_id: Uuid, _updates: serde_json::Value) -> AppResult<UserProfile> {
        // TODO: Implement user update
        Err(crate::error::AppError::Internal("Update user not implemented".to_string()))
    }

    /// Delete user
    pub async fn delete_user(&self, _user_id: Uuid) -> AppResult<()> {
        // TODO: Implement user deletion
        Err(crate::error::AppError::Internal("Delete user not implemented".to_string()))
    }

    /// Check if user exists by email
    pub async fn user_exists_by_email(&self, email: &str) -> AppResult<bool> {
        let query = "SELECT COUNT(*) as count FROM users WHERE email = $1";
        let row = sqlx::query(query)
            .bind(email)
            .fetch_one(self.database.pool())
            .await
            .map_err(|e| AppError::Database(e))?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Check if user exists by username
    pub async fn user_exists_by_username(&self, username: &str) -> AppResult<bool> {
        let query = "SELECT COUNT(*) as count FROM users WHERE username = $1";
        let row = sqlx::query(query)
            .bind(username)
            .fetch_one(self.database.pool())
            .await
            .map_err(|e| AppError::Database(e))?;
        
        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Get user by email for authentication
    pub async fn get_user_by_email(&self, email: &str) -> AppResult<User> {
        let query = r#"
            SELECT id, username, email, password_hash, is_active, role, is_admin, permissions, created_at, updated_at, last_login_at
            FROM users 
            WHERE email = $1 AND is_active = true
        "#;
        
        sqlx::query_as::<_, User>(query)
            .bind(email)
            .fetch_one(self.database.pool())
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("用户不存在".to_string()),
                _ => AppError::Database(e),
            })
    }

    /// Verify user password
    pub async fn verify_password(&self, email: &str, password: &str) -> AppResult<User> {
        let user = self.get_user_by_email(email).await?;
        
        if verify(password, &user.password_hash)
            .map_err(|e| AppError::Internal(format!("密码验证失败: {}", e)))? {
            Ok(user)
        } else {
            Err(AppError::BadRequest("用户名或密码错误".to_string()))
        }
    }

    /// Update user last login time
    pub async fn update_last_login(&self, user_id: Uuid) -> AppResult<()> {
        let query = "UPDATE users SET last_login_at = $1 WHERE id = $2";
        sqlx::query(query)
            .bind(Utc::now())
            .bind(user_id)
            .execute(self.database.pool())
            .await
            .map_err(|e| AppError::Database(e))?;
        
        Ok(())
    }

    /// Create default user configuration
    async fn create_default_user_config(&self, user_id: Uuid) -> AppResult<()> {
        let config_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO user_configs (
                id, user_id, deeplx_api_url, jina_api_url, 
                default_source_lang, default_target_lang,
                max_requests_per_second, max_text_length, max_paragraphs_per_request,
                created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#;

        sqlx::query(query)
            .bind(&config_id)
            .bind(&user_id)
            .bind::<Option<String>>(None) // deeplx_api_url
            .bind("https://r.jina.ai") // jina_api_url
            .bind("auto") // default_source_lang
            .bind("zh") // default_target_lang
            .bind(10i32) // max_requests_per_second
            .bind(5000i32) // max_text_length
            .bind(10i32) // max_paragraphs_per_request
            .bind(&now)
            .bind(&now)
            .execute(self.database.pool())
            .await
            .map_err(|e| AppError::Database(e))?;

        Ok(())
    }
}