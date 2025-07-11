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
    pub original_content: String,
    pub translated_content: String,
    pub url: String,
    pub user_id: String,
    pub source_lang: String,
    pub target_lang: String,
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
        
        // Configure searchable attributes - include both original and translated content
        let mut settings = Settings::new();
        settings.searchable_attributes = Some(vec![
            "title".to_string(),
            "original_content".to_string(),
            "translated_content".to_string(),
            "content".to_string(), // Combined content for backward compatibility
            "url".to_string(),
            "tags".to_string(),
        ]);
        settings.filterable_attributes = Some(vec![
            "user_id".to_string(),
            "source_lang".to_string(),
            "target_lang".to_string(),
            "tags".to_string(),
            "created_at".to_string(),
        ]);
        settings.sortable_attributes = Some(vec![
            "created_at".to_string(),
            "title".to_string(),
        ]);
        
        // Configure displayed attributes for search results
        settings.displayed_attributes = Some(vec![
            "id".to_string(),
            "title".to_string(),
            "content".to_string(),
            "original_content".to_string(),
            "translated_content".to_string(),
            "url".to_string(),
            "source_lang".to_string(),
            "target_lang".to_string(),
            "created_at".to_string(),
            "tags".to_string(),
        ]);
        
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

    /// Search documents with enhanced options
    pub async fn search(&self, query: &str, user_id: &str, limit: usize) -> AppResult<Vec<SearchDocument>> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        let results = index.search()
            .with_query(query)
            .with_filter(&format!("user_id = '{}'", user_id))
            .with_limit(limit)
            .with_attributes_to_highlight(&["title", "original_content", "translated_content"])
            .with_attributes_to_crop(&["original_content", "translated_content"])
            .with_crop_length(200)
            .execute::<SearchDocument>()
            .await
            .map_err(|e| AppError::MeiliSearch(e.to_string()))?;
            
        Ok(results.hits.into_iter().map(|hit| hit.result).collect())
    }
    
    /// Advanced search with filters
    pub async fn search_with_filters(
        &self,
        query: &str,
        user_id: &str,
        source_lang: Option<&str>,
        target_lang: Option<&str>,
        tags: Option<&[String]>,
        limit: usize,
        offset: usize,
    ) -> AppResult<Vec<SearchDocument>> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        // Build filter string
        let mut filters = vec![format!("user_id = '{}'", user_id)];
        
        if let Some(source) = source_lang {
            filters.push(format!("source_lang = '{}'", source));
        }
        
        if let Some(target) = target_lang {
            filters.push(format!("target_lang = '{}'", target));
        }
        
        if let Some(tag_list) = tags {
            if !tag_list.is_empty() {
                let tag_filters: Vec<String> = tag_list.iter()
                    .map(|tag| format!("tags = '{}'", tag))
                    .collect();
                filters.push(format!("({})", tag_filters.join(" OR ")));
            }
        }
        
        let filter_string = filters.join(" AND ");
        
        let results = index.search()
            .with_query(query)
            .with_filter(&filter_string)
            .with_limit(limit)
            .with_offset(offset)
            .with_attributes_to_highlight(&["title", "original_content", "translated_content"])
            .with_attributes_to_crop(&["original_content", "translated_content"])
            .with_crop_length(200)
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

    /// Get search suggestions based on existing documents
    pub async fn get_suggestions(&self, query: &str, limit: usize) -> AppResult<Vec<String>> {
        let index_name = format!("{}translations", self.index_prefix);
        let index = self.client.index(&index_name);
        
        // Search for similar documents and extract keywords
        let results = index.search()
            .with_query(query)
            .with_limit(limit * 2) // Get more results to extract suggestions
            .with_attributes_to_retrieve(&["title", "tags"])
            .execute::<serde_json::Value>()
            .await
            .map_err(|e| AppError::MeiliSearch(e.to_string()))?;
        
        let mut suggestions = std::collections::HashSet::new();
        
        // Extract suggestions from titles and tags
        for hit in results.hits {
            if let Some(title) = hit.result.get("title").and_then(|v| v.as_str()) {
                // Extract keywords from title
                let words: Vec<&str> = title.split_whitespace()
                    .filter(|word| word.len() > 2)
                    .take(3)
                    .collect();
                for word in words {
                    suggestions.insert(word.to_lowercase());
                }
            }
            
            if let Some(tags) = hit.result.get("tags").and_then(|v| v.as_array()) {
                for tag in tags {
                    if let Some(tag_str) = tag.as_str() {
                        suggestions.insert(tag_str.to_lowercase());
                    }
                }
            }
        }
        
        // Add some common search terms if we don't have enough suggestions
        if suggestions.len() < limit {
            let common_terms = vec!["api", "guide", "tutorial", "documentation", "reference"];
            for term in common_terms {
                if term.contains(&query.to_lowercase()) {
                    suggestions.insert(term.to_string());
                }
            }
        }
        
        let mut result: Vec<String> = suggestions.into_iter().collect();
        result.truncate(limit);
        result.sort();
        
        Ok(result)
    }
    
    /// Create document from translation for indexing
    pub fn create_search_document(
        translation_id: &str,
        url: &str,
        original_content: &str,
        translated_content: &str,
        source_lang: &str,
        target_lang: &str,
        user_id: &str,
        created_at: chrono::DateTime<chrono::Utc>,
    ) -> SearchDocument {
        let title = Self::extract_title_from_content(original_content)
            .or_else(|| Self::extract_title_from_content(translated_content))
            .unwrap_or_else(|| Self::extract_title_from_url(url));
        
        let tags = Self::generate_tags_from_content(url, original_content, translated_content);
        
        SearchDocument {
            id: translation_id.to_string(),
            title,
            content: format!("{}\n\n{}", original_content, translated_content), // Combined for backward compatibility
            original_content: original_content.to_string(),
            translated_content: translated_content.to_string(),
            url: url.to_string(),
            user_id: user_id.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
            created_at: created_at.to_rfc3339(),
            tags,
        }
    }
    
    /// Extract title from content
    fn extract_title_from_content(content: &str) -> Option<String> {
        for line in content.lines().take(10) {
            let line = line.trim();
            if line.starts_with("# ") && line.len() > 2 {
                return Some(line[2..].trim().to_string());
            }
            if line.starts_with("## ") && line.len() > 3 {
                return Some(line[3..].trim().to_string());
            }
        }
        None
    }
    
    /// Extract title from URL
    fn extract_title_from_url(url: &str) -> String {
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                let path = parsed_url.path();
                if let Some(last_segment) = path.split('/').next_back() {
                    if !last_segment.is_empty() && last_segment != "index.html" {
                        let name = last_segment.split('.').next().unwrap_or(last_segment);
                        if !name.is_empty() {
                            return format!("{} - {}", name, domain);
                        }
                    }
                }
                return domain.to_string();
            }
        }
        "Untitled Document".to_string()
    }
    
    /// Generate tags from content and URL
    fn generate_tags_from_content(url: &str, original: &str, translated: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Extract domain as tag
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(domain) = parsed_url.domain() {
                tags.push(domain.replace("www.", ""));
            }
        }
        
        // Detect content type based on keywords
        let content = format!("{} {}", original, translated).to_lowercase();
        
        if content.contains("api") || content.contains("endpoint") || content.contains("request") {
            tags.push("api".to_string());
        }
        if content.contains("tutorial") || content.contains("guide") || content.contains("how to") {
            tags.push("tutorial".to_string());
        }
        if content.contains("documentation") || content.contains("docs") {
            tags.push("documentation".to_string());
        }
        if content.contains("reference") || content.contains("manual") {
            tags.push("reference".to_string());
        }
        if content.contains("example") || content.contains("sample") {
            tags.push("example".to_string());
        }
        
        // Programming language detection
        let languages = ["rust", "python", "javascript", "typescript", "java", "go", "c++", "c#"];
        for lang in languages {
            if content.contains(lang) {
                tags.push(lang.to_string());
            }
        }
        
        tags
    }
}