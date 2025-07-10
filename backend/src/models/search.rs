//! Search-related database models

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub query: String,
    pub results_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchRequest {
    #[validate(length(min = 1, max = 500))]
    pub query: String,
    
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub project_id: Option<Uuid>,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub sort_by: Option<SearchSortBy>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
    pub query: String,
    pub search_time_ms: u64,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub translation_id: Uuid,
    pub url: String,
    pub title: Option<String>,
    pub content_snippet: String,
    pub source_language: String,
    pub target_language: String,
    pub project_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub relevance_score: f32,
    pub highlight: SearchHighlight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHighlight {
    pub title: Option<String>,
    pub content: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SearchSuggestionsRequest {
    #[validate(length(min = 1, max = 100))]
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestionsResponse {
    pub suggestions: Vec<SearchSuggestion>,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSuggestion {
    pub text: String,
    pub score: f32,
    pub type_: SuggestionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryResponse {
    pub history: Vec<SearchHistoryItem>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistoryItem {
    pub query: String,
    pub results_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndexStatus {
    pub total_documents: i64,
    pub last_update: DateTime<Utc>,
    pub index_size_mb: f64,
    pub is_healthy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchSortBy {
    Relevance,
    Date,
    Title,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionType {
    Query,
    Title,
    Url,
    Content,
}

impl SearchHistory {
    /// Create a new search history entry
    pub fn new(user_id: Uuid, query: String, results_count: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            query,
            results_count,
            created_at: Utc::now(),
        }
    }
}

impl From<SearchHistory> for SearchHistoryItem {
    fn from(history: SearchHistory) -> Self {
        Self {
            query: history.query,
            results_count: history.results_count,
            created_at: history.created_at,
        }
    }
}

impl Default for SearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            page: Some(1),
            per_page: Some(20),
            project_id: None,
            source_language: None,
            target_language: None,
            date_from: None,
            date_to: None,
            sort_by: Some(SearchSortBy::Relevance),
            sort_order: Some(SortOrder::Desc),
        }
    }
}

impl ToString for SearchSortBy {
    fn to_string(&self) -> String {
        match self {
            SearchSortBy::Relevance => "relevance".to_string(),
            SearchSortBy::Date => "date".to_string(),
            SearchSortBy::Title => "title".to_string(),
            SearchSortBy::Url => "url".to_string(),
        }
    }
}

impl ToString for SortOrder {
    fn to_string(&self) -> String {
        match self {
            SortOrder::Asc => "asc".to_string(),
            SortOrder::Desc => "desc".to_string(),
        }
    }
}

impl ToString for SuggestionType {
    fn to_string(&self) -> String {
        match self {
            SuggestionType::Query => "query".to_string(),
            SuggestionType::Title => "title".to_string(),
            SuggestionType::Url => "url".to_string(),
            SuggestionType::Content => "content".to_string(),
        }
    }
}