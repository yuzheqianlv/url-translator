//! Search handlers

use axum::{extract::{Query, State}, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::AppResult;
use crate::services::Services;
use crate::middleware::auth::AuthenticatedUser;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub source_language: Option<String>,
    pub target_language: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub translation_id: String,
    pub url: String,
    pub title: Option<String>,
    pub content_snippet: String,
    pub source_language: String,
    pub target_language: String,
    pub project_name: Option<String>,
    pub created_at: String,
    pub relevance_score: f32,
}

/// Search translations
pub async fn search_translations(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<SearchResponse>> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    
    // Perform search using MeiliSearch service
    let search_results = services.search_service
        .search(&query.query, &user.user_id.to_string(), per_page as usize)
        .await?;
    
    // Convert search results to API format
    let results: Vec<SearchResult> = search_results
        .into_iter()
        .map(|doc| SearchResult {
            translation_id: doc.id.clone(),
            url: doc.url.clone(),
            title: extract_title_from_content(&doc.content),
            content_snippet: create_content_snippet(&doc.content, &query.query),
            source_language: "auto".to_string(), // TODO: Get from database
            target_language: "zh".to_string(), // TODO: Get from database
            project_name: None, // TODO: Get from database
            created_at: doc.created_at.clone(),
            relevance_score: 1.0, // TODO: Calculate relevance score
        })
        .collect();
    
    let total = results.len() as i64;
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;
    
    // Get search suggestions
    let suggestions = services.search_service
        .get_suggestions(&query.query, 5)
        .await
        .unwrap_or_default();
    
    let response = SearchResponse {
        results,
        total,
        page,
        per_page,
        total_pages,
        query: query.query,
        search_time_ms: 50, // TODO: Measure actual search time
        suggestions,
    };
    
    Ok(Json(response))
}

/// Extract title from content (first heading)
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

/// Create content snippet with search term highlighted
fn create_content_snippet(content: &str, search_term: &str) -> String {
    let words: Vec<&str> = content.split_whitespace().collect();
    let search_lower = search_term.to_lowercase();
    
    // Find the first occurrence of search term
    if let Some(pos) = words.iter().position(|word| {
        word.to_lowercase().contains(&search_lower)
    }) {
        let start = pos.saturating_sub(20);
        let end = std::cmp::min(start + 40, words.len());
        let snippet_words = &words[start..end];
        
        let mut snippet = snippet_words.join(" ");
        if snippet.len() > 200 {
            snippet.truncate(200);
            snippet.push_str("...");
        }
        
        snippet
    } else {
        // No search term found, return first 200 characters
        let mut snippet = content.chars().take(200).collect::<String>();
        if content.len() > 200 {
            snippet.push_str("...");
        }
        snippet
    }
}

/// Get search suggestions
pub async fn get_search_suggestions(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Query(query): Query<SearchSuggestionsQuery>,
) -> AppResult<Json<SearchSuggestionsResponse>> {
    let suggestions = services.search_service
        .get_suggestions(&query.query, query.limit.unwrap_or(5))
        .await?;
    
    let response = SearchSuggestionsResponse {
        suggestions,
        query: query.query,
    };
    
    Ok(Json(response))
}

/// Get search history
pub async fn get_search_history(
    State(_services): State<Services>,
    user: AuthenticatedUser,
) -> AppResult<Json<SearchHistoryResponse>> {
    // TODO: Implement search history storage in Redis
    // For now, return empty history
    let response = SearchHistoryResponse {
        history: vec![],
        total: 0,
    };
    
    Ok(Json(response))
}

/// Reindex content
pub async fn reindex_content(
    State(services): State<Services>,
    user: AuthenticatedUser,
) -> AppResult<Json<Value>> {
    // TODO: Add admin permission check
    
    // Re-initialize the search indices
    services.search_service.initialize_indices().await?;
    
    Ok(Json(json!({
        "message": "Search indices reinitialized successfully",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

#[derive(Debug, Deserialize)]
pub struct SearchSuggestionsQuery {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchSuggestionsResponse {
    pub suggestions: Vec<String>,
    pub query: String,
}

#[derive(Debug, Serialize)]
pub struct SearchHistoryResponse {
    pub history: Vec<SearchHistoryItem>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct SearchHistoryItem {
    pub query: String,
    pub timestamp: String,
    pub results_count: i64,
}