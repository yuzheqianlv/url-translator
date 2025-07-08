use crate::services::{
    jina_service::JinaService, 
    deeplx_service::DeepLXService, 
    content_processor::ContentProcessor,
};
use crate::types::api_types::AppConfig;
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder;

#[derive(Debug, Clone)]
pub struct DocumentLink {
    pub title: String,
    pub url: String,
    pub level: usize,  // 缩进级别，用于目录结构
    pub order: usize,  // 在目录中的顺序
}

#[derive(Debug, Clone)]
pub struct TranslatedDocument {
    pub link: DocumentLink,
    pub original_content: String,
    pub translated_content: String,
    pub file_name: String,  // 文件保存名称
    pub folder_path: String,  // 文件夹路径
    pub selected: bool,       // 是否选中下载
}

#[derive(Debug, Clone)]
pub struct BatchProgress {
    pub total: usize,
    pub completed: usize,
    pub current_task: String,
    pub failed_count: usize,
    pub status: BatchStatus,
}

#[derive(Debug, Clone)]
pub enum BatchStatus {
    Idle,
    Parsing,
    Translating,
    Packaging,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct FolderStructure {
    pub folders: HashMap<String, Vec<TranslatedDocument>>,
    pub total_files: usize,
    pub selected_files: usize,
}

impl FolderStructure {
    pub fn new() -> Self {
        Self {
            folders: HashMap::new(),
            total_files: 0,
            selected_files: 0,
        }
    }

    pub fn add_document(&mut self, doc: TranslatedDocument) {
        let folder = doc.folder_path.clone();
        self.folders.entry(folder).or_insert_with(Vec::new).push(doc);
        self.total_files += 1;
    }

    pub fn update_selection_count(&mut self) {
        self.selected_files = self.folders.values()
            .flat_map(|docs| docs.iter())
            .filter(|doc| doc.selected)
            .count();
    }
}

pub struct BatchTranslationService {
    jina_service: JinaService,
    deeplx_service: DeepLXService,
    config: AppConfig,
}

impl BatchTranslationService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            jina_service: JinaService::new(config),
            deeplx_service: DeepLXService::new(config),
            config: config.clone(),
        }
    }

    /// 解析文档主页，提取所有链接和目录结构
    pub async fn parse_document_index(&self, index_url: &str) -> Result<Vec<DocumentLink>, String> {
        web_sys::console::log_1(&"=== 开始解析文档索引 ===".into());
        
        // 提取索引页面内容
        let index_content = self.jina_service.extract_content(index_url, &self.config).await
            .map_err(|e| format!("无法提取索引页面内容: {}", e))?;
        
        // 解析链接
        let links = self.extract_links_from_content(&index_content);
        
        web_sys::console::log_1(&format!("解析完成，找到 {} 个文档链接", links.len()).into());
        
        Ok(links)
    }

    /// 从内容中提取链接和目录结构
    fn extract_links_from_content(&self, content: &str) -> Vec<DocumentLink> {
        let mut links = Vec::new();
        let mut order = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            
            // 查找包含链接的行
            if let Some(link_info) = self.parse_link_line(trimmed) {
                let level = self.calculate_indent_level(line);
                
                links.push(DocumentLink {
                    title: link_info.0,
                    url: link_info.1,
                    level,
                    order,
                });
                
                order += 1;
            }
        }

        links
    }

    /// 解析单行中的链接信息
    fn parse_link_line(&self, line: &str) -> Option<(String, String)> {
        // 匹配 Markdown 链接格式 [title](url)
        if let Some(start) = line.find('[') {
            if let Some(middle) = line[start..].find("](") {
                if let Some(end) = line[start + middle + 2..].find(')') {
                    let title_start = start + 1;
                    let title_end = start + middle;
                    let url_start = start + middle + 2;
                    let url_end = start + middle + 2 + end;
                    
                    let title = line[title_start..title_end].trim();
                    let url = line[url_start..url_end].trim();
                    
                    // 过滤掉非文档链接和特殊链接
                    if url.starts_with("http") && !title.is_empty() 
                        && !url.contains("print.html")  // 排除打印版本
                        && !url.contains("github.com")  // 排除GitHub链接
                        && !title.starts_with("**")     // 排除空的编号项
                        && title.len() > 1 {
                        
                        // 清理标题中的特殊字符和编号
                        let clean_title = self.clean_title(title);
                        if !clean_title.is_empty() {
                            return Some((clean_title, url.to_string()));
                        }
                    }
                }
            }
        }
        None
    }

    /// 计算行的缩进级别
    fn calculate_indent_level(&self, line: &str) -> usize {
        let leading_spaces = line.len() - line.trim_start().len();
        // 每4个空格或1个tab算一个级别
        leading_spaces / 4
    }

    /// 清理标题，移除编号和特殊字符
    fn clean_title(&self, title: &str) -> String {
        // 移除类似 "**1.**" 这样的编号
        let clean = title
            .replace("**", "")
            .trim()
            .to_string();
        
        // 移除开头的数字编号
        if let Some(pos) = clean.find('.') {
            if let Ok(_) = clean[..pos].trim().parse::<i32>() {
                return clean[pos + 1..].trim().to_string();
            }
        }
        
        clean
    }

    /// 批量翻译文档
    pub async fn batch_translate(
        &self, 
        links: Vec<DocumentLink>,
        progress_callback: impl Fn(BatchProgress) + 'static,
    ) -> Result<Vec<TranslatedDocument>, String> {
        let total = links.len();
        let mut translated_docs = Vec::new();
        let mut failed_count = 0;

        progress_callback(BatchProgress {
            total,
            completed: 0,
            current_task: "开始批量翻译".to_string(),
            failed_count: 0,
            status: BatchStatus::Translating,
        });

        for (index, link) in links.iter().enumerate() {
            progress_callback(BatchProgress {
                total,
                completed: index,
                current_task: format!("正在翻译: {} ({}/{})", link.title, index + 1, total),
                failed_count,
                status: BatchStatus::Translating,
            });

            // 增加重试机制
            let mut retry_count = 0;
            let max_retries = 3;
            
            loop {
                match self.translate_single_document(link).await {
                    Ok(translated_doc) => {
                        translated_docs.push(translated_doc);
                        web_sys::console::log_1(&format!("✓ 翻译完成: {}", link.title).into());
                        break;
                    }
                    Err(e) => {
                        retry_count += 1;
                        web_sys::console::log_1(&format!("✗ 翻译失败 (尝试 {}/{}): {} - {}", retry_count, max_retries, link.title, e).into());
                        
                        if retry_count >= max_retries {
                            failed_count += 1;
                            web_sys::console::log_1(&format!("✗ 最终失败: {}", link.title).into());
                            break;
                        } else {
                            // 重试前等待更长时间
                            let retry_delay = 2000 * retry_count as u32;
                            web_sys::console::log_1(&format!("等待 {}ms 后重试...", retry_delay).into());
                            TimeoutFuture::new(retry_delay).await;
                        }
                    }
                }
            }

            // 每个文档之间的基本延迟
            TimeoutFuture::new(1500).await;
            
            // 每处理5个文档后，额外休息一下
            if (index + 1) % 5 == 0 {
                web_sys::console::log_1(&"批量处理中途休息...".into());
                TimeoutFuture::new(3000).await;
            }
        }

        progress_callback(BatchProgress {
            total,
            completed: translated_docs.len(),
            current_task: if failed_count > 0 {
                format!("翻译完成，成功: {}, 失败: {}", translated_docs.len(), failed_count)
            } else {
                "所有文档翻译完成".to_string()
            },
            failed_count,
            status: BatchStatus::Completed,
        });

        Ok(translated_docs)
    }

    /// 翻译单个文档
    async fn translate_single_document(&self, link: &DocumentLink) -> Result<TranslatedDocument, String> {
        web_sys::console::log_1(&format!("开始翻译文档: {}", link.url).into());

        // 提取内容
        let original_content = match self.jina_service.extract_content(&link.url, &self.config).await {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Err("提取的内容为空".to_string());
                }
                content
            }
            Err(e) => return Err(format!("提取内容失败: {}", e)),
        };

        web_sys::console::log_1(&format!("内容提取成功，长度: {} 字符", original_content.len()).into());

        // 保护代码块
        let mut content_processor = ContentProcessor::new();
        let protected_content = content_processor.protect_code_blocks(&original_content);
        let protection_stats = content_processor.get_protection_stats();
        
        if protection_stats.total_blocks() > 0 {
            web_sys::console::log_1(&format!("代码块保护: {}", protection_stats.get_summary()).into());
        }

        // 翻译内容
        let translated_protected = match self.deeplx_service.translate(
            &protected_content,
            &self.config.default_source_lang,
            &self.config.default_target_lang,
            &self.config
        ).await {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Err("翻译结果为空".to_string());
                }
                content
            }
            Err(e) => return Err(format!("翻译失败: {}", e)),
        };

        web_sys::console::log_1(&format!("翻译成功，长度: {} 字符", translated_protected.len()).into());

        // 恢复代码块
        let translated_content = content_processor.restore_code_blocks(&translated_protected);

        // 生成文件名和文件夹路径
        let (folder_path, file_name) = self.generate_folder_structure(link);

        Ok(TranslatedDocument {
            link: link.clone(),
            original_content,
            translated_content,
            file_name,
            folder_path,
            selected: true,  // 默认选中
        })
    }

    /// 生成文件夹结构和文件名（保持层级结构）
    fn generate_folder_structure(&self, link: &DocumentLink) -> (String, String) {
        let url_path = link.url.split('/').collect::<Vec<&str>>();
        
        // 提取路径信息来生成文件夹结构
        let (folder_path, file_name) = if url_path.len() > 3 {
            // 分析URL路径来创建合理的文件夹结构
            let path_parts: Vec<&str> = url_path.iter().skip(3).copied().collect(); // 跳过 https://domain.com
            
            if path_parts.len() > 1 {
                // 有子路径，创建文件夹结构
                let folder_parts: Vec<String> = path_parts[..path_parts.len()-1]
                    .iter()
                    .map(|&part| self.clean_path_segment(part))
                    .filter(|part| !part.is_empty())
                    .collect();
                
                let folder_path = if folder_parts.is_empty() {
                    "docs".to_string()
                } else {
                    format!("docs/{}", folder_parts.join("/"))
                };
                
                (folder_path, self.generate_file_name_from_url_and_title(link))
            } else {
                // 只有一个文件，放在根目录
                ("docs".to_string(), self.generate_file_name_from_url_and_title(link))
            }
        } else {
            // URL太短，使用默认结构
            ("docs".to_string(), self.generate_file_name_from_url_and_title(link))
        };

        (folder_path, file_name)
    }

    /// 清理路径段，移除无效字符
    fn clean_path_segment(&self, segment: &str) -> String {
        segment
            .trim()
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    /// 根据URL和标题生成文件名
    fn generate_file_name_from_url_and_title(&self, link: &DocumentLink) -> String {
        // 首先尝试从URL提取文件名
        if let Some(path) = link.url.split('/').last() {
            if path.ends_with(".html") {
                let base_name = path.replace(".html", "");
                if !base_name.is_empty() && base_name != "index" {
                    return format!("{}.md", self.clean_path_segment(&base_name));
                }
            }
        }
        
        // 使用标题生成文件名
        let safe_title = link.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string();
        
        let title_part = if safe_title.len() > 50 {
            safe_title[..50].to_string()
        } else {
            safe_title
        };
        
        format!("{:02}_{}.md", link.order + 1, title_part)
    }

    /// 生成文件名 (保留原有方法以兼容性)
    fn generate_file_name(&self, link: &DocumentLink) -> String {
        self.generate_file_name_from_url_and_title(link)
    }

    /// 创建tar.gz归档文件包含选中的文档（保持文件夹结构）
    pub fn create_compressed_archive(&self, documents: &[TranslatedDocument]) -> Result<Vec<u8>, String> {
        web_sys::console::log_1(&"开始创建tar.gz归档文件".into());
        
        // 只处理选中的文档，按order排序
        let mut selected_docs: Vec<&TranslatedDocument> = documents.iter()
            .filter(|doc| doc.selected)
            .collect();
        
        if selected_docs.is_empty() {
            return Err("没有选中任何文档".to_string());
        }
        
        // 按索引顺序排序
        selected_docs.sort_by(|a, b| a.link.order.cmp(&b.link.order));
        
        // 创建gzip压缩的tar归档
        let tar_data = Vec::new();
        let encoder = GzEncoder::new(tar_data, Compression::default());
        let mut tar = Builder::new(encoder);
        
        // 添加README.md文件
        let readme_content = self.generate_readme_content(&selected_docs);
        let readme_bytes = readme_content.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_size(readme_bytes.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        
        tar.append_data(&mut header, "README.md", std::io::Cursor::new(readme_bytes))
            .map_err(|e| format!("无法添加README文件: {}", e))?;
        
        // 按文件夹分组并按顺序添加文档
        for doc in &selected_docs {
            let file_path = if doc.folder_path.is_empty() || doc.folder_path == "docs" {
                // 在根目录下添加序号前缀
                format!("{:03}_{}", doc.link.order + 1, doc.file_name)
            } else {
                // 在子文件夹下添加序号前缀
                format!("{}/{:03}_{}", doc.folder_path, doc.link.order + 1, doc.file_name)
            };

            web_sys::console::log_1(&format!("添加文件: {}", file_path).into());

            // 创建完整的文档内容，包含元数据
            let mut file_content = String::new();
            file_content.push_str(&format!("# {}\n\n", doc.link.title));
            file_content.push_str(&format!("> **原始URL**: {}\n", doc.link.url));
            file_content.push_str(&format!("> **翻译时间**: {}\n", js_sys::Date::new_0().to_locale_string("zh-CN", &js_sys::Object::new())));
            file_content.push_str(&format!("> **文档序号**: {}\n\n", doc.link.order + 1));
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

        web_sys::console::log_1(&format!(
            "tar.gz归档创建完成，包含 {} 个文档，压缩后大小: {} 字节",
            selected_docs.len(),
            compressed_data.len()
        ).into());

        Ok(compressed_data)
    }

    /// 生成README内容
    fn generate_readme_content(&self, documents: &[&TranslatedDocument]) -> String {
        let mut content = String::new();
        
        content.push_str("# 翻译文档归档\n\n");
        content.push_str(&format!("生成时间: {}\n", js_sys::Date::new_0().to_locale_string("zh-CN", &js_sys::Object::new())));
        content.push_str(&format!("文档总数: {} 个\n\n", documents.len()));
        
        // 按文件夹分组显示目录
        let mut folders: HashMap<String, Vec<&TranslatedDocument>> = HashMap::new();
        for doc in documents {
            folders.entry(doc.folder_path.clone())
                .or_insert_with(Vec::new)
                .push(doc);
        }

        content.push_str("## 文档目录\n\n");
        
        for (folder, docs) in folders {
            if !folder.is_empty() && folder != "documents" {
                content.push_str(&format!("### 📁 {}\n\n", folder));
            }
            
            for doc in docs {
                content.push_str(&format!(
                    "- [{}]({})\n  - 原始URL: {}\n  - 文件路径: {}/{}\n\n",
                    doc.link.title,
                    doc.file_name,
                    doc.link.url,
                    if folder.is_empty() || folder == "documents" { "." } else { &folder },
                    doc.file_name
                ));
            }
        }
        
        content.push_str("---\n\n");
        content.push_str("*此归档由URL翻译工具自动生成*\n");
        
        content
    }

    /// 为选中的文档创建单个文件下载
    pub fn create_single_file_download(&self, document: &TranslatedDocument) -> Vec<u8> {
        let mut content = String::new();
        
        // 添加文档头部信息
        content.push_str(&format!("# {}\n\n", document.link.title));
        content.push_str(&format!("> **原始URL**: {}\n", document.link.url));
        content.push_str(&format!("> **翻译时间**: {}\n", js_sys::Date::new_0().to_locale_string("zh-CN", &js_sys::Object::new())));
        content.push_str(&format!("> **文档序号**: {}\n\n", document.link.order + 1));
        content.push_str("---\n\n");
        
        // 添加翻译内容
        content.push_str(&document.translated_content);
        
        content.into_bytes()
    }
}