//! Redis caching service

use redis::{Client, RedisResult};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct RedisService {
    client: Client,
    default_ttl: Duration,
}

impl RedisService {
    /// Create a new Redis service
    pub async fn new(config: &AppConfig) -> AppResult<Self> {
        let client = Client::open(config.redis.url.as_str())
            .map_err(|e| AppError::Redis(e))?;
        
        // Test connection
        let mut conn = client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        redis::cmd("PING")
            .query::<String>(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        info!("Redis connection established successfully");
        
        Ok(Self {
            client,
            default_ttl: Duration::from_secs(config.redis.default_ttl_seconds),
        })
    }

    /// Get a value from Redis
    pub async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: RedisResult<String> = redis::cmd("GET")
            .arg(key)
            .query(&mut conn);
        
        match result {
            Ok(value) => {
                let deserialized: T = serde_json::from_str(&value)
                    .map_err(|e| AppError::Json(e))?;
                Ok(Some(deserialized))
            }
            Err(e) if e.kind() == redis::ErrorKind::TypeError => {
                // Key doesn't exist
                Ok(None)
            }
            Err(e) => {
                error!("Redis GET error for key '{}': {}", key, e);
                Err(AppError::Redis(e))
            }
        }
    }

    /// Set a value in Redis with default TTL
    pub async fn set<T>(&self, key: &str, value: &T) -> AppResult<()>
    where
        T: Serialize,
    {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value in Redis with custom TTL
    pub async fn set_with_ttl<T>(&self, key: &str, value: &T, ttl: Duration) -> AppResult<()>
    where
        T: Serialize,
    {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::Json(e))?;
        
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl.as_secs())
            .arg(serialized)
            .query::<()>(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(())
    }

    /// Delete a key from Redis
    pub async fn delete(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: i32 = redis::cmd("DEL")
            .arg(key)
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(result > 0)
    }

    /// Check if a key exists in Redis
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: i32 = redis::cmd("EXISTS")
            .arg(key)
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(result > 0)
    }

    /// Set TTL for an existing key
    pub async fn expire(&self, key: &str, ttl: Duration) -> AppResult<bool> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: i32 = redis::cmd("EXPIRE")
            .arg(key)
            .arg(ttl.as_secs())
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(result > 0)
    }

    /// Get multiple keys at once
    pub async fn mget<T>(&self, keys: &[&str]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if keys.is_empty() {
            return Ok(vec![]);
        }

        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(keys)
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        let mut deserialized = Vec::new();
        for result in results {
            match result {
                Some(value) => {
                    let item: T = serde_json::from_str(&value)
                        .map_err(|e| AppError::Json(e))?;
                    deserialized.push(Some(item));
                }
                None => deserialized.push(None),
            }
        }
        
        Ok(deserialized)
    }

    /// Set multiple key-value pairs
    pub async fn mset<T>(&self, pairs: &[(&str, &T)]) -> AppResult<()>
    where
        T: Serialize,
    {
        if pairs.is_empty() {
            return Ok(());
        }

        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let mut cmd = redis::cmd("MSET");
        for (key, value) in pairs {
            let serialized = serde_json::to_string(value)
                .map_err(|e| AppError::Json(e))?;
            cmd.arg(*key).arg(serialized);
        }
        
        cmd.query::<()>(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(())
    }

    /// Increment a counter
    pub async fn incr(&self, key: &str) -> AppResult<i64> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: i64 = redis::cmd("INCR")
            .arg(key)
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(result)
    }

    /// Decrement a counter
    pub async fn decr(&self, key: &str) -> AppResult<i64> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: i64 = redis::cmd("DECR")
            .arg(key)
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(result)
    }

    /// Clear all keys (use with caution!)
    pub async fn flush_all(&self) -> AppResult<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        redis::cmd("FLUSHALL")
            .query::<()>(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(())
    }

    /// Get Redis info
    pub async fn info(&self) -> AppResult<String> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        let result: String = redis::cmd("INFO")
            .query(&mut conn)
            .map_err(|e| AppError::Redis(e))?;
        
        Ok(result)
    }

    /// Health check for Redis
    pub async fn health_check(&self) -> AppResult<()> {
        let mut conn = self.client.get_connection()
            .map_err(|e| AppError::Redis(e))?;
        
        redis::cmd("PING")
            .query::<String>(&mut conn)
            .map_err(|e| AppError::Redis(e))
            .map(|_| ())
    }

    /// Get Redis connection for async operations
    pub async fn get_async_connection(&self) -> AppResult<redis::aio::Connection> {
        let conn = self.client.get_async_connection()
            .await
            .map_err(|e| AppError::Redis(e))?;
        Ok(conn)
    }

    /// Get Redis client for pub/sub operations
    pub fn get_client(&self) -> Client {
        self.client.clone()
    }
}

/// Helper functions for creating cache keys
pub mod cache_keys {
    use uuid::Uuid;

    /// User profile cache key
    pub fn user_profile(user_id: Uuid) -> String {
        format!("user:profile:{}", user_id)
    }

    /// User config cache key
    pub fn user_config(user_id: Uuid) -> String {
        format!("user:config:{}", user_id)
    }

    /// Translation cache key
    pub fn translation(translation_id: Uuid) -> String {
        format!("translation:{}", translation_id)
    }

    /// User translations list cache key
    pub fn user_translations(user_id: Uuid, page: u32, per_page: u32) -> String {
        format!("user:{}:translations:{}:{}", user_id, page, per_page)
    }

    /// Project cache key
    pub fn project(project_id: Uuid) -> String {
        format!("project:{}", project_id)
    }

    /// User projects list cache key
    pub fn user_projects(user_id: Uuid) -> String {
        format!("user:{}:projects", user_id)
    }

    /// Search results cache key
    pub fn search_results(query: &str, user_id: Uuid, page: u32) -> String {
        use base64::{Engine as _, engine::general_purpose};
        format!("search:{}:{}:{}", 
            general_purpose::STANDARD.encode(query), 
            user_id, 
            page
        )
    }

    /// System config cache key
    pub fn system_config(key: &str) -> String {
        format!("system:config:{}", key)
    }

    /// API rate limit key
    pub fn rate_limit(key: &str) -> String {
        format!("rate_limit:{}", key)
    }
}