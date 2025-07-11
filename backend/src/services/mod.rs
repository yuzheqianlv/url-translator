//! Business logic services

pub mod auth_service;
pub mod event_listener;
pub mod redis_service;
pub mod search_service;
pub mod task_queue;
pub mod translation_service;
pub mod user_service;

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::AppResult;
use crate::handlers::websocket::WebSocketManager;

/// Container for all application services
#[derive(Clone)]
pub struct Services {
    pub config: AppConfig,
    pub db: Database,
    pub auth_service: auth_service::AuthService,
    pub redis_service: redis_service::RedisService,
    pub user_service: user_service::UserService,
    pub translation_service: translation_service::TranslationService,
    pub search_service: search_service::SearchService,
    pub task_queue: task_queue::TaskQueueService,
    pub websocket_manager: WebSocketManager,
}

impl Services {
    /// Create new services container
    pub async fn new(config: &AppConfig, database: Database) -> AppResult<Self> {
        let auth_service = auth_service::AuthService::new(config, database.clone()).await?;
        let redis_service = redis_service::RedisService::new(config).await?;
        let user_service = user_service::UserService::new(config, database.clone()).await?;
        let search_service = search_service::SearchService::new(config).await?;
        
        // Initialize translation service with search service for automatic indexing
        let translation_service = translation_service::TranslationService::new(config, database.clone())
            .await?
            .with_search_service(search_service.clone());
            
        let task_queue = task_queue::TaskQueueService::new(redis_service.clone());
        let websocket_manager = WebSocketManager::new();
        
        // 启动事件监听器
        let event_listener = event_listener::EventListener::new(redis_service.clone(), websocket_manager.clone());
        tokio::spawn(async move {
            event_listener.start().await;
        });

        Ok(Self {
            config: config.clone(),
            db: database,
            auth_service,
            redis_service,
            user_service,
            translation_service,
            search_service,
            task_queue,
            websocket_manager,
        })
    }
}