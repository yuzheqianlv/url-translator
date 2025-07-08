use url_translator::services::content_processor::ContentProcessor;

fn main() {
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

    println!("Original content:");
    println!("{}", content);
    
    let protected = processor.protect_code_blocks(content);
    println!("\nProtected content:");
    println!("{}", protected);
    
    let stats = processor.get_protection_stats();
    println!("\nStats: {}", stats.get_summary());
    
    println!("\nContains 'fn test() {}': {}", protected.contains("fn test() {}"));
}