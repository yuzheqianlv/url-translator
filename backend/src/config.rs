//! Configuration management for the URL Translator backend

use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub meilisearch: MeiliSearchConfig,
    pub auth: AuthConfig,
    pub rate_limiting: RateLimitingConfig,
    pub translation: TranslationConfig,
    pub api: ApiConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub workers: usize,
    pub max_connections: usize,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: usize,
    pub connect_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
    pub default_ttl_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeiliSearchConfig {
    pub url: String,
    pub api_key: String,
    pub connect_timeout_seconds: u64,
    pub request_timeout_seconds: u64,
    pub index_prefix: String,
    pub max_search_results: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub password_min_length: usize,
    pub max_login_attempts: u32,
    pub lockout_duration_minutes: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitingConfig {
    pub requests_per_minute: u64,
    pub burst_size: u32,
    pub window_size_seconds: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationConfig {
    pub max_text_length: usize,
    pub max_batch_size: usize,
    pub default_timeout_seconds: u64,
    pub max_concurrent_requests: usize,
    pub max_paragraphs_per_request: usize,
    pub max_requests_per_second: usize,
    pub deeplx_api_url: String,
    pub jina_api_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    pub max_file_size_mb: usize,
    pub allowed_file_types: Vec<String>,
    pub default_page_size: usize,
    pub max_page_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_rotation: String,
    pub max_log_files: usize,
}

impl AppConfig {
    /// Load configuration from environment variables and config files
    pub fn from_env() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());
        
        let s = Config::builder()
            // Start with default configuration
            .add_source(File::with_name("config/default"))
            // Add environment-specific configuration if available
            .add_source(
                File::with_name(&format!("config/{}", run_mode))
                    .required(false)
            )
            // Add local configuration if available
            .add_source(File::with_name("config/local").required(false))
            // Override with environment variables
            .add_source(
                Environment::with_prefix("APP")
                    .separator("_")
                    .try_parsing(true)
            )
            .build()?;

        let mut config: AppConfig = s.try_deserialize()?;
        
        // Override sensitive values from environment variables
        config.database.url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| config.database.url);
        config.redis.url = env::var("REDIS_URL")
            .unwrap_or_else(|_| config.redis.url);
        config.meilisearch.url = env::var("MEILISEARCH_URL")
            .unwrap_or_else(|_| config.meilisearch.url);
        config.meilisearch.api_key = env::var("MEILISEARCH_API_KEY")
            .unwrap_or_else(|_| config.meilisearch.api_key);
        config.auth.jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| config.auth.jwt_secret);
        
        // Override port from environment
        if let Ok(port) = env::var("PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.server.port = port_num;
            }
        }
        
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.auth.jwt_secret.is_empty() {
            return Err("JWT secret cannot be empty".to_string());
        }
        
        if self.auth.jwt_secret.len() < 32 {
            return Err("JWT secret must be at least 32 characters long".to_string());
        }
        
        if self.database.url.is_empty() {
            return Err("Database URL cannot be empty".to_string());
        }
        
        if self.redis.url.is_empty() {
            return Err("Redis URL cannot be empty".to_string());
        }
        
        if self.meilisearch.url.is_empty() {
            return Err("MeiliSearch URL cannot be empty".to_string());
        }
        
        if self.meilisearch.api_key.is_empty() {
            return Err("MeiliSearch API key cannot be empty".to_string());
        }
        
        Ok(())
    }
}