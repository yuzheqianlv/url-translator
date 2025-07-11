//! Translation service

use crate::config::AppConfig;
use crate::database::Database;
use crate::error::AppResult;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct TranslationService {
    database: Database,
    deeplx_client: Client,
    jina_client: Client,
    config: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub url: String,
    pub source_lang: String,
    pub target_lang: String,
    pub user_id: Option<uuid::Uuid>,
    pub project_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TranslationResponse {
    pub id: uuid::Uuid,
    pub url: String,
    pub source_lang: String,
    pub target_lang: String,
    pub original_content: String,
    pub translated_content: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub user_id: Option<uuid::Uuid>,
    pub project_id: Option<uuid::Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeepLXRequest {
    text: String,
    source_lang: String,
    target_lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeepLXResponse {
    code: i32,
    data: String,
}

impl TranslationService {
    pub async fn new(config: &AppConfig, database: Database) -> AppResult<Self> {
        let client_builder = Client::builder()
            .timeout(Duration::from_secs(config.translation.default_timeout_seconds))
            .user_agent("URL-Translator-Backend/1.0");
        
        Ok(Self {
            database,
            deeplx_client: reqwest::Client::builder().build().map_err(|e| {
                crate::error::AppError::Internal(format!("Failed to create DeepLX client: {}", e))
            })?,
            jina_client: client_builder.build().map_err(|e| {
                crate::error::AppError::Internal(format!("Failed to create Jina client: {}", e))
            })?,
            config: config.clone(),
        })
    }

    /// Translate a URL - Main translation workflow
    pub async fn translate_url(&self, request: TranslationRequest) -> AppResult<TranslationResponse> {
        tracing::info!("Starting URL translation for: {}", request.url);
        
        // Step 1: Extract content using Jina service
        let original_content = self.extract_content(&request.url).await
            .map_err(|e| crate::error::AppError::Internal(format!("Content extraction failed: {}", e)))?;
        
        tracing::info!("Extracted content, length: {} characters", original_content.len());
        
        // Step 2: Translate content using DeepLX
        let translated_content = self.translate_content(
            &original_content,
            &request.source_lang,
            &request.target_lang,
        ).await
            .map_err(|e| crate::error::AppError::Internal(format!("Translation failed: {}", e)))?;
        
        tracing::info!("Translation completed, result length: {} characters", translated_content.len());
        
        // Step 3: Save translation to database
        let translation_id = uuid::Uuid::new_v4();
        let translation = TranslationResponse {
            id: translation_id,
            url: request.url.clone(),
            source_lang: request.source_lang.clone(),
            target_lang: request.target_lang.clone(),
            original_content: original_content.clone(),
            translated_content: translated_content.clone(),
            created_at: chrono::Utc::now(),
            user_id: request.user_id,
            project_id: request.project_id,
        };
        
        self.save_translation(&translation).await
            .map_err(|e| crate::error::AppError::Internal(format!("Failed to save translation: {}", e)))?;
        
        tracing::info!("Translation saved to database with ID: {}", translation_id);
        
        Ok(translation)
    }

    /// Extract content from URL using Jina AI Reader
    async fn extract_content(&self, url: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let jina_url = format!("{}/{}", self.config.translation.jina_api_url, url);
        
        tracing::info!("Extracting content from: {}", jina_url);
        
        let response = self.jina_client
            .get(&jina_url)
            .header("User-Agent", "Mozilla/5.0 (compatible; URL-Translator-Backend/1.0)")
            .header("Accept", "text/plain, text/markdown, text/html, */*")
            .send()
            .await
            .map_err(|e| format!("Jina API request failed: {}", e))?;
        
        if response.status().is_success() {
            let content = response.text().await
                .map_err(|e| format!("Failed to read response: {}", e))?;
            
            if content.is_empty() {
                return Err("Jina API returned empty content".into());
            }
            
            Ok(content)
        } else {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("Jina API failed: {} - {}", status, error_text).into())
        }
    }
    
    /// Translate content using DeepLX
    async fn translate_content(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // If text is short, translate directly
        if text.len() <= self.config.translation.max_text_length {
            return self.translate_chunk(text, source_lang, target_lang).await;
        }
        
        // Split long text into chunks
        let chunks = self.split_text_into_chunks(text, self.config.translation.max_text_length);
        tracing::info!("Splitting long text into {} chunks", chunks.len());
        
        let mut translated_chunks = Vec::new();
        
        for (i, chunk) in chunks.iter().enumerate() {
            tracing::info!("Translating chunk {}/{}, length: {}", i + 1, chunks.len(), chunk.len());
            
            let translated_chunk = self.translate_chunk(chunk, source_lang, target_lang).await?;
            translated_chunks.push(translated_chunk);
            
            // Rate limiting: sleep between requests
            if i < chunks.len() - 1 {
                let delay = Duration::from_millis(1000 / self.config.translation.max_requests_per_second as u64);
                sleep(delay).await;
            }
        }
        
        Ok(translated_chunks.join("\n\n"))
    }
    
    /// Translate a single chunk of text
    async fn translate_chunk(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let request = DeepLXRequest {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        };
        
        tracing::info!("Sending translation request to: {}", self.config.translation.deeplx_api_url);
        
        let response = self.deeplx_client
            .post(&self.config.translation.deeplx_api_url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("DeepLX API request failed: {}", e))?;
        
        if response.status().is_success() {
            let response_text = response.text().await
                .map_err(|e| format!("Failed to read response: {}", e))?;
            
            tracing::debug!("DeepLX response: {}", response_text);
            
            // Try to parse as standard DeepLX response
            if let Ok(result) = serde_json::from_str::<DeepLXResponse>(&response_text) {
                if result.code == 200 {
                    if result.data.is_empty() {
                        return Err("DeepLX returned empty translation".into());
                    }
                    return Ok(result.data);
                } else {
                    return Err(format!("DeepLX translation failed, code: {}", result.code).into());
                }
            }
            
            // Try to parse as JSON and extract translation
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                if let Some(translated) = json_value
                    .get("translated_text")
                    .or_else(|| json_value.get("result"))
                    .or_else(|| json_value.get("translation"))
                    .or_else(|| json_value.get("data"))
                    .and_then(|v| v.as_str())
                {
                    return Ok(translated.to_string());
                }
            }
            
            // Assume it's plain text translation result
            if response_text.trim().is_empty() {
                return Err("API returned empty translation".into());
            }
            
            Ok(response_text)
        } else {
            let status = response.status();
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(format!("DeepLX API failed: {} - {}", status, error_text).into())
        }
    }
    
    /// Split text into chunks for translation
    fn split_text_into_chunks(&self, text: &str, max_length: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        
        if text.len() <= max_length {
            chunks.push(text.to_string());
            return chunks;
        }
        
        // First try to split on double newlines (paragraphs)
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        
        if paragraphs.len() > 1 {
            let mut current_chunk = String::new();
            
            for paragraph in paragraphs {
                let paragraph = paragraph.trim();
                if paragraph.is_empty() {
                    continue;
                }
                
                let test_chunk = if current_chunk.is_empty() {
                    paragraph.to_string()
                } else {
                    format!("{}\n\n{}", current_chunk, paragraph)
                };
                
                if test_chunk.len() <= max_length {
                    current_chunk = test_chunk;
                } else {
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                    }
                    
                    if paragraph.len() > max_length {
                        let sub_chunks = self.split_long_text(paragraph, max_length);
                        chunks.extend(sub_chunks);
                        current_chunk = String::new();
                    } else {
                        current_chunk = paragraph.to_string();
                    }
                }
            }
            
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
            }
        } else {
            // No paragraph breaks, split on single newlines
            let lines: Vec<&str> = text.split('\n').collect();
            
            if lines.len() > 1 {
                let mut current_chunk = String::new();
                
                for line in lines {
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    
                    let test_chunk = if current_chunk.is_empty() {
                        line.to_string()
                    } else {
                        format!("{}\n{}", current_chunk, line)
                    };
                    
                    if test_chunk.len() <= max_length {
                        current_chunk = test_chunk;
                    } else {
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.clone());
                        }
                        
                        if line.len() > max_length {
                            let sub_chunks = self.split_long_text(line, max_length);
                            chunks.extend(sub_chunks);
                            current_chunk = String::new();
                        } else {
                            current_chunk = line.to_string();
                        }
                    }
                }
                
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                }
            } else {
                // Single long text, split at sentence boundaries
                let sub_chunks = self.split_long_text(text, max_length);
                chunks.extend(sub_chunks);
            }
        }
        
        chunks
    }
    
    /// Split very long text at sentence boundaries
    fn split_long_text(&self, text: &str, max_length: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < text.len() {
            let end = std::cmp::min(start + max_length, text.len());
            
            let mut actual_end = end;
            if end < text.len() {
                // Find sentence boundary
                for i in (start..end).rev() {
                    if let Some(ch) = text.chars().nth(i) {
                        if ch == '.' || ch == '!' || ch == '?' || ch == '。' || ch == '！' || ch == '？' {
                            actual_end = i + 1;
                            break;
                        }
                    }
                }
                
                // If no sentence boundary found, look for space
                if actual_end == end {
                    for i in (start..end).rev() {
                        if let Some(ch) = text.chars().nth(i) {
                            if ch == ' ' || ch == '\n' || ch == '\t' {
                                actual_end = i + 1;
                                break;
                            }
                        }
                    }
                }
            }
            
            let chunk = text[start..actual_end].trim().to_string();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
            
            start = actual_end;
        }
        
        chunks
    }
    
    /// Save translation to database
    async fn save_translation(&self, translation: &TranslationResponse) -> AppResult<()> {
        let query = r#"
            INSERT INTO translations (id, url, source_language, target_language, original_content, translated_content, created_at, user_id, project_id, content_hash)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#;
        
        // Generate content hash
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&translation.original_content);
        let content_hash = format!("{:x}", hasher.finalize());
        
        sqlx::query(query)
            .bind(translation.id)
            .bind(&translation.url)
            .bind(&translation.source_lang)
            .bind(&translation.target_lang)
            .bind(&translation.original_content)
            .bind(&translation.translated_content)
            .bind(translation.created_at)
            .bind(translation.user_id)
            .bind(translation.project_id)
            .bind(content_hash)
            .execute(self.database.pool())
            .await
            .map_err(|e| crate::error::AppError::Database(e))?;
        
        Ok(())
    }
    
    /// Get translation history for user
    pub async fn get_user_translations(&self, user_id: uuid::Uuid, page: u32, limit: u32) -> AppResult<Vec<TranslationResponse>> {
        let offset = (page - 1) * limit;
        
        let query = r#"
            SELECT id, url, source_language as source_lang, target_language as target_lang, original_content, translated_content, created_at, user_id, project_id
            FROM translations
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
        "#;
        
        let rows = sqlx::query_as::<_, TranslationResponse>(
            "SELECT id, url, source_language as source_lang, target_language as target_lang, original_content, translated_content, created_at, user_id, project_id
            FROM translations
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3")
            .bind(user_id)
            .bind(limit as i64)
            .bind(offset as i64)
            .fetch_all(self.database.pool())
            .await
            .map_err(|e| crate::error::AppError::Database(e))?;
        
        Ok(rows)
    }

    /// Get specific translation
    pub async fn get_translation(&self, translation_id: uuid::Uuid, user_id: uuid::Uuid) -> AppResult<TranslationResponse> {
        let query = r#"
            SELECT id, url, source_language as source_lang, target_language as target_lang, original_content, translated_content, created_at, user_id, project_id
            FROM translations
            WHERE id = $1 AND user_id = $2
        "#;
        
        let row = sqlx::query_as::<_, TranslationResponse>(
            "SELECT id, url, source_language as source_lang, target_language as target_lang, original_content, translated_content, created_at, user_id, project_id
            FROM translations
            WHERE id = $1 AND user_id = $2")
            .bind(translation_id)
            .bind(user_id)
            .fetch_optional(self.database.pool())
            .await
            .map_err(|e| crate::error::AppError::Database(e))?
            .ok_or_else(|| crate::error::AppError::NotFound("Translation not found".to_string()))?;
        
        Ok(row)
    }

    /// Delete translation
    pub async fn delete_translation(&self, translation_id: uuid::Uuid, user_id: uuid::Uuid) -> AppResult<()> {
        let query = r#"
            DELETE FROM translations
            WHERE id = $1 AND user_id = $2
        "#;
        
        let result = sqlx::query(query)
            .bind(translation_id)
            .bind(user_id)
            .execute(self.database.pool())
            .await
            .map_err(|e| crate::error::AppError::Database(e))?;
        
        if result.rows_affected() == 0 {
            return Err(crate::error::AppError::NotFound("Translation not found".to_string()));
        }
        
        Ok(())
    }

    /// Start batch translation
    pub async fn start_batch_translation(&self, urls: Vec<String>, user_id: uuid::Uuid, source_lang: String, target_lang: String) -> AppResult<uuid::Uuid> {
        let batch_id = uuid::Uuid::new_v4();
        
        tracing::info!("Starting batch translation for {} URLs, batch ID: {}", urls.len(), batch_id);
        
        // TODO: Implement proper async job queue with Redis
        // For now, we'll process sequentially (not ideal for production)
        
        for url in urls {
            let url_clone = url.clone();
            let request = TranslationRequest {
                url,
                source_lang: source_lang.clone(),
                target_lang: target_lang.clone(),
                user_id: Some(user_id),
                project_id: None,
            };
            
            match self.translate_url(request).await {
                Ok(_) => {
                    tracing::info!("Successfully translated URL in batch: {}", url_clone);
                }
                Err(e) => {
                    tracing::error!("Failed to translate URL in batch: {} - {}", url_clone, e);
                    // Continue with other URLs instead of failing the entire batch
                }
            }
        }
        
        Ok(batch_id)
    }
    
    /// Get translation count for user
    pub async fn get_user_translation_count(&self, user_id: uuid::Uuid) -> AppResult<i64> {
        let query = r#"
            SELECT COUNT(*) as count
            FROM translations
            WHERE user_id = $1
        "#;
        
        let row = sqlx::query(
            "SELECT COUNT(*) as count
            FROM translations
            WHERE user_id = $1")
            .bind(user_id)
            .fetch_one(self.database.pool())
            .await
            .map_err(|e| crate::error::AppError::Database(e))?;
        
        Ok(row.get::<i64, _>("count"))
    }
}