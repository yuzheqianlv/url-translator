use url_translator::services::content_processor::ContentProcessor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protect_fenced_code_blocks() {
        let mut processor = ContentProcessor::new();
        let content = r#"
这是一些文本。

```rust
fn main() {
    println!("Hello, world!");
}
```

这是更多文本。

```javascript
console.log("Hello");
```

结束文本。
"#;

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        assert_eq!(stats.fenced_blocks, 2);
        assert_eq!(stats.total_blocks(), 2);
        
        // 检查代码块被替换为占位符
        assert!(!protected.contains("fn main()"));
        assert!(!protected.contains("console.log"));
        assert!(protected.contains("这是一些文本"));
        assert!(protected.contains("__CODE_BLOCK_"));
        
        // 恢复代码块
        let restored = processor.restore_code_blocks(&protected);
        assert!(restored.contains("fn main()"));
        assert!(restored.contains("console.log"));
    }

    #[test]
    fn test_protect_inline_code() {
        let mut processor = ContentProcessor::new();
        let content = "这里有一些 `inline code` 和另一个 `variable` 在句子中。";

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        assert_eq!(stats.inline_blocks, 2);
        assert!(!protected.contains("`inline code`"));
        assert!(!protected.contains("`variable`"));
        assert!(protected.contains("__CODE_BLOCK_"));
        
        let restored = processor.restore_code_blocks(&protected);
        assert!(restored.contains("`inline code`"));
        assert!(restored.contains("`variable`"));
    }

    #[test]
    fn test_protect_html_code_tags() {
        let mut processor = ContentProcessor::new();
        let content = r#"
<p>这是一段文本，包含 <code>some code</code> 标签。</p>
<pre>
formatted text
</pre>
<script>
console.log("script");
</script>
"#;

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        assert!(stats.html_code_blocks > 0 || stats.pre_blocks > 0 || stats.script_blocks > 0);
        assert!(!protected.contains("<code>some code</code>"));
        assert!(!protected.contains("console.log(\"script\")"));
        
        let restored = processor.restore_code_blocks(&protected);
        assert!(restored.contains("<code>some code</code>"));
        assert!(restored.contains("console.log(\"script\")"));
    }

    #[test]
    fn test_mixed_code_blocks() {
        let mut processor = ContentProcessor::new();
        let content = r#"
# 标题

这是文档内容，包含 `inline code` 示例。

```python
def hello():
    print("Hello")
```

还有HTML代码：<code>document.getElementById</code>

<pre>
预格式化文本
</pre>
"#;

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        assert!(stats.total_blocks() >= 4); // 至少应该有4个代码块
        assert!(!protected.contains("`inline code`"));
        assert!(!protected.contains("def hello():"));
        assert!(!protected.contains("document.getElementById"));
        assert!(!protected.contains("预格式化文本"));
        
        // 普通文本应该保留
        assert!(protected.contains("# 标题"));
        assert!(protected.contains("这是文档内容"));
        assert!(protected.contains("还有HTML代码"));
        
        let restored = processor.restore_code_blocks(&protected);
        assert!(restored.contains("`inline code`"));
        assert!(restored.contains("def hello():"));
        assert!(restored.contains("document.getElementById"));
        assert!(restored.contains("预格式化文本"));
    }

    #[test]
    fn test_no_code_blocks() {
        let mut processor = ContentProcessor::new();
        let content = "这是普通文本，没有任何代码块。只是一些正常的段落内容。";

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        assert_eq!(stats.total_blocks(), 0);
        assert_eq!(protected, content); // 内容应该不变
        
        let restored = processor.restore_code_blocks(&protected);
        assert_eq!(restored, content);
    }

    #[test]
    fn test_nested_code_blocks() {
        let mut processor = ContentProcessor::new();
        let content = r#"
```markdown
# 这是markdown

这里有 `inline code` 在代码块内。

```rust
fn test() {}
```
```
"#;

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        // 应该至少保护一个代码块
        assert!(stats.fenced_blocks >= 1);
        
        // 原始内容应该被保护（可能作为外层块，也可能作为内层块）
        // 关键是恢复后内容完整
        let restored = processor.restore_code_blocks(&protected);
        assert!(restored.contains("fn test() {}"));
        assert!(restored.contains("这里有"));
        assert!(restored.contains("# 这是markdown"));
    }

    #[test]
    fn test_code_block_with_language() {
        let mut processor = ContentProcessor::new();
        let content = r#"
```rust
fn main() {
    println!("Hello");
}
```

```javascript
console.log("Hello");
```

```
无语言标识的代码块
```
"#;

        let protected = processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        
        assert_eq!(stats.fenced_blocks, 3);
        
        // 检查代码块是否被正确保护
        let protected_blocks = processor.get_protection_stats();
        assert_eq!(protected_blocks.fenced_blocks, 3);
    }

    #[test]
    fn test_protection_stats_summary() {
        let mut processor = ContentProcessor::new();
        let content = r#"
```rust
fn main() {}
```

这里有 `inline` 代码和 <code>html code</code>。

<pre>预格式化</pre>
<script>console.log("test");</script>
"#;

        processor.protect_code_blocks(content);
        let stats = processor.get_protection_stats();
        let summary = stats.get_summary();
        
        assert!(summary.contains("已保护"));
        assert!(summary.contains("个代码块"));
        assert!(stats.total_blocks() > 0);
    }
}