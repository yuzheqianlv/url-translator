//! Search service using MeiliSearch

use meilisearch_sdk::{client::Client, settings::Settings};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::error::{AppError, AppResult};

#[derive(Clone)]
pub struct SearchService {
    client: Client,
    index_prefix: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: String,
    pub user_id: String,
    pub created_at: String,
    pub tags: Vec<String>,
}

impl SearchService {
    pub async fn new(config: &AppConfig) -> AppResult<Self> {
        let client = Client::new(&config.meilisearch.url, Some(&config.meilisearch.api_key))
            .map_err(|e| AppError::Internal(format!("Failed to create MeiliSearch client: {}", e)))?;
        
        Ok(Self {
            client,
            index_prefix: config.meilisearch.index_prefix.clone(),
        })
    }

    /// Initialize search indices
    pub async fn initialize_indices(&self) -> AppResult<()> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        // Configure searchable attributes
        let mut settings = Settings::new();
        settings.searchable_attributes = Some(vec!["title".to_string(), "content".to_string(), "url".to_string()]);
        settings.filterable_attributes = Some(vec!["user_id".to_string(), "tags".to_string()]);
        settings.sortable_attributes = Some(vec!["created_at".to_string()]);
        
        index.set_settings(&settings).await
            .map_err(|e| AppError::MeiliSearch(e.to_string()))?;
            
        Ok(())
    }

    /// Add document to search index
    pub async fn add_document(&self, document: SearchDocument) -> AppResult<()> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        index.add_documents(&[document], Some("id")).await
            .map_err(|e| AppError::MeiliSearch(e.to_string()))?;
            
        Ok(())
    }

    /// Search documents
    pub async fn search(&self, query: &str, user_id: &str, limit: usize) -> AppResult<Vec<SearchDocument>> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        let results = index.search()
            .with_query(query)
            .with_filter(&format!("user_id = '{}'", user_id))
            .with_limit(limit)
            .execute::<SearchDocument>()
            .await
            .map_err(|e| AppError::MeiliSearch(e.to_string()))?;
            
        Ok(results.hits.into_iter().map(|hit| hit.result).collect())
    }

    /// Delete document from search index
    pub async fn delete_document(&self, document_id: &str) -> AppResult<()> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        index.delete_document(document_id).await
            .map_err(|e| AppError::MeiliSearch(e.to_string()))?;
            
        Ok(())
    }

    /// Get search suggestions
    pub async fn get_suggestions(&self, _query: &str, _limit: usize) -> AppResult<Vec<String>> {
        // TODO: Implement search suggestions based on user history
        // For now, return empty suggestions
        Ok(vec![])
    }
}