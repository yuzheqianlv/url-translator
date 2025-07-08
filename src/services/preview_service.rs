use crate::services::{
    jina_service::JinaService,
    deeplx_service::DeepLXService,
    content_processor::ContentProcessor,
};
use crate::types::api_types::AppConfig;

#[derive(Debug, Clone)]
pub struct PreviewContent {
    pub original_text: String,
    pub translated_text: String,
    pub word_count: usize,
    pub character_count: usize,
    pub preview_length: usize,
}

#[derive(Debug, Clone)]
pub struct PreviewOptions {
    pub max_paragraphs: usize,
    pub max_characters: usize,
    pub include_title: bool,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            max_paragraphs: 3,
            max_characters: 800,
            include_title: true,
        }
    }
}

pub struct PreviewService {
    jina_service: JinaService,
    deeplx_service: DeepLXService,
}

impl PreviewService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            jina_service: JinaService::new(config),
            deeplx_service: DeepLXService::new(config),
        }
    }

    /// 生成翻译预览
    pub async fn generate_preview(
        &self,
        url: &str,
        config: &AppConfig,
        options: &PreviewOptions,
    ) -> Result<PreviewContent, String> {
        web_sys::console::log_1(&"=== 开始生成翻译预览 ===".into());
        web_sys::console::log_1(&format!("URL: {}", url).into());

        // 提取完整内容
        let full_content = self.jina_service.extract_content(url, config).await
            .map_err(|e| format!("无法提取网页内容: {}", e))?;

        web_sys::console::log_1(&format!("完整内容长度: {} 字符", full_content.len()).into());

        // 提取预览部分
        let preview_text = self.extract_preview_text(&full_content, options);
        
        web_sys::console::log_1(&format!("预览内容长度: {} 字符", preview_text.len()).into());

        if preview_text.trim().is_empty() {
            return Err("无法提取有效的预览内容".to_string());
        }

        // 保护代码块
        let mut content_processor = ContentProcessor::new();
        let protected_content = content_processor.protect_code_blocks(&preview_text);
        
        let protection_stats = content_processor.get_protection_stats();
        if protection_stats.total_blocks() > 0 {
            web_sys::console::log_1(&format!("代码块保护: {}", protection_stats.get_summary()).into());
        }

        // 翻译预览内容
        web_sys::console::log_1(&"开始翻译预览内容...".into());
        let translated_protected = self.deeplx_service.translate(
            &protected_content,
            &config.default_source_lang,
            &config.default_target_lang,
            config
        ).await.map_err(|e| format!("翻译失败: {}", e))?;

        // 恢复代码块
        let translated_text = content_processor.restore_code_blocks(&translated_protected);

        web_sys::console::log_1(&"预览翻译完成".into());

        // 统计信息
        let word_count = self.count_words(&preview_text);
        let character_count = preview_text.chars().count();

        Ok(PreviewContent {
            original_text: preview_text,
            translated_text,
            word_count,
            character_count,
            preview_length: options.max_characters,
        })
    }

    /// 提取预览文本
    fn extract_preview_text(&self, content: &str, options: &PreviewOptions) -> String {
        let mut preview = String::new();
        let mut char_count = 0;
        let mut paragraph_count = 0;
        let mut found_title = false;

        for line in content.lines() {
            let trimmed = line.trim();
            
            // 跳过空行
            if trimmed.is_empty() {
                continue;
            }

            // 提取标题
            if !found_title && options.include_title && (trimmed.starts_with('#') || self.looks_like_title(trimmed)) {
                if char_count + trimmed.len() > options.max_characters {
                    break;
                }
                preview.push_str(trimmed);
                preview.push_str("\n\n");
                char_count += trimmed.len() + 2;
                found_title = true;
                continue;
            }

            // 跳过元数据和导航
            if self.is_metadata_or_navigation(trimmed) {
                continue;
            }

            // 添加段落
            if paragraph_count < options.max_paragraphs {
                if char_count + trimmed.len() > options.max_characters {
                    // 如果加上这段会超过字符限制，截断到句子边界
                    let remaining = options.max_characters - char_count;
                    if remaining > 50 { // 只有剩余空间足够时才截断
                        let truncated = self.truncate_to_sentence(trimmed, remaining);
                        preview.push_str(&truncated);
                        if !truncated.ends_with("...") {
                            preview.push_str("...");
                        }
                    }
                    break;
                }

                preview.push_str(trimmed);
                preview.push_str("\n\n");
                char_count += trimmed.len() + 2;
                paragraph_count += 1;
            } else {
                break;
            }
        }

        preview.trim().to_string()
    }

    /// 判断是否看起来像标题
    fn looks_like_title(&self, text: &str) -> bool {
        text.len() < 100 && 
        text.len() > 5 &&
        !text.contains('.') &&
        !text.starts_with('-') &&
        !text.starts_with('*') &&
        text.chars().any(|c| c.is_uppercase())
    }

    /// 判断是否是元数据或导航
    fn is_metadata_or_navigation(&self, text: &str) -> bool {
        let lower = text.to_lowercase();
        lower.contains("published") ||
        lower.contains("updated") ||
        lower.contains("author") ||
        lower.contains("source:") ||
        lower.contains("url:") ||
        lower.contains("time:") ||
        lower.starts_with("breadcrumb") ||
        lower.starts_with("navigation") ||
        lower.starts_with("menu") ||
        (text.starts_with('[') && text.ends_with(')')) // Markdown链接
    }

    /// 截断到句子边界
    fn truncate_to_sentence(&self, text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            return text.to_string();
        }

        let truncated = &text[..max_length];
        
        // 尝试在句号处截断
        if let Some(pos) = truncated.rfind('.') {
            if pos > max_length / 2 { // 确保不会截得太短
                return format!("{}.", &truncated[..pos]);
            }
        }

        // 尝试在逗号处截断
        if let Some(pos) = truncated.rfind(',') {
            if pos > max_length / 2 {
                return format!("{}...", &truncated[..pos]);
            }
        }

        // 尝试在空格处截断
        if let Some(pos) = truncated.rfind(' ') {
            if pos > max_length / 2 {
                return format!("{}...", &truncated[..pos]);
            }
        }

        format!("{}...", truncated)
    }

    /// 简单的词数统计
    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// 快速验证URL和配置
    pub async fn quick_validate(
        &self,
        url: &str,
        config: &AppConfig,
    ) -> Result<String, String> {
        web_sys::console::log_1(&"快速验证URL和配置...".into());

        // 尝试提取很少的内容进行测试
        let options = PreviewOptions {
            max_paragraphs: 1,
            max_characters: 200,
            include_title: true,
        };

        match self.generate_preview(url, config, &options).await {
            Ok(preview) => {
                Ok(format!(
                    "验证成功！提取了 {} 字符，翻译为 {} 字符",
                    preview.character_count,
                    preview.translated_text.chars().count()
                ))
            }
            Err(e) => Err(format!("验证失败: {}", e)),
        }
    }
}