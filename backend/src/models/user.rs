//! User-related database models

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserConfig {
    pub id: Uuid,
    pub user_id: Uuid,
    pub deeplx_api_url: Option<String>,
    pub jina_api_url: String,
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub max_requests_per_second: i32,
    pub max_text_length: i32,
    pub max_paragraphs_per_request: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserConfigRequest {
    pub deeplx_api_url: Option<String>,
    
    #[validate(url)]
    pub jina_api_url: Option<String>,
    
    #[validate(length(min = 2, max = 10))]
    pub default_source_lang: Option<String>,
    
    #[validate(length(min = 2, max = 10))]
    pub default_target_lang: Option<String>,
    
    #[validate(range(min = 1, max = 100))]
    pub max_requests_per_second: Option<i32>,
    
    #[validate(range(min = 100, max = 100000))]
    pub max_text_length: Option<i32>,
    
    #[validate(range(min = 1, max = 50))]
    pub max_paragraphs_per_request: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_translations: i64,
    pub total_projects: i64,
    pub total_characters_translated: i64,
    pub average_translation_time_ms: Option<f64>,
    pub most_used_source_language: Option<String>,
    pub most_used_target_language: Option<String>,
    pub translations_this_month: i64,
    pub translations_this_week: i64,
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

impl UserConfig {
    pub fn default_for_user(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            deeplx_api_url: None,
            jina_api_url: "https://r.jina.ai".to_string(),
            default_source_lang: "auto".to_string(),
            default_target_lang: "zh".to_string(),
            max_requests_per_second: 10,
            max_text_length: 5000,
            max_paragraphs_per_request: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}