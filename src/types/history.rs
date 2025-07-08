use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub url: String,
    pub title: String,
    pub source_lang: String,
    pub target_lang: String,
    pub original_content: String,
    pub translated_content: String,
    pub created_at: String,
    pub word_count: usize,
}

impl HistoryEntry {
    pub fn new(
        url: String,
        title: String,
        source_lang: String,
        target_lang: String,
        original_content: String,
        translated_content: String,
    ) -> Self {
        let word_count = original_content.split_whitespace().count();
        let now = js_sys::Date::new_0();
        let created_at = now.to_iso_string().as_string().unwrap();
        
        Self {
            id: Uuid::new_v4().to_string(),
            url,
            title,
            source_lang,
            target_lang,
            original_content,
            translated_content,
            created_at,
            word_count,
        }
    }
    
    pub fn get_summary(&self) -> String {
        format!(
            "{} ({} -> {})",
            if self.title.len() > 50 {
                format!("{}...", &self.title[..47])
            } else {
                self.title.clone()
            },
            self.source_lang,
            self.target_lang
        )
    }
    
    pub fn get_formatted_date(&self) -> String {
        // 简化的时间格式化
        let date = js_sys::Date::new(&self.created_at.clone().into());
        let year = date.get_full_year();
        let month = date.get_month() + 1;
        let day = date.get_date();
        let hours = date.get_hours();
        let minutes = date.get_minutes();
        
        format!("{}-{:02}-{:02} {:02}:{:02}", year, month, day, hours, minutes)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    pub search_term: Option<String>,
    pub source_lang: Option<String>,
    pub target_lang: Option<String>,
    pub date_from: Option<String>,
    pub date_to: Option<String>,
}

impl Default for HistoryFilter {
    fn default() -> Self {
        Self {
            search_term: None,
            source_lang: None,
            target_lang: None,
            date_from: None,
            date_to: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum HistorySortBy {
    CreatedAtDesc,
    CreatedAtAsc,
    TitleAsc,
    TitleDesc,
    WordCountDesc,
    WordCountAsc,
}

impl Default for HistorySortBy {
    fn default() -> Self {
        Self::CreatedAtDesc
    }
}