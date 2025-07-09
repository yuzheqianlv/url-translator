use crate::services::{
    deeplx_service::DeepLXService,
    file_naming_service::{FileNamingContext, FileNamingService},
    jina_service::JinaService,
};
use crate::types::api_types::AppConfig;
use chrono::Utc;
use flate2::write::GzEncoder;
use flate2::Compression;
use gloo_timers::future::TimeoutFuture;
use std::collections::HashMap;
use tar::Builder;

#[derive(Debug, Clone)]
pub struct DocumentLink {
    pub title: String,
    pub url: String,
    pub level: usize, // 缩进级别，用于目录结构
    pub order: usize, // 在目录中的顺序
}

#[derive(Debug, Clone)]
pub struct TranslatedDocument {
    pub link: DocumentLink,
    pub original_content: String,
    pub translated_content: String,
    pub file_name: String,   // 文件保存名称
    pub folder_path: String, // 文件夹路径
    pub selected: bool,      // 是否选中下载
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

impl Default for FolderStructure {
    fn default() -> Self {
        Self::new()
    }
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
        self.folders
            .entry(folder)
            .or_default()
            .push(doc);
        self.total_files += 1;
    }

    pub fn update_selection_count(&mut self) {
        self.selected_files = self
            .folders
            .values()
            .flat_map(|docs| docs.iter())
            .filter(|doc| doc.selected)
            .count();
    }
}

pub struct BatchTranslationService {
    jina_service: JinaService,
    deeplx_service: DeepLXService,
    config: AppConfig,
    file_naming_service: FileNamingService,
}

impl BatchTranslationService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            jina_service: JinaService::new(config),
            deeplx_service: DeepLXService::new(config),
            config: config.clone(),
            file_naming_service: FileNamingService::new(config.file_naming.clone()),
        }
    }

    /// 解析文档主页，提取所有链接和目录结构
    pub async fn parse_document_index(&self, index_url: &str) -> Result<Vec<DocumentLink>, String> {
        web_sys::console::log_1(&"=== 开始解析文档索引 ===".into());

        // 提取基础域名
        let base_domain = self.extract_base_domain(index_url)
            .ok_or_else(|| "无法解析输入URL的域名".to_string())?;
        
        web_sys::console::log_1(&format!("基础域名: {}", base_domain).into());

        // 提取索引页面内容
        let index_content = self
            .jina_service
            .extract_content(index_url, &self.config)
            .await
            .map_err(|e| format!("无法提取索引页面内容: {e}"))?;

        // 解析链接（只保留相同域名的链接）
        let links = self.extract_links_from_content(&index_content, &base_domain);

        web_sys::console::log_1(&format!("解析完成，找到 {} 个同域名文档链接", links.len()).into());

        Ok(links)
    }

    /// 从内容中提取链接和目录结构
    fn extract_links_from_content(&self, content: &str, base_domain: &str) -> Vec<DocumentLink> {
        let mut links = Vec::new();
        let mut order = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // 查找包含链接的行
            if let Some(link_info) = self.parse_link_line(trimmed, base_domain) {
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

    /// 解析单行中的链接信息 - 增强版（仅保留相同域名的链接）
    fn parse_link_line(&self, line: &str, base_domain: &str) -> Option<(String, String)> {
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

                    // 检查是否为相同域名的有效文档URL
                    if self.is_same_domain_documentation_url(url, base_domain) && !title.is_empty() {
                        // 清理标题中的特殊字符和编号
                        let clean_title = self.clean_title_enhanced(title);
                        if !clean_title.is_empty() && clean_title.len() > 1 {
                            return Some((clean_title, url.to_string()));
                        }
                    }
                }
            }
        }
        None
    }

    /// 从URL中提取基础域名
    fn extract_base_domain(&self, url: &str) -> Option<String> {
        if let Ok(parsed_url) = url::Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                return Some(host.to_string());
            }
        }
        None
    }

    /// 检查是否为相同域名的有效文档URL
    fn is_same_domain_documentation_url(&self, url: &str, base_domain: &str) -> bool {
        // 首先检查基本URL格式
        if !self.is_valid_documentation_url(url) {
            return false;
        }

        // 检查是否为相同域名
        if let Some(url_domain) = self.extract_base_domain(url) {
            // 完全匹配域名
            if url_domain == base_domain {
                return true;
            }

            // 检查子域名情况（如 docs.example.com 和 example.com）
            if url_domain.ends_with(&format!(".{}", base_domain)) || 
               base_domain.ends_with(&format!(".{}", url_domain)) {
                return true;
            }
        }

        false
    }

    /// 验证是否为有效的文档URL - 增强版
    fn is_valid_documentation_url(&self, url: &str) -> bool {
        // 基本URL格式检查
        if !url.starts_with("http") {
            return false;
        }

        // 排除的URL模式
        let exclusions = [
            "print.html",  // 打印版本
            "github.com",  // GitHub链接
            "mailto:",     // 邮件链接
            "javascript:", // JS链接
            ".git",        // Git仓库
            "#",           // 锚点链接（仅限页面内）
            "search.html", // 搜索页面
            "404.html",    // 错误页面
        ];

        for exclusion in &exclusions {
            if url.contains(exclusion) {
                return false;
            }
        }

        // 检查文件扩展名（只接受HTML或无扩展名）
        if url.contains('.') && !url.ends_with(".html") && !url.ends_with(".htm") {
            // 排除图片、样式等非文档文件
            let extensions = [
                ".css", ".js", ".png", ".jpg", ".gif", ".svg", ".ico", ".pdf", ".woff", ".woff2",
                ".ttf",
            ];
            for ext in &extensions {
                if url.ends_with(ext) {
                    return false;
                }
            }
        }

        // 检查是否包含常见的文档路径模式
        let doc_patterns = [
            "/docs/",
            "/doc/",
            "/guide/",
            "/manual/",
            "/tutorial/",
            "/reference/",
            "/api/",
            "/book/",
            "/help/",
        ];

        let has_doc_pattern = doc_patterns.iter().any(|pattern| url.contains(pattern));

        // 如果包含文档模式或者是根目录下的HTML文件，则认为是有效的
        has_doc_pattern || url.matches('/').count() <= 3
    }

    /// 增强的标题清理方法
    fn clean_title_enhanced(&self, title: &str) -> String {
        let mut clean = title.to_string();

        // 移除粗体标记
        clean = clean.replace("**", "");

        // 移除斜体标记
        clean = clean.replace("*", "");

        // 移除代码标记
        clean = clean.replace("`", "");

        // 手动处理各种编号格式
        clean = self.remove_numbering_manual(&clean);

        // 清理多余的空格和特殊字符
        clean = clean
            .trim()
            .replace("  ", " ") // 多个空格变为单个
            .replace("\t", " ") // 制表符替换为空格
            .to_string();

        // 如果清理后的标题为空或过短，返回原始标题
        if clean.is_empty() || clean.len() < 2 {
            title.trim().to_string()
        } else {
            clean
        }
    }

    /// 手动移除编号的备用方案 - 增强版
    fn remove_numbering_manual(&self, text: &str) -> String {
        let trimmed = text.trim();

        // 特殊处理：移除开头的数字编号模式
        // 匹配类似 "1.", "1.1.", "4.1.", "5.2.1." 等格式
        let mut chars = trimmed.chars().peekable();
        let mut found_digit = false;
        let mut found_dot = false;
        let mut end_pos = 0;

        while let Some(ch) = chars.next() {
            match ch {
                '0'..='9' => {
                    found_digit = true;
                    end_pos += ch.len_utf8();
                }
                '.' => {
                    if found_digit {
                        found_dot = true;
                        end_pos += ch.len_utf8();
                        // 检查点后面是否还有数字
                        if let Some(&next_ch) = chars.peek() {
                            if next_ch.is_numeric() {
                                continue; // 继续处理多级编号
                            }
                        }
                    } else {
                        break;
                    }
                }
                ' ' | '\t' => {
                    end_pos += ch.len_utf8();
                    // 如果已经找到了数字和点，遇到空格就停止
                    if found_digit && found_dot {
                        break;
                    }
                }
                _ => {
                    // 遇到其他字符就停止
                    break;
                }
            }
        }

        // 如果找到了编号模式，移除它
        if found_digit && found_dot && end_pos > 0 && end_pos < trimmed.len() {
            trimmed[end_pos..].trim().to_string()
        } else {
            // 备用方案：查找第一个字母字符的位置
            for (i, ch) in trimmed.char_indices() {
                if ch.is_alphabetic() {
                    // 检查这个位置之前是否都是数字、点和空格
                    let prefix = &trimmed[..i];
                    if prefix
                        .chars()
                        .all(|c| c.is_numeric() || c == '.' || c.is_whitespace())
                    {
                        return trimmed[i..].trim().to_string();
                    }
                    break;
                }
            }
            trimmed.to_string()
        }
    }

    /// 计算行的缩进级别 - 增强版
    fn calculate_indent_level(&self, line: &str) -> usize {
        let leading_spaces = line.len() - line.trim_start().len();
        let trimmed = line.trim();

        // 优先通过编号判断级别：1. = 0级，1.1. = 1级，1.1.1. = 2级
        let mut dot_count = 0;
        let mut in_number = false;

        for ch in trimmed.chars() {
            match ch {
                '0'..='9' => {
                    in_number = true;
                }
                '.' => {
                    if in_number {
                        dot_count += 1;
                        in_number = false;
                    } else {
                        break;
                    }
                }
                ' ' | '\t' => {
                    if dot_count > 0 && !in_number {
                        // 已经找到完整的编号，遇到空格就停止
                        break;
                    }
                }
                _ => {
                    // 遇到其他字符就停止编号解析
                    break;
                }
            }
        }

        // 如果找到了编号，使用编号确定级别
        if dot_count > 0 && dot_count <= 4 {
            return std::cmp::min(dot_count - 1, 3); // 1个点=0级，2个点=1级，最大3级
        }

        // 如果没有编号，通过缩进和列表标记判断级别
        // 首先检查制表符缩进
        let tabs = line.chars().take_while(|&c| c == '\t').count();
        

        if tabs > 0 {
            std::cmp::min(tabs, 3)
        } else {
            // 按空格缩进计算，每4个空格算一级
            let base_level = std::cmp::min(leading_spaces / 4, 3);

            // 检查是否有列表标记，如果有，可能需要调整级别
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
                // 列表项可能需要额外的缩进级别
                std::cmp::min(base_level + (leading_spaces / 8), 3)
            } else {
                base_level
            }
        }
    }

    /// 清理标题，移除编号和特殊字符
    fn clean_title(&self, title: &str) -> String {
        // 移除类似 "**1.**" 这样的编号
        let clean = title.replace("**", "").trim().to_string();

        // 移除开头的数字编号
        if let Some(pos) = clean.find('.') {
            if clean[..pos].trim().parse::<i32>().is_ok() {
                return clean[pos + 1..].trim().to_string();
            }
        }

        clean
    }

    /// 批量翻译文档
    /// 
    /// 这个方法实现了优化的批量翻译功能，包括：
    /// - 智能重试机制
    /// - 动态延迟调整
    /// - 实时进度反馈
    /// - 错误恢复处理
    /// 
    /// # 参数
    /// 
    /// * `links` - 要翻译的文档链接列表
    /// * `progress_callback` - 进度回调函数
    /// 
    /// # 返回值
    /// 
    /// 返回成功翻译的文档列表，失败的文档会被跳过
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

            // 优化的重试机制
            let mut retry_count = 0;
            let max_retries = 2; // 减少重试次数

            loop {
                match self.translate_single_document(link).await {
                    Ok(translated_doc) => {
                        translated_docs.push(translated_doc);
                        web_sys::console::log_1(&format!("✓ 翻译完成: {}", link.title).into());
                        break;
                    }
                    Err(e) => {
                        retry_count += 1;
                        web_sys::console::log_1(
                            &format!(
                                "✗ 翻译失败 (尝试 {}/{}): {} - {}",
                                retry_count, max_retries, link.title, e
                            )
                            .into(),
                        );

                        if retry_count >= max_retries {
                            failed_count += 1;
                            web_sys::console::log_1(&format!("✗ 最终失败: {}", link.title).into());
                            break;
                        } else {
                            // 指数退避重试延迟
                            let retry_delay = 1000 * (2_u32.pow(retry_count as u32));
                            web_sys::console::log_1(
                                &format!("等待 {retry_delay}ms 后重试...").into(),
                            );
                            TimeoutFuture::new(retry_delay).await;
                        }
                    }
                }
            }

            // 动态调整延迟时间（根据成功率）
            let delay = if failed_count == 0 {
                1000 // 无错误时减少延迟
            } else {
                2000 // 有错误时增加延迟
            };
            TimeoutFuture::new(delay).await;

            // 每处理10个文档后，额外休息一下（减少休息频率）
            if (index + 1) % 10 == 0 {
                web_sys::console::log_1(&"批量处理中途休息...".into());
                TimeoutFuture::new(2000).await; // 缩短休息时间
            }
        }

        progress_callback(BatchProgress {
            total,
            completed: translated_docs.len(),
            current_task: if failed_count > 0 {
                format!(
                    "翻译完成，成功: {}, 失败: {}",
                    translated_docs.len(),
                    failed_count
                )
            } else {
                "所有文档翻译完成".to_string()
            },
            failed_count,
            status: BatchStatus::Completed,
        });

        Ok(translated_docs)
    }

    /// 翻译单个文档
    async fn translate_single_document(
        &self,
        link: &DocumentLink,
    ) -> Result<TranslatedDocument, String> {
        web_sys::console::log_1(&format!("开始翻译文档: {}", link.url).into());

        // 提取内容
        let original_content = match self
            .jina_service
            .extract_content(&link.url, &self.config)
            .await
        {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Err("提取的内容为空".to_string());
                }
                content
            }
            Err(e) => return Err(format!("提取内容失败: {e}")),
        };

        web_sys::console::log_1(
            &format!("内容提取成功，长度: {} 字符", original_content.len()).into(),
        );

        // 直接翻译内容（简化保护机制）
        let translated_content = match self
            .deeplx_service
            .translate(
                &original_content,
                &self.config.default_source_lang,
                &self.config.default_target_lang,
                &self.config,
            )
            .await
        {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Err("翻译结果为空".to_string());
                }
                content
            }
            Err(e) => return Err(format!("翻译失败: {e}")),
        };

        web_sys::console::log_1(
            &format!("翻译成功，长度: {} 字符", translated_content.len()).into(),
        );

        // 生成包含路径信息的文件名
        let enhanced_title = self.create_enhanced_title_with_path(&link.url, &link.title);

        let naming_context = FileNamingContext {
            url: link.url.clone(),
            title: enhanced_title,
            order: Some(link.order),
            timestamp: Utc::now(),
            content_type: "documentation".to_string(),
            folder_path: None,
        };

        let mut naming_service = FileNamingService::new(self.config.file_naming.clone());
        let naming_result = naming_service.generate_file_name(&naming_context);

        // 生成文件夹路径 - 统一使用documents文件夹
        let folder_path = self.generate_smart_folder_path(&link.url);

        Ok(TranslatedDocument {
            link: link.clone(),
            original_content,
            translated_content,
            file_name: naming_result.file_name,
            folder_path,
            selected: true, // 默认选中
        })
    }

    /// 创建包含路径信息的增强标题
    fn create_enhanced_title_with_path(&self, url: &str, original_title: &str) -> String {
        if let Ok(parsed_url) = url::Url::parse(url) {
            let path = parsed_url.path();
            let path_segments: Vec<&str> = path
                .split('/')
                .filter(|s| !s.is_empty() && !s.ends_with(".html") && *s != "index")
                .collect();

            if path_segments.len() > 1 {
                // 提取路径信息，创建层级标题
                let path_info: Vec<String> = path_segments[..path_segments.len()]
                    .iter()
                    .map(|&part| self.clean_path_segment(part))
                    .filter(|part| !part.is_empty())
                    .collect();

                if !path_info.is_empty() {
                    // 将路径信息加入到标题前面，用下划线分隔
                    format!("{}_{}", path_info.join("_"), original_title)
                } else {
                    original_title.to_string()
                }
            } else {
                original_title.to_string()
            }
        } else {
            original_title.to_string()
        }
    }

    /// 生成智能文件夹路径 - 简化为单层文件夹
    fn generate_smart_folder_path(&self, _url: &str) -> String {
        // 所有文件都放在同一个documents文件夹中
        // 文件排序通过文件名的序号前缀来实现
        "documents".to_string()
    }

    /// 生成文件夹结构和文件名（保持层级结构）- 保留兼容性
    fn generate_folder_structure(&self, link: &DocumentLink) -> (String, String) {
        let folder_path = self.generate_smart_folder_path(&link.url);
        let file_name = self.generate_file_name_from_url_and_title(link);
        (folder_path, file_name)
    }

    /// 清理路径段，移除无效字符
    fn clean_path_segment(&self, segment: &str) -> String {
        segment
            .trim()
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

    /// 根据URL和标题生成文件名
    fn generate_file_name_from_url_and_title(&self, link: &DocumentLink) -> String {
        // 首先尝试从URL提取文件名
        if let Some(path) = link.url.split('/').next_back() {
            if path.ends_with(".html") {
                let base_name = path.replace(".html", "");
                if !base_name.is_empty() && base_name != "index" {
                    return format!("{}.md", self.clean_path_segment(&base_name));
                }
            }
        }

        // 使用标题生成文件名
        let safe_title = link
            .title
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
    pub fn create_compressed_archive(
        &self,
        documents: &[TranslatedDocument],
    ) -> Result<Vec<u8>, String> {
        web_sys::console::log_1(&"开始创建tar.gz归档文件".into());

        // 只处理选中的文档，按order排序
        let mut selected_docs: Vec<&TranslatedDocument> =
            documents.iter().filter(|doc| doc.selected).collect();

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
            .map_err(|e| format!("无法添加README文件: {e}"))?;

        // 按文件夹分组并按顺序添加文档
        for doc in &selected_docs {
            let file_path = if doc.folder_path.is_empty() || doc.folder_path == "docs" {
                // 在根目录下添加序号前缀
                format!("{:03}_{}", doc.link.order + 1, doc.file_name)
            } else {
                // 在子文件夹下添加序号前缀
                format!(
                    "{}/{:03}_{}",
                    doc.folder_path,
                    doc.link.order + 1,
                    doc.file_name
                )
            };

            web_sys::console::log_1(&format!("添加文件: {file_path}").into());

            // 创建完整的文档内容，包含元数据
            let mut file_content = String::new();
            file_content.push_str(&format!("# {}\n\n", doc.link.title));
            file_content.push_str(&format!("> **原始URL**: {}\n", doc.link.url));
            file_content.push_str(&format!(
                "> **翻译时间**: {}\n",
                js_sys::Date::new_0().to_locale_string("zh-CN", &js_sys::Object::new())
            ));
            file_content.push_str(&format!("> **文档序号**: {}\n\n", doc.link.order + 1));
            file_content.push_str("---\n\n");
            file_content.push_str(&doc.translated_content);

            let file_bytes = file_content.as_bytes();
            let mut header = tar::Header::new_gnu();
            header.set_size(file_bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();

            tar.append_data(&mut header, &file_path, std::io::Cursor::new(file_bytes))
                .map_err(|e| format!("无法添加文件 {file_path}: {e}"))?;
        }

        // 完成tar归档
        let encoder = tar
            .into_inner()
            .map_err(|e| format!("无法完成tar归档: {e}"))?;

        let compressed_data = encoder
            .finish()
            .map_err(|e| format!("无法完成gzip压缩: {e}"))?;

        web_sys::console::log_1(
            &format!(
                "tar.gz归档创建完成，包含 {} 个文档，压缩后大小: {} 字节",
                selected_docs.len(),
                compressed_data.len()
            )
            .into(),
        );

        Ok(compressed_data)
    }

    /// 生成README内容
    fn generate_readme_content(&self, documents: &[&TranslatedDocument]) -> String {
        let mut content = String::new();

        content.push_str("# 翻译文档归档\n\n");
        content.push_str(&format!(
            "生成时间: {}\n",
            js_sys::Date::new_0().to_locale_string("zh-CN", &js_sys::Object::new())
        ));
        content.push_str(&format!("文档总数: {} 个\n\n", documents.len()));

        // 按文件夹分组显示目录
        let mut folders: HashMap<String, Vec<&TranslatedDocument>> = HashMap::new();
        for doc in documents {
            folders
                .entry(doc.folder_path.clone())
                .or_default()
                .push(doc);
        }

        content.push_str("## 文档目录\n\n");

        for (folder, docs) in folders {
            if !folder.is_empty() && folder != "documents" {
                content.push_str(&format!("### 📁 {folder}\n\n"));
            }

            for doc in docs {
                content.push_str(&format!(
                    "- [{}]({})\n  - 原始URL: {}\n  - 文件路径: {}/{}\n\n",
                    doc.link.title,
                    doc.file_name,
                    doc.link.url,
                    if folder.is_empty() || folder == "documents" {
                        "."
                    } else {
                        &folder
                    },
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
        content.push_str(&format!(
            "> **翻译时间**: {}\n",
            js_sys::Date::new_0().to_locale_string("zh-CN", &js_sys::Object::new())
        ));
        content.push_str(&format!("> **文档序号**: {}\n\n", document.link.order + 1));
        content.push_str("---\n\n");

        // 添加翻译内容
        content.push_str(&document.translated_content);

        content.into_bytes()
    }
}
