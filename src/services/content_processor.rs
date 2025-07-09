use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    pub id: String,
    pub content: String,
    pub language: Option<String>,
    pub block_type: CodeBlockType,
}

#[derive(Debug, Clone, Copy)]
pub enum CodeBlockType {
    FencedCode,   // ```language
    InlineCode,   // `code`
    HtmlCode,     // <code>
    PreFormatted, // <pre>
    ScriptTag,    // <script>
    StyleTag,     // <style>

    // 新增的保护内容类型
    ApiEndpoint,   // /api/v1/users
    FilePath,      // /path/to/file
    EnvVariable,   // $ENV_VAR, ${VAR}
    VersionNumber, // v1.2.3, 1.0.0
    FunctionName,  // function_name()
    ConfigKey,     // config.key.value
    CommandLine,   // $ command --flag
    JsonData,      // {"key": "value"}
    SqlQuery,      // SELECT * FROM table
    RegexPattern,  // /pattern/flags
    HtmlTag,       // <div>, <span>
    MarkdownLink,  // [text](url)
    TechnicalTerm, // 技术术语
}

/// 内容处理器，用于保护代码块不被翻译
#[derive(Debug, Clone)]
pub struct ContentProcessor {
    preserved_blocks: HashMap<String, CodeBlock>,
}

impl ContentProcessor {
    pub fn new() -> Self {
        Self {
            preserved_blocks: HashMap::new(),
        }
    }

    /// 处理内容，保护代码块和其他技术内容不被翻译
    pub fn protect_code_blocks(&mut self, content: &str) -> String {
        let mut processed_content = content.to_string();

        // 按顺序处理各种代码块（顺序很重要）
        processed_content = self.protect_fenced_code_blocks(processed_content);
        processed_content = self.protect_inline_code(processed_content);
        processed_content = self.protect_html_code_elements(processed_content);

        // 新增的保护内容类型
        processed_content = self.protect_api_endpoints(processed_content);
        processed_content = self.protect_file_paths(processed_content);
        processed_content = self.protect_env_variables(processed_content);
        processed_content = self.protect_version_numbers(processed_content);
        processed_content = self.protect_function_names(processed_content);
        processed_content = self.protect_command_lines(processed_content);
        processed_content = self.protect_json_data(processed_content);
        processed_content = self.protect_html_tags(processed_content);

        processed_content
    }

    /// 恢复被保护的代码块
    pub fn restore_code_blocks(&self, content: &str) -> String {
        let mut restored_content = content.to_string();

        for (placeholder, code_block) in &self.preserved_blocks {
            restored_content = restored_content.replace(placeholder, &code_block.content);
        }

        restored_content
    }

    /// 获取保护的代码块统计
    pub fn get_protection_stats(&self) -> CodeProtectionStats {
        let mut stats = CodeProtectionStats::default();

        for code_block in self.preserved_blocks.values() {
            match &code_block.block_type {
                CodeBlockType::FencedCode => stats.fenced_blocks += 1,
                CodeBlockType::InlineCode => stats.inline_blocks += 1,
                CodeBlockType::HtmlCode => stats.html_code_blocks += 1,
                CodeBlockType::PreFormatted => stats.pre_blocks += 1,
                CodeBlockType::ScriptTag => stats.script_blocks += 1,
                CodeBlockType::StyleTag => stats.style_blocks += 1,
                CodeBlockType::ApiEndpoint => stats.api_endpoints += 1,
                CodeBlockType::FilePath => stats.file_paths += 1,
                CodeBlockType::EnvVariable => stats.env_variables += 1,
                CodeBlockType::VersionNumber => stats.version_numbers += 1,
                CodeBlockType::FunctionName => stats.function_names += 1,
                CodeBlockType::ConfigKey => stats.config_keys += 1,
                CodeBlockType::CommandLine => stats.command_lines += 1,
                CodeBlockType::JsonData => stats.json_data += 1,
                CodeBlockType::SqlQuery => stats.sql_queries += 1,
                CodeBlockType::RegexPattern => stats.regex_patterns += 1,
                CodeBlockType::HtmlTag => stats.html_tags += 1,
                CodeBlockType::MarkdownLink => stats.markdown_links += 1,
                CodeBlockType::TechnicalTerm => stats.technical_terms += 1,
            }
        }

        stats
    }

    /// 清理保护的代码块
    pub fn clear(&mut self) {
        self.preserved_blocks.clear();
    }

    /// 保护围栏代码块 ```
    fn protect_fenced_code_blocks(&mut self, content: String) -> String {
        let mut result = content;
        let mut search_start = 0;

        while let Some(start) = result[search_start..].find("```") {
            let actual_start = search_start + start;

            // 找到结束的```
            if let Some(end_pos) = result[actual_start + 3..].find("```") {
                let actual_end = actual_start + 3 + end_pos + 3;
                let full_block = &result[actual_start..actual_end];

                // 提取语言（第一行的```后面的内容）
                let first_line_end = result[actual_start..].find('\n').unwrap_or(3);
                let language = if first_line_end > 3 {
                    Some(
                        result[actual_start + 3..actual_start + first_line_end]
                            .trim()
                            .to_string(),
                    )
                } else {
                    None
                };

                let placeholder = self.create_placeholder();
                let code_block = CodeBlock {
                    id: Uuid::new_v4().to_string(),
                    content: full_block.to_string(),
                    language: language.clone(),
                    block_type: CodeBlockType::FencedCode,
                };

                self.preserved_blocks
                    .insert(placeholder.clone(), code_block);
                result.replace_range(actual_start..actual_end, &placeholder);

                search_start = actual_start + placeholder.len();
            } else {
                search_start = actual_start + 3;
            }
        }

        result
    }

    /// 保护行内代码 `code`
    fn protect_inline_code(&mut self, content: String) -> String {
        let mut result = content;
        let mut search_start = 0;

        while let Some(start) = result[search_start..].find('`') {
            let actual_start = search_start + start;

            // 找到结束的`，但不能跨行
            if let Some(end_pos) = result[actual_start + 1..].find('`') {
                let actual_end = actual_start + 1 + end_pos + 1;
                let code_content = &result[actual_start + 1..actual_end - 1];

                // 检查是否包含换行符（如果包含则跳过）
                if !code_content.contains('\n') && !code_content.trim().is_empty() {
                    let full_block = &result[actual_start..actual_end];

                    let placeholder = self.create_placeholder();
                    let code_block = CodeBlock {
                        id: Uuid::new_v4().to_string(),
                        content: full_block.to_string(),
                        language: None,
                        block_type: CodeBlockType::InlineCode,
                    };

                    self.preserved_blocks
                        .insert(placeholder.clone(), code_block);
                    result.replace_range(actual_start..actual_end, &placeholder);

                    search_start = actual_start + placeholder.len();
                } else {
                    search_start = actual_start + 1;
                }
            } else {
                search_start = actual_start + 1;
            }
        }

        result
    }

    /// 保护HTML代码元素
    fn protect_html_code_elements(&mut self, content: String) -> String {
        let mut result = content;

        // 保护 <code> 标签
        result = self.protect_html_tag(result, "code", CodeBlockType::HtmlCode);

        // 保护 <pre> 标签
        result = self.protect_html_tag(result, "pre", CodeBlockType::PreFormatted);

        // 保护 <script> 标签
        result = self.protect_html_tag(result, "script", CodeBlockType::ScriptTag);

        // 保护 <style> 标签
        result = self.protect_html_tag(result, "style", CodeBlockType::StyleTag);

        result
    }

    /// 保护特定的HTML标签
    fn protect_html_tag(
        &mut self,
        content: String,
        tag: &str,
        block_type: CodeBlockType,
    ) -> String {
        let mut result = content;
        let start_tag = format!("<{}", tag);
        let end_tag = format!("</{}>", tag);
        let mut search_start = 0;

        while let Some(start) = result[search_start..].find(&start_tag) {
            let actual_start = search_start + start;

            // 找到标签的结束 >
            if let Some(tag_end) = result[actual_start..].find('>') {
                let tag_close_pos = actual_start + tag_end + 1;

                // 找到结束标签
                if let Some(end_pos) = result[tag_close_pos..].find(&end_tag) {
                    let actual_end = tag_close_pos + end_pos + end_tag.len();
                    let full_block = &result[actual_start..actual_end];

                    let placeholder = self.create_placeholder();
                    let language = match tag {
                        "script" => Some("javascript".to_string()),
                        "style" => Some("css".to_string()),
                        _ => None,
                    };

                    let code_block = CodeBlock {
                        id: Uuid::new_v4().to_string(),
                        content: full_block.to_string(),
                        language,
                        block_type,
                    };

                    self.preserved_blocks
                        .insert(placeholder.clone(), code_block);
                    result.replace_range(actual_start..actual_end, &placeholder);

                    search_start = actual_start + placeholder.len();
                } else {
                    search_start = tag_close_pos;
                }
            } else {
                search_start = actual_start + start_tag.len();
            }
        }

        result
    }

    /// 创建唯一的占位符
    fn create_placeholder(&self) -> String {
        format!("__CODE_BLOCK_{}__", Uuid::new_v4().simple())
    }

    /// 保护API端点 (例如: /api/v1/users, /docs/api)
    fn protect_api_endpoints(&mut self, content: String) -> String {
        let mut result = content;
        let mut replacements = Vec::new();

        // 简单的API端点匹配
        for line in result.lines() {
            if line.contains("/api/")
                || line.contains("/docs/")
                || line.contains("/v1/")
                || line.contains("/v2/")
            {
                // 查找类似 /api/v1/users 的模式
                let mut start = 0;
                while let Some(pos) = line[start..].find('/') {
                    let actual_start = start + pos;
                    let end = line[actual_start..]
                        .find(' ')
                        .unwrap_or(line.len() - actual_start);
                    let potential_endpoint = &line[actual_start..actual_start + end];

                    // 检查是否是有效的API端点
                    if potential_endpoint.len() > 3 && potential_endpoint.matches('/').count() >= 2
                    {
                        let placeholder = self.create_placeholder();
                        let code_block = CodeBlock {
                            id: Uuid::new_v4().to_string(),
                            content: potential_endpoint.to_string(),
                            language: Some("api".to_string()),
                            block_type: CodeBlockType::ApiEndpoint,
                        };

                        self.preserved_blocks
                            .insert(placeholder.clone(), code_block);
                        replacements.push((potential_endpoint.to_string(), placeholder));
                    }

                    start = actual_start + 1;
                }
            }
        }

        // 应用替换
        for (original, placeholder) in replacements {
            result = result.replace(&original, &placeholder);
        }

        result
    }

    /// 保护文件路径 (例如: /path/to/file, ./src/main.rs)
    fn protect_file_paths(&mut self, content: String) -> String {
        let mut result = content;
        let mut replacements = Vec::new();

        for line in result.lines() {
            // 查找文件路径模式
            for word in line.split_whitespace() {
                if (word.starts_with('/') || word.starts_with("./") || word.starts_with("~/"))
                    && word.contains('.')
                    && word.len() > 3
                {
                    let placeholder = self.create_placeholder();
                    let code_block = CodeBlock {
                        id: Uuid::new_v4().to_string(),
                        content: word.to_string(),
                        language: Some("path".to_string()),
                        block_type: CodeBlockType::FilePath,
                    };

                    self.preserved_blocks
                        .insert(placeholder.clone(), code_block);
                    replacements.push((word.to_string(), placeholder));
                }
            }
        }

        // 应用替换
        for (original, placeholder) in replacements {
            result = result.replace(&original, &placeholder);
        }

        result
    }

    /// 保护环境变量 (例如: $VAR, ${VAR}, %VAR%)
    fn protect_env_variables(&mut self, content: String) -> String {
        let mut result = content;
        let mut replacements = Vec::new();

        for line in result.lines() {
            for word in line.split_whitespace() {
                if (word.starts_with('$') || word.starts_with('%')) && word.len() > 2 {
                    let placeholder = self.create_placeholder();
                    let code_block = CodeBlock {
                        id: Uuid::new_v4().to_string(),
                        content: word.to_string(),
                        language: Some("env".to_string()),
                        block_type: CodeBlockType::EnvVariable,
                    };

                    self.preserved_blocks
                        .insert(placeholder.clone(), code_block);
                    replacements.push((word.to_string(), placeholder));
                }
            }
        }

        // 应用替换
        for (original, placeholder) in replacements {
            result = result.replace(&original, &placeholder);
        }

        result
    }

    /// 保护版本号 (例如: v1.2.3, 1.0.0, 2.4.1-beta)
    fn protect_version_numbers(&mut self, content: String) -> String {
        let mut result = content;
        let mut replacements = Vec::new();

        for line in result.lines() {
            for word in line.split_whitespace() {
                // 检查是否是版本号格式
                if self.is_version_number(word) {
                    let placeholder = self.create_placeholder();
                    let code_block = CodeBlock {
                        id: Uuid::new_v4().to_string(),
                        content: word.to_string(),
                        language: Some("version".to_string()),
                        block_type: CodeBlockType::VersionNumber,
                    };

                    self.preserved_blocks
                        .insert(placeholder.clone(), code_block);
                    replacements.push((word.to_string(), placeholder));
                }
            }
        }

        // 应用替换
        for (original, placeholder) in replacements {
            result = result.replace(&original, &placeholder);
        }

        result
    }

    /// 检查是否是版本号格式
    fn is_version_number(&self, text: &str) -> bool {
        // 简单的版本号检查
        if text.len() < 3 {
            return false;
        }

        // 检查 v1.2.3 格式
        if text.starts_with('v') && text[1..].chars().next().unwrap_or('a').is_numeric() {
            return text[1..].contains('.')
                && text[1..]
                    .chars()
                    .filter(|c| c.is_numeric() || *c == '.' || *c == '-')
                    .count()
                    == text[1..].len();
        }

        // 检查 1.2.3 格式
        if text.chars().next().unwrap_or('a').is_numeric() {
            let dot_count = text.chars().filter(|c| *c == '.').count();
            return dot_count >= 1
                && dot_count <= 3
                && text
                    .chars()
                    .all(|c| c.is_numeric() || c == '.' || c == '-' || c.is_alphabetic());
        }

        false
    }

    /// 保护函数名 (例如: function_name(), method.call())
    fn protect_function_names(&mut self, content: String) -> String {
        let mut result = content;
        let mut replacements = Vec::new();

        for line in result.lines() {
            for word in line.split_whitespace() {
                // 检查是否是函数调用格式
                if word.ends_with("()") && word.len() > 2 {
                    let function_name = &word[..word.len() - 2];
                    if function_name
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                    {
                        let placeholder = self.create_placeholder();
                        let code_block = CodeBlock {
                            id: Uuid::new_v4().to_string(),
                            content: word.to_string(),
                            language: Some("function".to_string()),
                            block_type: CodeBlockType::FunctionName,
                        };

                        self.preserved_blocks
                            .insert(placeholder.clone(), code_block);
                        replacements.push((word.to_string(), placeholder));
                    }
                }
            }
        }

        // 应用替换
        for (original, placeholder) in replacements {
            result = result.replace(&original, &placeholder);
        }

        result
    }

    /// 保护命令行 (例如: $ command, # command)
    fn protect_command_lines(&mut self, content: String) -> String {
        let mut result = content;
        let mut replacements = Vec::new();

        for line in result.lines() {
            let trimmed = line.trim();
            if (trimmed.starts_with("$ ") || trimmed.starts_with("# ")) && trimmed.len() > 2 {
                let placeholder = self.create_placeholder();
                let code_block = CodeBlock {
                    id: Uuid::new_v4().to_string(),
                    content: trimmed.to_string(),
                    language: Some("shell".to_string()),
                    block_type: CodeBlockType::CommandLine,
                };

                self.preserved_blocks
                    .insert(placeholder.clone(), code_block);
                replacements.push((trimmed.to_string(), placeholder));
            }
        }

        // 应用替换
        for (original, placeholder) in replacements {
            result = result.replace(&original, &placeholder);
        }

        result
    }

    /// 保护JSON数据 (例如: {"key": "value"})
    fn protect_json_data(&mut self, content: String) -> String {
        let mut result = content;

        // 简单的JSON对象检测
        let mut brace_count = 0;
        let mut start_pos = None;

        for (i, ch) in result.char_indices() {
            match ch {
                '{' => {
                    if brace_count == 0 {
                        start_pos = Some(i);
                    }
                    brace_count += 1;
                }
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        if let Some(start) = start_pos {
                            let json_text = &result[start..i + 1];
                            if json_text.len() > 10
                                && json_text.contains('"')
                                && json_text.contains(':')
                            {
                                let placeholder = self.create_placeholder();
                                let code_block = CodeBlock {
                                    id: Uuid::new_v4().to_string(),
                                    content: json_text.to_string(),
                                    language: Some("json".to_string()),
                                    block_type: CodeBlockType::JsonData,
                                };

                                self.preserved_blocks
                                    .insert(placeholder.clone(), code_block);
                                result = result.replace(json_text, &placeholder);
                                break; // 重新开始扫描
                            }
                        }
                        start_pos = None;
                    }
                }
                _ => {}
            }
        }

        result
    }

    /// 保护HTML标签 (例如: <div>, <span>)
    fn protect_html_tags(&mut self, content: String) -> String {
        let mut result = content;

        // 匹配HTML标签模式 <tag> 或 <tag/>
        let mut pos = 0;
        while let Some(start) = result[pos..].find('<') {
            let actual_start = pos + start;

            if let Some(end) = result[actual_start..].find('>') {
                let actual_end = actual_start + end + 1;
                let tag = &result[actual_start..actual_end];

                // 检查是否是有效的HTML标签
                if tag.len() > 2 && tag.chars().nth(1).unwrap_or(' ').is_alphabetic() {
                    let placeholder = self.create_placeholder();
                    let code_block = CodeBlock {
                        id: Uuid::new_v4().to_string(),
                        content: tag.to_string(),
                        language: Some("html".to_string()),
                        block_type: CodeBlockType::HtmlTag,
                    };

                    self.preserved_blocks
                        .insert(placeholder.clone(), code_block);
                    result = result.replace(tag, &placeholder);
                    pos = actual_start + placeholder.len();
                } else {
                    pos = actual_end;
                }
            } else {
                pos = actual_start + 1;
            }
        }

        result
    }
}

#[derive(Debug, Clone, Default)]
pub struct CodeProtectionStats {
    pub fenced_blocks: usize,
    pub inline_blocks: usize,
    pub html_code_blocks: usize,
    pub pre_blocks: usize,
    pub script_blocks: usize,
    pub style_blocks: usize,

    // 新增的保护内容统计
    pub api_endpoints: usize,
    pub file_paths: usize,
    pub env_variables: usize,
    pub version_numbers: usize,
    pub function_names: usize,
    pub config_keys: usize,
    pub command_lines: usize,
    pub json_data: usize,
    pub sql_queries: usize,
    pub regex_patterns: usize,
    pub html_tags: usize,
    pub markdown_links: usize,
    pub technical_terms: usize,
}

impl CodeProtectionStats {
    pub fn total_blocks(&self) -> usize {
        self.fenced_blocks
            + self.inline_blocks
            + self.html_code_blocks
            + self.pre_blocks
            + self.script_blocks
            + self.style_blocks
            + self.api_endpoints
            + self.file_paths
            + self.env_variables
            + self.version_numbers
            + self.function_names
            + self.config_keys
            + self.command_lines
            + self.json_data
            + self.sql_queries
            + self.regex_patterns
            + self.html_tags
            + self.markdown_links
            + self.technical_terms
    }

    pub fn get_summary(&self) -> String {
        if self.total_blocks() == 0 {
            "未检测到需要保护的内容".to_string()
        } else {
            let mut parts = Vec::new();

            // 原有的代码块类型
            if self.fenced_blocks > 0 {
                parts.push(format!("代码块: {}", self.fenced_blocks));
            }
            if self.inline_blocks > 0 {
                parts.push(format!("行内代码: {}", self.inline_blocks));
            }
            if self.html_code_blocks > 0 {
                parts.push(format!("HTML代码: {}", self.html_code_blocks));
            }
            if self.pre_blocks > 0 {
                parts.push(format!("预格式化: {}", self.pre_blocks));
            }
            if self.script_blocks > 0 {
                parts.push(format!("脚本: {}", self.script_blocks));
            }
            if self.style_blocks > 0 {
                parts.push(format!("样式: {}", self.style_blocks));
            }

            // 新增的保护内容类型
            if self.api_endpoints > 0 {
                parts.push(format!("API端点: {}", self.api_endpoints));
            }
            if self.file_paths > 0 {
                parts.push(format!("文件路径: {}", self.file_paths));
            }
            if self.env_variables > 0 {
                parts.push(format!("环境变量: {}", self.env_variables));
            }
            if self.version_numbers > 0 {
                parts.push(format!("版本号: {}", self.version_numbers));
            }
            if self.function_names > 0 {
                parts.push(format!("函数名: {}", self.function_names));
            }
            if self.config_keys > 0 {
                parts.push(format!("配置键: {}", self.config_keys));
            }
            if self.command_lines > 0 {
                parts.push(format!("命令行: {}", self.command_lines));
            }
            if self.json_data > 0 {
                parts.push(format!("JSON数据: {}", self.json_data));
            }
            if self.sql_queries > 0 {
                parts.push(format!("SQL查询: {}", self.sql_queries));
            }
            if self.regex_patterns > 0 {
                parts.push(format!("正则表达式: {}", self.regex_patterns));
            }
            if self.html_tags > 0 {
                parts.push(format!("HTML标签: {}", self.html_tags));
            }
            if self.markdown_links > 0 {
                parts.push(format!("Markdown链接: {}", self.markdown_links));
            }
            if self.technical_terms > 0 {
                parts.push(format!("技术术语: {}", self.technical_terms));
            }

            format!(
                "已保护 {} 个内容项 ({})",
                self.total_blocks(),
                parts.join(", ")
            )
        }
    }
}
