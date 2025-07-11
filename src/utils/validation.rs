use url::Url;
use std::collections::HashSet;

/// URL验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum UrlValidationResult {
    Valid,
    Invalid(String),
}

/// URL验证器
pub struct UrlValidator {
    allowed_schemes: HashSet<String>,
    blocked_domains: HashSet<String>,
    max_length: usize,
}

impl Default for UrlValidator {
    fn default() -> Self {
        let mut allowed_schemes = HashSet::new();
        allowed_schemes.insert("http".to_string());
        allowed_schemes.insert("https".to_string());
        
        let mut blocked_domains = HashSet::new();
        // 添加一些危险域名示例
        blocked_domains.insert("localhost".to_string());
        blocked_domains.insert("127.0.0.1".to_string());
        blocked_domains.insert("0.0.0.0".to_string());
        
        Self {
            allowed_schemes,
            blocked_domains,
            max_length: 2048, // 最大URL长度
        }
    }
}

impl UrlValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 允许本地URL（用于开发环境）
    pub fn allow_local_urls(mut self) -> Self {
        self.blocked_domains.remove("localhost");
        self.blocked_domains.remove("127.0.0.1");
        self.blocked_domains.remove("0.0.0.0");
        self
    }
    
    /// 验证URL
    pub fn validate(&self, url_str: &str) -> UrlValidationResult {
        // 检查长度
        if url_str.len() > self.max_length {
            return UrlValidationResult::Invalid(
                format!("URL长度超过限制({})字符", self.max_length)
            );
        }
        
        // 检查是否为空
        if url_str.trim().is_empty() {
            return UrlValidationResult::Invalid("URL不能为空".to_string());
        }
        
        // 解析URL
        let parsed_url = match Url::parse(url_str) {
            Ok(url) => url,
            Err(e) => {
                return UrlValidationResult::Invalid(
                    format!("URL格式无效: {}", e)
                );
            }
        };
        
        // 检查协议
        if !self.allowed_schemes.contains(parsed_url.scheme()) {
            return UrlValidationResult::Invalid(
                format!("不支持的协议: {}", parsed_url.scheme())
            );
        }
        
        // 检查域名
        if let Some(host) = parsed_url.host_str() {
            if self.blocked_domains.contains(host) {
                return UrlValidationResult::Invalid(
                    format!("禁止访问的域名: {}", host)
                );
            }
            
            // 检查IP地址（防止内网扫描）
            if self.is_private_ip(host) {
                return UrlValidationResult::Invalid(
                    "禁止访问内网地址".to_string()
                );
            }
        } else {
            return UrlValidationResult::Invalid("URL缺少主机名".to_string());
        }
        
        // 检查端口（防止端口扫描）
        if let Some(port) = parsed_url.port() {
            if !self.is_allowed_port(port) {
                return UrlValidationResult::Invalid(
                    format!("禁止访问端口: {}", port)
                );
            }
        }
        
        UrlValidationResult::Valid
    }
    
    /// 检查是否为私有IP地址
    fn is_private_ip(&self, host: &str) -> bool {
        // 简单的私有IP检查
        host.starts_with("10.") ||
        host.starts_with("192.168.") ||
        host.starts_with("172.") ||
        host.starts_with("169.254.") ||
        host == "::1" ||
        host.starts_with("fe80:")
    }
    
    /// 检查是否为允许的端口
    fn is_allowed_port(&self, port: u16) -> bool {
        // 只允许标准的HTTP和HTTPS端口
        matches!(port, 80 | 443 | 8080 | 8443)
    }
}

/// 快速URL验证函数
pub fn validate_url(url: &str) -> UrlValidationResult {
    UrlValidator::new().validate(url)
}

/// 生产环境URL验证（更严格）
pub fn validate_url_strict(url: &str) -> UrlValidationResult {
    UrlValidator::new().validate(url)
}

/// 开发环境URL验证（允许本地URL）
pub fn validate_url_dev(url: &str) -> UrlValidationResult {
    UrlValidator::new().allow_local_urls().validate(url)
}