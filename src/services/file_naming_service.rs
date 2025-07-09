use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

/// 文件命名模式枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileNamingMode {
    /// 仅使用标题: title.md
    TitleOnly,
    /// 标题+时间戳: title_20250708_143022.md
    TitleWithTimestamp,
    /// 域名+路径+标题: example_docs_getting_started.md
    DomainPathTitle,
    /// 序号+标题: 001_getting_started.md
    OrderTitle,
    /// 自定义模板: {domain}_{title}_{timestamp}.md
    Custom(String),
}

/// 文件命名配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNamingConfig {
    pub mode: FileNamingMode,
    pub max_length: usize,
    pub timestamp_format: String,
    pub remove_special_chars: bool,
    pub lowercase: bool,
    pub word_separator: String,
    pub include_extension: bool,
}

impl Default for FileNamingConfig {
    fn default() -> Self {
        Self {
            mode: FileNamingMode::TitleWithTimestamp,
            max_length: 100,
            timestamp_format: "%Y%m%d_%H%M%S".to_string(),
            remove_special_chars: true,
            lowercase: true,
            word_separator: "_".to_string(),
            include_extension: true,
        }
    }
}

/// 文件命名上下文信息
#[derive(Debug, Clone)]
pub struct FileNamingContext {
    pub url: String,
    pub title: String,
    pub order: Option<usize>,
    pub timestamp: DateTime<Utc>,
    pub content_type: String,
    pub folder_path: Option<String>,
}

/// 文件命名结果
#[derive(Debug, Clone)]
pub struct FileNamingResult {
    pub file_name: String,
    pub folder_path: String,
    pub full_path: String,
    pub original_title: String,
    pub cleaned_title: String,
    pub extension: String,
}

/// 智能文件命名服务
pub struct FileNamingService {
    config: FileNamingConfig,
    used_names: HashSet<String>,
}

impl FileNamingService {
    /// 创建新的文件命名服务实例
    pub fn new(config: FileNamingConfig) -> Self {
        Self {
            config,
            used_names: HashSet::new(),
        }
    }

    /// 使用默认配置创建文件命名服务
    pub fn with_default_config() -> Self {
        Self::new(FileNamingConfig::default())
    }

    /// 生成文件名
    pub fn generate_file_name(&mut self, context: &FileNamingContext) -> FileNamingResult {
        let cleaned_title = self.clean_title(&context.title);
        let base_name = self.generate_base_name(context, &cleaned_title);
        let file_name = self.ensure_unique_name(&base_name);
        let extension = if self.config.include_extension {
            ".md"
        } else {
            ""
        };
        let final_name = format!("{}{}", file_name, extension);

        let folder_path = context
            .folder_path
            .clone()
            .unwrap_or_else(|| "documents".to_string());
        let full_path = format!("{}/{}", folder_path, final_name);

        // 记录已使用的文件名
        self.used_names.insert(file_name.clone());

        FileNamingResult {
            file_name: final_name,
            folder_path,
            full_path,
            original_title: context.title.clone(),
            cleaned_title,
            extension: extension.to_string(),
        }
    }

    /// 生成基础文件名（不包含扩展名）
    fn generate_base_name(&self, context: &FileNamingContext, cleaned_title: &str) -> String {
        let base_name = match &self.config.mode {
            FileNamingMode::TitleOnly => cleaned_title.to_string(),

            FileNamingMode::TitleWithTimestamp => {
                let timestamp = context.timestamp.format(&self.config.timestamp_format);
                format!("{}_{}", cleaned_title, timestamp)
            }

            FileNamingMode::DomainPathTitle => {
                let domain = self.extract_domain(&context.url);
                let path = self.extract_path(&context.url);
                if path.is_empty() {
                    format!("{}_{}", domain, cleaned_title)
                } else {
                    format!("{}_{}_{}", domain, path, cleaned_title)
                }
            }

            FileNamingMode::OrderTitle => {
                if let Some(order) = context.order {
                    format!("{:03}_{}", order + 1, cleaned_title)
                } else {
                    cleaned_title.to_string()
                }
            }

            FileNamingMode::Custom(template) => {
                self.apply_custom_template(template, context, cleaned_title)
            }
        };

        self.truncate_name(&base_name)
    }

    /// 清理标题，移除特殊字符
    fn clean_title(&self, title: &str) -> String {
        let mut result = title.to_string();

        // 移除特殊字符
        if self.config.remove_special_chars {
            result = result
                .chars()
                .map(|c| {
                    if c.is_alphanumeric() || c == '-' || c == '_' || c.is_whitespace() {
                        c
                    } else {
                        '_'
                    }
                })
                .collect();
        }

        // 转换为小写
        if self.config.lowercase {
            result = result.to_lowercase();
        }

        // 替换空格为分隔符
        result = result
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(&self.config.word_separator);

        // 清理连续的分隔符
        let separator = &self.config.word_separator;
        let double_separator = format!("{}{}", separator, separator);
        while result.contains(&double_separator) {
            result = result.replace(&double_separator, separator);
        }

        // 去除首尾分隔符
        result
            .trim_matches(separator.chars().next().unwrap_or('_'))
            .to_string()
    }

    /// 从URL提取域名
    fn extract_domain(&self, url_str: &str) -> String {
        if let Ok(url) = Url::parse(url_str) {
            if let Some(domain) = url.domain() {
                let clean_domain = domain.replace('.', "_");
                return self.clean_path_segment(&clean_domain);
            }
        }
        "unknown".to_string()
    }

    /// 从URL提取路径
    fn extract_path(&self, url_str: &str) -> String {
        if let Ok(url) = Url::parse(url_str) {
            let path = url.path();
            let path_segments: Vec<&str> = path
                .split('/')
                .filter(|s| !s.is_empty() && *s != "index.html")
                .collect();

            if path_segments.len() > 1 {
                let cleaned_segments: Vec<String> = path_segments[..path_segments.len() - 1]
                    .iter()
                    .map(|s| self.clean_path_segment(s))
                    .filter(|s| !s.is_empty())
                    .collect();

                return cleaned_segments.join("_");
            }
        }
        String::new()
    }

    /// 清理路径段
    fn clean_path_segment(&self, segment: &str) -> String {
        segment
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    /// 应用自定义模板
    fn apply_custom_template(
        &self,
        template: &str,
        context: &FileNamingContext,
        cleaned_title: &str,
    ) -> String {
        let domain = self.extract_domain(&context.url);
        let path = self.extract_path(&context.url);
        let timestamp = context.timestamp.format(&self.config.timestamp_format);
        let order = context
            .order
            .map(|o| format!("{:03}", o + 1))
            .unwrap_or_else(|| "000".to_string());

        template
            .replace("{domain}", &domain)
            .replace("{path}", &path)
            .replace("{title}", cleaned_title)
            .replace("{timestamp}", &timestamp.to_string())
            .replace("{order}", &order)
            .replace("{content_type}", &context.content_type)
    }

    /// 截断文件名到指定长度
    fn truncate_name(&self, name: &str) -> String {
        if name.len() <= self.config.max_length {
            name.to_string()
        } else {
            let mut truncated = name
                .chars()
                .take(self.config.max_length)
                .collect::<String>();
            // 确保不在单词中间截断
            if let Some(last_separator_pos) = truncated.rfind(&self.config.word_separator) {
                if last_separator_pos > self.config.max_length / 2 {
                    truncated = truncated[..last_separator_pos].to_string();
                }
            }
            truncated
        }
    }

    /// 确保文件名唯一性
    fn ensure_unique_name(&self, base_name: &str) -> String {
        if !self.used_names.contains(base_name) {
            return base_name.to_string();
        }

        // 添加数字后缀
        let mut counter = 1;
        loop {
            let candidate = format!("{}_{}", base_name, counter);
            if !self.used_names.contains(&candidate) {
                return candidate;
            }
            counter += 1;
        }
    }

    /// 预览文件名（不记录为已使用）
    pub fn preview_file_name(&self, context: &FileNamingContext) -> FileNamingResult {
        let cleaned_title = self.clean_title(&context.title);
        let base_name = self.generate_base_name(context, &cleaned_title);
        let file_name = self.ensure_unique_name(&base_name);
        let extension = if self.config.include_extension {
            ".md"
        } else {
            ""
        };
        let final_name = format!("{}{}", file_name, extension);

        let folder_path = context
            .folder_path
            .clone()
            .unwrap_or_else(|| "documents".to_string());
        let full_path = format!("{}/{}", folder_path, final_name);

        FileNamingResult {
            file_name: final_name,
            folder_path,
            full_path,
            original_title: context.title.clone(),
            cleaned_title,
            extension: extension.to_string(),
        }
    }

    /// 批量生成文件名
    pub fn generate_batch_file_names(
        &mut self,
        contexts: &[FileNamingContext],
    ) -> Vec<FileNamingResult> {
        contexts
            .iter()
            .map(|context| self.generate_file_name(context))
            .collect()
    }

    /// 重置已使用的文件名集合
    pub fn reset_used_names(&mut self) {
        self.used_names.clear();
    }

    /// 添加已使用的文件名
    pub fn add_used_name(&mut self, name: &str) {
        self.used_names.insert(name.to_string());
    }

    /// 获取配置
    pub fn get_config(&self) -> &FileNamingConfig {
        &self.config
    }

    /// 更新配置
    pub fn update_config(&mut self, config: FileNamingConfig) {
        self.config = config;
    }
}

/// 便捷函数：为单页翻译生成文件名
pub fn generate_single_page_filename(url: &str, title: &str) -> String {
    let context = FileNamingContext {
        url: url.to_string(),
        title: title.to_string(),
        order: None,
        timestamp: Utc::now(),
        content_type: "article".to_string(),
        folder_path: None,
    };

    let mut service = FileNamingService::with_default_config();
    let result = service.generate_file_name(&context);
    result.file_name
}

/// 便捷函数：为批量翻译生成文件名
pub fn generate_batch_filename(url: &str, title: &str, order: usize) -> String {
    let context = FileNamingContext {
        url: url.to_string(),
        title: title.to_string(),
        order: Some(order),
        timestamp: Utc::now(),
        content_type: "documentation".to_string(),
        folder_path: None,
    };

    let mut service = FileNamingService::with_default_config();
    let result = service.generate_file_name(&context);
    result.file_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_title() {
        let service = FileNamingService::with_default_config();

        assert_eq!(service.clean_title("Hello World"), "hello_world");
        assert_eq!(
            service.clean_title("Rust Programming Guide"),
            "rust_programming_guide"
        );
        assert_eq!(
            service.clean_title("API Reference / Getting Started"),
            "api_reference_getting_started"
        );
        assert_eq!(service.clean_title("文档 - 快速开始"), "文档_快速开始");
    }

    #[test]
    fn test_extract_domain() {
        let service = FileNamingService::with_default_config();

        assert_eq!(
            service.extract_domain("https://example.com/docs"),
            "example_com"
        );
        assert_eq!(
            service.extract_domain("https://rust-lang.org/book"),
            "rust_lang_org"
        );
        assert_eq!(service.extract_domain("invalid-url"), "unknown");
    }

    #[test]
    fn test_file_naming_modes() {
        let context = FileNamingContext {
            url: "https://example.com/docs/getting-started".to_string(),
            title: "Getting Started Guide".to_string(),
            order: Some(0),
            timestamp: Utc::now(),
            content_type: "documentation".to_string(),
            folder_path: None,
        };

        // Test TitleOnly mode
        let mut config = FileNamingConfig::default();
        config.mode = FileNamingMode::TitleOnly;
        let mut service = FileNamingService::new(config);
        let result = service.generate_file_name(&context);
        assert_eq!(result.file_name, "getting_started_guide.md");

        // Test OrderTitle mode
        config.mode = FileNamingMode::OrderTitle;
        service.update_config(config);
        let result = service.generate_file_name(&context);
        assert_eq!(result.file_name, "001_getting_started_guide.md");
    }

    #[test]
    fn test_unique_naming() {
        let context = FileNamingContext {
            url: "https://example.com/docs".to_string(),
            title: "Test Document".to_string(),
            order: None,
            timestamp: Utc::now(),
            content_type: "article".to_string(),
            folder_path: None,
        };

        let mut service = FileNamingService::with_default_config();

        // First file
        let result1 = service.generate_file_name(&context);
        // Second file with same title
        let result2 = service.generate_file_name(&context);

        assert_ne!(result1.file_name, result2.file_name);
        assert!(result2.file_name.contains("_1"));
    }
}
