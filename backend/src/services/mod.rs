//! Business logic services

pub mod auth_service;
pub mod redis_service;
pub mod search_service;
pub mod translation_service;
pub mod user_service;

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::AppResult;

/// Container for all application services
#[derive(Clone)]
pub struct Services {
    pub auth_service: auth_service::AuthService,
    pub redis_service: redis_service::RedisService,
    pub user_service: user_service::UserService,
    pub translation_service: translation_service::TranslationService,
    pub search_service: search_service::SearchService,
}

impl Services {
    /// Create new services container
    pub async fn new(config: &AppConfig, database: Database) -> AppResult<Self> {
        let auth_service = auth_service::AuthService::new(config, database.clone()).await?;
        let redis_service = redis_service::RedisService::new(config).await?;
        let user_service = user_service::UserService::new(config, database.clone()).await?;
        let translation_service = translation_service::TranslationService::new(config, database.clone()).await?;
        let search_service = search_service::SearchService::new(config).await?;

        Ok(Self {
            auth_service,
            redis_service,
            user_service,
            translation_service,
            search_service,
        })
    }
}