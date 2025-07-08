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
    FencedCode,              // ```language
    InlineCode,              // `code`
    HtmlCode,                // <code>
    PreFormatted,            // <pre>
    ScriptTag,               // <script>
    StyleTag,                // <style>
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
    
    /// 处理内容，保护代码块不被翻译
    pub fn protect_code_blocks(&mut self, content: &str) -> String {
        let mut processed_content = content.to_string();
        
        // 按顺序处理各种代码块
        processed_content = self.protect_fenced_code_blocks(processed_content);
        processed_content = self.protect_inline_code(processed_content);
        processed_content = self.protect_html_code_elements(processed_content);
        
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
                    Some(result[actual_start + 3..actual_start + first_line_end].trim().to_string())
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
                
                self.preserved_blocks.insert(placeholder.clone(), code_block);
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
                    
                    self.preserved_blocks.insert(placeholder.clone(), code_block);
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
    fn protect_html_tag(&mut self, content: String, tag: &str, block_type: CodeBlockType) -> String {
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
                    
                    self.preserved_blocks.insert(placeholder.clone(), code_block);
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
}

#[derive(Debug, Clone, Default)]
pub struct CodeProtectionStats {
    pub fenced_blocks: usize,
    pub inline_blocks: usize,
    pub html_code_blocks: usize,
    pub pre_blocks: usize,
    pub script_blocks: usize,
    pub style_blocks: usize,
}

impl CodeProtectionStats {
    pub fn total_blocks(&self) -> usize {
        self.fenced_blocks + self.inline_blocks + self.html_code_blocks + 
        self.pre_blocks + self.script_blocks + self.style_blocks
    }
    
    pub fn get_summary(&self) -> String {
        if self.total_blocks() == 0 {
            "未检测到代码块".to_string()
        } else {
            let mut parts = Vec::new();
            
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
            
            format!("已保护 {} 个代码块 ({})", self.total_blocks(), parts.join(", "))
        }
    }
}