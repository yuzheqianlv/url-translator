//! Configuration-related database models

use super::*;

/// System configuration stored in database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User configuration preferences
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
pub struct UpdateSystemConfigRequest {
    pub configs: Vec<SystemConfigUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SystemConfigUpdate {
    #[validate(length(min = 1, max = 255))]
    pub key: String,
    
    #[validate(length(min = 1, max = 1000))]
    pub value: String,
    
    #[validate(length(max = 500))]
    pub description: Option<String>,
    
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfigResponse {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfigResponse {
    pub deeplx_api_url: Option<String>,
    pub jina_api_url: String,
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub max_requests_per_second: i32,
    pub max_text_length: i32,
    pub max_paragraphs_per_request: i32,
    pub updated_at: DateTime<Utc>,
}

/// API key management
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyCreatedResponse {
    pub api_key: ApiKeyResponse,
    pub key: String, // Full API key, only shown once
}

/// Application statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatistics {
    pub total_users: i64,
    pub active_users_today: i64,
    pub active_users_week: i64,
    pub total_translations: i64,
    pub translations_today: i64,
    pub translations_week: i64,
    pub total_projects: i64,
    pub average_translation_time_ms: f64,
    pub top_languages: Vec<LanguageStats>,
    pub system_health: SystemHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageStats {
    pub language: String,
    pub count: i64,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub database_status: String,
    pub redis_status: String,
    pub meilisearch_status: String,
    pub overall_status: String,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f32,
}

impl SystemConfig {
    /// Create a new system configuration entry
    pub fn new(key: String, value: String, description: Option<String>, is_public: bool) -> Self {
        Self {
            id: Uuid::new_v4(),
            key,
            value,
            description,
            is_public,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl UserConfig {
    /// Create default user configuration
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

    /// Update user configuration
    pub fn update(&mut self, updates: crate::models::user::UpdateUserConfigRequest) {
        if let Some(url) = updates.deeplx_api_url {
            self.deeplx_api_url = Some(url);
        }
        if let Some(url) = updates.jina_api_url {
            self.jina_api_url = url;
        }
        if let Some(lang) = updates.default_source_lang {
            self.default_source_lang = lang;
        }
        if let Some(lang) = updates.default_target_lang {
            self.default_target_lang = lang;
        }
        if let Some(rate) = updates.max_requests_per_second {
            self.max_requests_per_second = rate;
        }
        if let Some(length) = updates.max_text_length {
            self.max_text_length = length;
        }
        if let Some(paragraphs) = updates.max_paragraphs_per_request {
            self.max_paragraphs_per_request = paragraphs;
        }
        self.updated_at = Utc::now();
    }
}

impl From<SystemConfig> for SystemConfigResponse {
    fn from(config: SystemConfig) -> Self {
        Self {
            key: config.key,
            value: config.value,
            description: config.description,
            is_public: config.is_public,
            updated_at: config.updated_at,
        }
    }
}

impl From<UserConfig> for UserConfigResponse {
    fn from(config: UserConfig) -> Self {
        Self {
            deeplx_api_url: config.deeplx_api_url,
            jina_api_url: config.jina_api_url,
            default_source_lang: config.default_source_lang,
            default_target_lang: config.default_target_lang,
            max_requests_per_second: config.max_requests_per_second,
            max_text_length: config.max_text_length,
            max_paragraphs_per_request: config.max_paragraphs_per_request,
            updated_at: config.updated_at,
        }
    }
}

impl From<ApiKey> for ApiKeyResponse {
    fn from(key: ApiKey) -> Self {
        Self {
            id: key.id,
            name: key.name,
            key_prefix: key.key_prefix,
            permissions: key.permissions,
            expires_at: key.expires_at,
            last_used_at: key.last_used_at,
            is_active: key.is_active,
            created_at: key.created_at,
        }
    }
}