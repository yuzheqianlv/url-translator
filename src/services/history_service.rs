use gloo_storage::{LocalStorage, Storage};
use crate::types::history::{HistoryEntry, HistoryFilter, HistorySortBy, HistoryEntryType, BatchDocumentInfo};
use std::collections::HashMap;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

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
    
    /// 下载单页翻译记录
    pub fn download_single_page(&self, entry_id: &str) -> Result<Vec<u8>, String> {
        let entry = self.get_entry_by_id(entry_id)
            .map_err(|e| format!("获取记录失败: {}", e))?
            .ok_or("未找到指定记录".to_string())?;
        
        match entry.entry_type {
            HistoryEntryType::SinglePage => {
                let mut content = String::new();
                
                // 添加文档头部信息
                content.push_str(&format!("# {}\n\n", entry.title));
                content.push_str(&format!("> **原始URL**: {}\n", entry.url));
                content.push_str(&format!("> **翻译时间**: {}\n", entry.get_formatted_date()));
                content.push_str(&format!("> **语言**: {} -> {}\n", entry.source_lang, entry.target_lang));
                content.push_str(&format!("> **字数**: {} 字\n\n", entry.word_count));
                content.push_str("---\n\n");
                
                // 添加翻译内容
                content.push_str(&entry.translated_content);
                
                Ok(content.into_bytes())
            }
            HistoryEntryType::BatchTranslation => {
                Err("该记录是批量翻译，请使用批量下载功能".to_string())
            }
        }
    }
    
    /// 下载批量翻译记录
    pub fn download_batch_translation(&self, entry_id: &str, selected_docs: Option<Vec<usize>>) -> Result<Vec<u8>, String> {
        let entry = self.get_entry_by_id(entry_id)
            .map_err(|e| format!("获取记录失败: {}", e))?
            .ok_or("未找到指定记录".to_string())?;
        
        match entry.entry_type {
            HistoryEntryType::BatchTranslation => {
                let batch_data = entry.batch_data.as_ref()
                    .ok_or("批量翻译数据缺失".to_string())?;
                
                // 确定要下载的文档
                let docs_to_download: Vec<&BatchDocumentInfo> = if let Some(selected_indices) = selected_docs {
                    selected_indices.iter()
                        .filter_map(|&index| batch_data.document_list.get(index))
                        .collect()
                } else {
                    batch_data.document_list.iter().filter(|doc| doc.translated).collect()
                };
                
                if docs_to_download.is_empty() {
                    return Err("没有选中任何文档".to_string());
                }
                
                // 创建tar.gz归档
                self.create_batch_archive(&entry, &docs_to_download)
            }
            HistoryEntryType::SinglePage => {
                Err("该记录是单页翻译，请使用单页下载功能".to_string())
            }
        }
    }
    
    /// 创建批量翻译的tar.gz归档
    fn create_batch_archive(&self, entry: &HistoryEntry, documents: &[&BatchDocumentInfo]) -> Result<Vec<u8>, String> {
        let tar_data = Vec::new();
        let encoder = GzEncoder::new(tar_data, Compression::default());
        let mut tar = Builder::new(encoder);
        
        // 添加README.md文件
        let readme_content = self.generate_batch_readme(entry, documents);
        let readme_bytes = readme_content.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_size(readme_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        
        tar.append_data(&mut header, "README.md", std::io::Cursor::new(readme_bytes))
            .map_err(|e| format!("无法添加README文件: {}", e))?;
        
        // 按顺序添加文档
        for doc in documents {
            if !doc.translated {
                continue; // 跳过未翻译的文档
            }
            
            let file_path = if doc.folder_path.is_empty() || doc.folder_path == "docs" {
                format!("{:03}_{}", doc.order + 1, doc.file_name)
            } else {
                format!("{}/{:03}_{}", doc.folder_path, doc.order + 1, doc.file_name)
            };
            
            // 创建文档内容
            let mut file_content = String::new();
            file_content.push_str(&format!("# {}\n\n", doc.title));
            file_content.push_str(&format!("> **原始URL**: {}\n", doc.url));
            file_content.push_str(&format!("> **翻译时间**: {}\n", entry.get_formatted_date()));
            file_content.push_str(&format!("> **文档序号**: {}\n", doc.order + 1));
            file_content.push_str(&format!("> **语言**: {} -> {}\n\n", entry.source_lang, entry.target_lang));
            file_content.push_str("---\n\n");
            file_content.push_str(&doc.translated_content);
            
            let file_bytes = file_content.as_bytes();
            let mut header = tar::Header::new_gnu();
            header.set_size(file_bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            
            tar.append_data(&mut header, &file_path, std::io::Cursor::new(file_bytes))
                .map_err(|e| format!("无法添加文件 {}: {}", file_path, e))?;
        }
        
        // 完成tar归档
        let encoder = tar.into_inner()
            .map_err(|e| format!("无法完成tar归档: {}", e))?;
        
        let compressed_data = encoder.finish()
            .map_err(|e| format!("无法完成gzip压缩: {}", e))?;
        
        Ok(compressed_data)
    }
    
    /// 生成批量翻译的README内容
    fn generate_batch_readme(&self, entry: &HistoryEntry, documents: &[&BatchDocumentInfo]) -> String {
        let mut content = String::new();
        
        content.push_str("# 批量翻译文档归档\n\n");
        content.push_str(&format!("**项目名称**: {}\n", entry.title));
        content.push_str(&format!("**索引URL**: {}\n", entry.url));
        content.push_str(&format!("**翻译时间**: {}\n", entry.get_formatted_date()));
        content.push_str(&format!("**语言**: {} -> {}\n", entry.source_lang, entry.target_lang));
        content.push_str(&format!("**文档总数**: {} 个\n\n", documents.len()));
        
        if let Some(batch_data) = &entry.batch_data {
            content.push_str(&format!("**翻译统计**:\n"));
            content.push_str(&format!("- 总文档数: {}\n", batch_data.total_documents));
            content.push_str(&format!("- 成功翻译: {}\n", batch_data.successful_documents));
            content.push_str(&format!("- 失败文档: {}\n\n", batch_data.failed_documents));
        }
        
        content.push_str("## 文档目录\n\n");
        
        // 按文件夹分组显示目录
        let mut folders: HashMap<String, Vec<&BatchDocumentInfo>> = HashMap::new();
        for doc in documents {
            folders.entry(doc.folder_path.clone())
                .or_insert_with(Vec::new)
                .push(doc);
        }
        
        for (folder, docs) in folders {
            if !folder.is_empty() && folder != "docs" {
                content.push_str(&format!("### 📁 {}\n\n", folder));
            }
            
            for doc in docs {
                let file_name = if doc.folder_path.is_empty() || doc.folder_path == "docs" {
                    format!("{:03}_{}", doc.order + 1, doc.file_name)
                } else {
                    format!("{:03}_{}", doc.order + 1, doc.file_name)
                };
                
                content.push_str(&format!(
                    "- [{}]({})\n  - 原始URL: {}\n  - 文件路径: {}/{}\n\n",
                    doc.title,
                    file_name,
                    doc.url,
                    if folder.is_empty() || folder == "docs" { "." } else { &folder },
                    file_name
                ));
            }
        }
        
        content.push_str("---\n\n");
        content.push_str("*此归档由URL翻译工具历史记录功能生成*\n");
        
        content
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