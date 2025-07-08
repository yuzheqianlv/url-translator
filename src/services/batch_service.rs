use crate::services::{
    jina_service::JinaService, 
    deeplx_service::DeepLXService, 
    content_processor::ContentProcessor,
};
use crate::types::api_types::AppConfig;
use gloo_timers::future::TimeoutFuture;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::io::Write;

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

        // 生成文件名
        let file_name = self.generate_file_name(link);

        Ok(TranslatedDocument {
            link: link.clone(),
            original_content,
            translated_content,
            file_name,
        })
    }

    /// 生成文件名
    fn generate_file_name(&self, link: &DocumentLink) -> String {
        // 从URL中提取文件名，或使用标题
        if let Some(path) = link.url.split('/').last() {
            if path.ends_with(".html") {
                return path.replace(".html", ".md");
            }
        }
        
        // 使用标题生成文件名
        let safe_title = link.title
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect::<String>();
        
        format!("{:02}_{}.md", link.order + 1, safe_title)
    }

    /// 生成压缩归档文件（返回字节数组）
    pub fn create_zip_archive(&self, documents: &[TranslatedDocument]) -> Result<Vec<u8>, String> {
        web_sys::console::log_1(&"开始创建压缩归档文件".into());
        
        // 创建一个包含所有文档的文本文件
        let mut all_content = String::new();
        
        // 添加目录和统计信息
        all_content.push_str("# 翻译文档归档\n\n");
        all_content.push_str(&format!("总共翻译了 {} 个文档\n", documents.len()));
        all_content.push_str(&format!("归档时间: {}\n\n", js_sys::Date::new_0().to_string()));
        
        // 添加目录
        all_content.push_str("## 文档目录\n\n");
        for doc in documents {
            all_content.push_str(&format!(
                "{}. **{}**\n   - 原始URL: {}\n   - 文件名: {}\n\n",
                doc.link.order + 1,
                doc.link.title,
                doc.link.url,
                doc.file_name
            ));
        }
        
        all_content.push_str("\n---\n\n");
        
        // 添加所有翻译后的文档内容
        for doc in documents {
            all_content.push_str(&format!("# 文档 {}: {}\n\n", doc.link.order + 1, doc.link.title));
            all_content.push_str(&format!("**原始URL**: {}\n\n", doc.link.url));
            all_content.push_str("---\n\n");
            all_content.push_str(&doc.translated_content);
            all_content.push_str("\n\n");
            all_content.push_str(&"=".repeat(80));
            all_content.push_str("\n\n");
        }

        // 使用gzip压缩
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(all_content.as_bytes())
            .map_err(|e| format!("压缩失败: {}", e))?;
        
        let compressed_data = encoder.finish()
            .map_err(|e| format!("完成压缩失败: {}", e))?;

        web_sys::console::log_1(&format!(
            "归档文件创建完成，原始大小: {} 字节，压缩后: {} 字节，压缩率: {:.1}%", 
            all_content.len(),
            compressed_data.len(),
            (1.0 - compressed_data.len() as f64 / all_content.len() as f64) * 100.0
        ).into());
        
        Ok(compressed_data)
    }
}