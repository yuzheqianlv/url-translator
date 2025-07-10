//! Database connection and management for PostgreSQL

use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::{Executor, Postgres, Transaction};
use std::time::Duration;
use tracing::{info, warn};

use crate::config::DatabaseConfig;
use crate::error::{AppError, AppResult};

#[derive(Debug, Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Create a new database connection pool
    pub async fn new(config: &DatabaseConfig) -> AppResult<Self> {
        info!("Connecting to database...");
        
        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_seconds))
            .max_lifetime(Duration::from_secs(config.max_lifetime_seconds))
            .idle_timeout(Duration::from_secs(config.idle_timeout_seconds))
            .connect(&config.url)
            .await
            .map_err(|e| {
                warn!("Failed to connect to database: {}", e);
                AppError::Database(e)
            })?;
        
        info!("Database connection pool created successfully");
        Ok(Self { pool })
    }
    
    /// Get a reference to the connection pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    /// Run database migrations
    pub async fn migrate(&self) -> AppResult<()> {
        info!("Running database migrations...");
        
        sqlx::migrate!("../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| {
                warn!("Failed to run migrations: {}", e);
                AppError::Database(e)
            })?;
        
        info!("Database migrations completed successfully");
        Ok(())
    }
    
    /// Begin a new transaction
    pub async fn begin_transaction(&self) -> AppResult<Transaction<'_, Postgres>> {
        self.pool.begin().await.map_err(AppError::Database)
    }
    
    /// Execute a SQL statement
    pub async fn execute(&self, query: &str) -> AppResult<sqlx::postgres::PgQueryResult> {
        self.pool.execute(query).await.map_err(AppError::Database)
    }
    
    /// Test database connection
    pub async fn health_check(&self) -> AppResult<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)
            .map(|_| ())
    }
    
    /// Get database connection statistics
    pub fn connection_info(&self) -> ConnectionInfo {
        ConnectionInfo {
            size: self.pool.size(),
            idle: self.pool.num_idle(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub size: u32,
    pub idle: usize,
}