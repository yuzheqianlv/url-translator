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
    #[serde(default = "default_entry_type")]
    pub entry_type: HistoryEntryType,
    #[serde(default)]
    pub batch_data: Option<BatchTranslationData>,
}

fn default_entry_type() -> HistoryEntryType {
    HistoryEntryType::SinglePage
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HistoryEntryType {
    SinglePage,
    BatchTranslation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTranslationData {
    pub total_documents: usize,
    pub successful_documents: usize,
    pub failed_documents: usize,
    pub index_url: String,
    pub document_list: Vec<BatchDocumentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchDocumentInfo {
    pub title: String,
    pub url: String,
    pub file_name: String,
    pub folder_path: String,
    pub order: usize,
    pub translated: bool,
    pub original_content: String,
    pub translated_content: String,
}

impl HistoryEntry {
    pub fn new_single_page(
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
            entry_type: HistoryEntryType::SinglePage,
            batch_data: None,
        }
    }

    pub fn new_batch_translation(
        index_url: String,
        title: String,
        source_lang: String,
        target_lang: String,
        batch_data: BatchTranslationData,
    ) -> Self {
        let word_count = batch_data
            .document_list
            .iter()
            .map(|doc| doc.original_content.split_whitespace().count())
            .sum();
        let now = js_sys::Date::new_0();
        let created_at = now.to_iso_string().as_string().unwrap();

        Self {
            id: Uuid::new_v4().to_string(),
            url: index_url,
            title,
            source_lang,
            target_lang,
            original_content: format!("批量翻译 {} 个文档", batch_data.total_documents),
            translated_content: format!(
                "成功: {}, 失败: {}",
                batch_data.successful_documents, batch_data.failed_documents
            ),
            created_at,
            word_count,
            entry_type: HistoryEntryType::BatchTranslation,
            batch_data: Some(batch_data),
        }
    }

    // 保持向后兼容性
    pub fn new(
        url: String,
        title: String,
        source_lang: String,
        target_lang: String,
        original_content: String,
        translated_content: String,
    ) -> Self {
        Self::new_single_page(
            url,
            title,
            source_lang,
            target_lang,
            original_content,
            translated_content,
        )
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

        format!(
            "{}-{:02}-{:02} {:02}:{:02}",
            year, month, day, hours, minutes
        )
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
