//! Translation-related database models

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Translation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub url: String,
    pub title: Option<String>,
    pub original_content: String,
    pub translated_content: String,
    pub source_language: String,
    pub target_language: String,
    pub content_hash: String,
    pub translation_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TranslationBatch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub status: String,
    pub total_urls: i32,
    pub completed_urls: i32,
    pub failed_urls: i32,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BatchUrl {
    pub id: Uuid,
    pub batch_id: Uuid,
    pub url: String,
    pub status: String,
    pub translation_id: Option<Uuid>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TranslateUrlRequest {
    #[validate(url)]
    pub url: String,
    
    #[validate(length(min = 2, max = 10))]
    pub source_language: String,
    
    #[validate(length(min = 2, max = 10))]
    pub target_language: String,
    
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct StartBatchTranslationRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    
    #[validate(length(min = 1, max = 100))]
    pub urls: Vec<String>,
    
    #[validate(length(min = 2, max = 10))]
    pub source_language: String,
    
    #[validate(length(min = 2, max = 10))]
    pub target_language: String,
    
    pub project_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    pub id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub original_content: String,
    pub translated_content: String,
    pub source_language: String,
    pub target_language: String,
    pub translation_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationListResponse {
    pub translations: Vec<TranslationResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTranslationResponse {
    pub id: Uuid,
    pub name: String,
    pub status: String,
    pub total_urls: i32,
    pub completed_urls: i32,
    pub failed_urls: i32,
    pub progress_percentage: f32,
    pub started_at: Option<DateTime<Utc>>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

impl From<Translation> for TranslationResponse {
    fn from(translation: Translation) -> Self {
        Self {
            id: translation.id,
            url: translation.url,
            title: translation.title,
            original_content: translation.original_content,
            translated_content: translation.translated_content,
            source_language: translation.source_language,
            target_language: translation.target_language,
            translation_time_ms: translation.translation_time_ms,
            created_at: translation.created_at,
        }
    }
}

impl From<TranslationBatch> for BatchTranslationResponse {
    fn from(batch: TranslationBatch) -> Self {
        let progress_percentage = if batch.total_urls > 0 {
            (batch.completed_urls as f32 / batch.total_urls as f32) * 100.0
        } else {
            0.0
        };
        
        Self {
            id: batch.id,
            name: batch.name,
            status: batch.status,
            total_urls: batch.total_urls,
            completed_urls: batch.completed_urls,
            failed_urls: batch.failed_urls,
            progress_percentage,
            started_at: batch.started_at,
            estimated_completion: None, // TODO: Calculate based on current progress
            created_at: batch.created_at,
        }
    }
}

impl Translation {
    /// Create a new translation record
    pub fn new(
        user_id: Uuid,
        project_id: Option<Uuid>,
        url: String,
        title: Option<String>,
        original_content: String,
        translated_content: String,
        source_language: String,
        target_language: String,
        content_hash: String,
        translation_time_ms: Option<i32>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            project_id,
            url,
            title,
            original_content,
            translated_content,
            source_language,
            target_language,
            content_hash,
            translation_time_ms,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl TranslationBatch {
    /// Create a new translation batch
    pub fn new(user_id: Uuid, project_id: Option<Uuid>, name: String, total_urls: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            project_id,
            name,
            status: BatchStatus::Pending.to_string(),
            total_urls,
            completed_urls: 0,
            failed_urls: 0,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl ToString for BatchStatus {
    fn to_string(&self) -> String {
        match self {
            BatchStatus::Pending => "pending".to_string(),
            BatchStatus::Processing => "processing".to_string(),
            BatchStatus::Completed => "completed".to_string(),
            BatchStatus::Failed => "failed".to_string(),
            BatchStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl ToString for TranslationStatus {
    fn to_string(&self) -> String {
        match self {
            TranslationStatus::Pending => "pending".to_string(),
            TranslationStatus::Processing => "processing".to_string(),
            TranslationStatus::Completed => "completed".to_string(),
            TranslationStatus::Failed => "failed".to_string(),
        }
    }
}