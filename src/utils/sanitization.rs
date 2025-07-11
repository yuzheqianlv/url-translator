/// 文本清理和防XSS工具

/// HTML字符转义
/// 防止XSS攻击的基本措施
pub fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('/', "&#x2F;")
}

/// 清理用户输入
/// 移除危险字符和控制字符
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            // 保留可打印字符和常用空白字符
            c.is_ascii_graphic() || c.is_ascii_whitespace() || !c.is_ascii()
        })
        .filter(|c| {
            // 过滤掉危险的控制字符
            !matches!(c, '\x00'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f' | '\x7f')
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// 清理URL输入
/// 移除不必要的空白和危险字符
pub fn sanitize_url(url: &str) -> String {
    url.trim()
        .chars()
        .filter(|c| {
            // URL只允许特定字符
            c.is_ascii_alphanumeric() ||
            matches!(c, '-' | '.' | '_' | '~' | ':' | '/' | '?' | '#' | '[' | ']' | '@' | 
                        '!' | '$' | '&' | '\'' | '(' | ')' | '*' | '+' | ',' | ';' | '=' | '%')
        })
        .collect()
}

/// 清理文件名
/// 确保文件名安全且兼容
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .filter(|c| {
            // 只允许安全的文件名字符
            c.is_ascii_alphanumeric() ||
            matches!(c, '-' | '_' | '.' | ' ' | '(' | ')' | '[' | ']')
        })
        .collect::<String>()
        .trim()
        .replace("  ", " ") // 替换多个空格
        .trim_matches('.')
        .to_string()
}

/// 限制字符串长度
pub fn limit_length(input: &str, max_length: usize) -> String {
    if input.len() <= max_length {
        input.to_string()
    } else {
        let mut result = String::new();
        let mut current_length = 0;
        
        for char in input.chars() {
            let char_len = char.len_utf8();
            if current_length + char_len > max_length {
                break;
            }
            result.push(char);
            current_length += char_len;
        }
        
        result
    }
}

/// 综合文本清理函数
pub fn sanitize_text(input: &str, max_length: Option<usize>) -> String {
    let cleaned = sanitize_input(input);
    
    if let Some(max_len) = max_length {
        limit_length(&cleaned, max_len)
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html() {
        assert_eq!(escape_html("<script>alert('xss')</script>"), "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;&#x2F;script&gt;");
        assert_eq!(escape_html("Hello & goodbye"), "Hello &amp; goodbye");
    }

    #[test]
    fn test_sanitize_url() {
        assert_eq!(sanitize_url("  https://example.com  "), "https://example.com");
        assert_eq!(sanitize_url("https://example.com/path?q=test#anchor"), "https://example.com/path?q=test#anchor");
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "..etcpasswd");
        assert_eq!(sanitize_filename("normal file.txt"), "normal file.txt");
        assert_eq!(sanitize_filename("file<>:|\"/\\*.txt"), "file.txt");
    }

    #[test]
    fn test_limit_length() {
        assert_eq!(limit_length("hello world", 5), "hello");
        assert_eq!(limit_length("hi", 10), "hi");
        assert_eq!(limit_length("中文测试", 6), "中文"); // 每个中文字符3字节
    }
}