use gloo_storage::{LocalStorage, Storage};
use crate::types::history::{HistoryEntry, HistoryFilter, HistorySortBy};
use std::collections::HashMap;

const HISTORY_STORAGE_KEY: &str = "translation_history";
const MAX_HISTORY_ENTRIES: usize = 100;

#[derive(Clone, Debug)]
pub struct HistoryService;

impl HistoryService {
    pub fn new() -> Self {
        Self
    }
    
    pub fn get_all_entries(&self) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        match LocalStorage::get::<Vec<HistoryEntry>>(HISTORY_STORAGE_KEY) {
            Ok(entries) => Ok(entries),
            Err(_) => Ok(Vec::new()),
        }
    }
    
    pub fn add_entry(&self, entry: HistoryEntry) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.get_all_entries()?;
        
        // 检查是否已存在相同URL的条目
        if let Some(existing_index) = entries.iter().position(|e| e.url == entry.url) {
            // 更新现有条目
            entries[existing_index] = entry;
        } else {
            // 添加新条目
            entries.insert(0, entry);
            
            // 保持历史记录数量在限制内
            if entries.len() > MAX_HISTORY_ENTRIES {
                entries.truncate(MAX_HISTORY_ENTRIES);
            }
        }
        
        self.save_entries(&entries)
    }
    
    pub fn get_entry_by_id(&self, id: &str) -> Result<Option<HistoryEntry>, Box<dyn std::error::Error>> {
        let entries = self.get_all_entries()?;
        Ok(entries.into_iter().find(|entry| entry.id == id))
    }
    
    pub fn delete_entry(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut entries = self.get_all_entries()?;
        entries.retain(|entry| entry.id != id);
        self.save_entries(&entries)
    }
    
    pub fn clear_history(&self) -> Result<(), Box<dyn std::error::Error>> {
        LocalStorage::delete(HISTORY_STORAGE_KEY);
        Ok(())
    }
    
    pub fn search_entries(
        &self,
        filter: &HistoryFilter,
        sort_by: &HistorySortBy,
    ) -> Result<Vec<HistoryEntry>, Box<dyn std::error::Error>> {
        let mut entries = self.get_all_entries()?;
        
        // 应用过滤器
        if let Some(ref search_term) = filter.search_term {
            let search_lower = search_term.to_lowercase();
            entries.retain(|entry| {
                entry.title.to_lowercase().contains(&search_lower) ||
                entry.url.to_lowercase().contains(&search_lower) ||
                entry.translated_content.to_lowercase().contains(&search_lower)
            });
        }
        
        if let Some(ref source_lang) = filter.source_lang {
            entries.retain(|entry| entry.source_lang == *source_lang);
        }
        
        if let Some(ref target_lang) = filter.target_lang {
            entries.retain(|entry| entry.target_lang == *target_lang);
        }
        
        // 应用排序
        match sort_by {
            HistorySortBy::CreatedAtDesc => {
                entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
            HistorySortBy::CreatedAtAsc => {
                entries.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            HistorySortBy::TitleAsc => {
                entries.sort_by(|a, b| a.title.cmp(&b.title));
            }
            HistorySortBy::TitleDesc => {
                entries.sort_by(|a, b| b.title.cmp(&a.title));
            }
            HistorySortBy::WordCountDesc => {
                entries.sort_by(|a, b| b.word_count.cmp(&a.word_count));
            }
            HistorySortBy::WordCountAsc => {
                entries.sort_by(|a, b| a.word_count.cmp(&b.word_count));
            }
        }
        
        Ok(entries)
    }
    
    pub fn get_statistics(&self) -> Result<HistoryStatistics, Box<dyn std::error::Error>> {
        let entries = self.get_all_entries()?;
        
        let total_entries = entries.len();
        let total_words = entries.iter().map(|e| e.word_count).sum();
        
        let mut language_pairs: HashMap<String, usize> = HashMap::new();
        let mut most_translated_domains: HashMap<String, usize> = HashMap::new();
        
        for entry in &entries {
            let pair = format!("{} -> {}", entry.source_lang, entry.target_lang);
            *language_pairs.entry(pair).or_insert(0) += 1;
            
            if let Ok(parsed_url) = web_sys::Url::new(&entry.url) {
                let hostname = parsed_url.hostname();
                if !hostname.is_empty() {
                    *most_translated_domains.entry(hostname).or_insert(0) += 1;
                }
            }
        }
        
        let most_used_language_pair = language_pairs
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(pair, _)| pair);
        
        let most_translated_domain = most_translated_domains
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(domain, _)| domain);
        
        Ok(HistoryStatistics {
            total_entries,
            total_words,
            most_used_language_pair,
            most_translated_domain,
        })
    }
    
    pub fn export_history(&self, format: ExportFormat) -> Result<String, Box<dyn std::error::Error>> {
        let entries = self.get_all_entries()?;
        
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(&entries)
                    .map_err(|e| format!("JSON序列化失败: {}", e).into())
            }
            ExportFormat::Csv => {
                let mut csv = String::from("ID,URL,Title,Source Language,Target Language,Created At,Word Count\n");
                for entry in entries {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        entry.id,
                        entry.url,
                        entry.title.replace(",", ";"),
                        entry.source_lang,
                        entry.target_lang,
                        entry.created_at,
                        entry.word_count
                    ));
                }
                Ok(csv)
            }
            ExportFormat::Markdown => {
                let mut md = String::from("# 翻译历史记录\n\n");
                for entry in entries {
                    md.push_str(&format!(
                        "## {}\n\n**URL**: {}\n\n**语言**: {} -> {}\n\n**创建时间**: {}\n\n**字数**: {}\n\n---\n\n",
                        entry.title,
                        entry.url,
                        entry.source_lang,
                        entry.target_lang,
                        entry.get_formatted_date(),
                        entry.word_count
                    ));
                }
                Ok(md)
            }
        }
    }
    
    fn save_entries(&self, entries: &[HistoryEntry]) -> Result<(), Box<dyn std::error::Error>> {
        LocalStorage::set(HISTORY_STORAGE_KEY, entries)
            .map_err(|e| format!("保存历史记录失败: {:?}", e).into())
    }
}

#[derive(Debug, Clone)]
pub struct HistoryStatistics {
    pub total_entries: usize,
    pub total_words: usize,
    pub most_used_language_pair: Option<String>,
    pub most_translated_domain: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
}