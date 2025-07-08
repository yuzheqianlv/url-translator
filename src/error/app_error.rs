use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AppError {
    #[error("网络请求失败: {message}")]
    NetworkError { message: String },
    
    #[error("API调用失败: {service} - {message}")]
    ApiError { service: String, message: String },
    
    #[error("翻译服务错误: {message}")]
    TranslationError { message: String },
    
    #[error("内容提取失败: {message}")]
    ExtractionError { message: String },
    
    #[error("配置错误: {message}")]
    ConfigError { message: String },
    
    #[error("验证错误: {field} - {message}")]
    ValidationError { field: String, message: String },
    
    #[error("速率限制错误: {message}")]
    RateLimitError { message: String },
    
    #[error("文件操作错误: {message}")]
    FileError { message: String },
    
    #[error("解析错误: {message}")]
    ParseError { message: String },
    
    #[error("未知错误: {message}")]
    Unknown { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    Low,      // 用户可以继续操作
    Medium,   // 需要用户注意但不阻塞
    High,     // 阻塞当前操作
    Critical, // 应用程序级错误
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorContext {
    pub error: AppError,
    pub severity: ErrorSeverity,
    pub timestamp: f64,
    pub retry_count: u32,
    pub user_message: String,
    pub technical_details: Option<String>,
    pub suggested_actions: Vec<String>,
}

impl AppError {
    pub fn network(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }
    
    pub fn api(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ApiError {
            service: service.into(),
            message: message.into(),
        }
    }
    
    pub fn translation(message: impl Into<String>) -> Self {
        Self::TranslationError {
            message: message.into(),
        }
    }
    
    pub fn extraction(message: impl Into<String>) -> Self {
        Self::ExtractionError {
            message: message.into(),
        }
    }
    
    pub fn config(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }
    
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
    
    pub fn rate_limit(message: impl Into<String>) -> Self {
        Self::RateLimitError {
            message: message.into(),
        }
    }
    
    pub fn file(message: impl Into<String>) -> Self {
        Self::FileError {
            message: message.into(),
        }
    }
    
    pub fn parse(message: impl Into<String>) -> Self {
        Self::ParseError {
            message: message.into(),
        }
    }
    
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }
    
    /// 获取错误的严重程度
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::NetworkError { .. } => ErrorSeverity::Medium,
            AppError::ApiError { .. } => ErrorSeverity::High,
            AppError::TranslationError { .. } => ErrorSeverity::High,
            AppError::ExtractionError { .. } => ErrorSeverity::High,
            AppError::ConfigError { .. } => ErrorSeverity::Critical,
            AppError::ValidationError { .. } => ErrorSeverity::Medium,
            AppError::RateLimitError { .. } => ErrorSeverity::Low,
            AppError::FileError { .. } => ErrorSeverity::Medium,
            AppError::ParseError { .. } => ErrorSeverity::Medium,
            AppError::Unknown { .. } => ErrorSeverity::High,
        }
    }
    
    /// 获取用户友好的错误消息
    pub fn user_message(&self) -> String {
        match self {
            AppError::NetworkError { .. } => "网络连接失败，请检查网络设置后重试".to_string(),
            AppError::ApiError { service, .. } => format!("{} 服务暂时不可用，请稍后重试", service),
            AppError::TranslationError { .. } => "翻译失败，请检查文本内容或稍后重试".to_string(),
            AppError::ExtractionError { .. } => "内容提取失败，请检查URL是否正确".to_string(),
            AppError::ConfigError { .. } => "配置错误，请检查设置页面".to_string(),
            AppError::ValidationError { field, .. } => format!("{} 格式不正确，请重新输入", field),
            AppError::RateLimitError { .. } => "请求过于频繁，请稍后重试".to_string(),
            AppError::FileError { .. } => "文件操作失败".to_string(),
            AppError::ParseError { .. } => "数据解析失败".to_string(),
            AppError::Unknown { .. } => "发生未知错误，请稍后重试".to_string(),
        }
    }
    
    /// 获取建议的操作
    pub fn suggested_actions(&self) -> Vec<String> {
        match self {
            AppError::NetworkError { .. } => vec![
                "检查网络连接".to_string(),
                "刷新页面重试".to_string(),
            ],
            AppError::ApiError { service, .. } => vec![
                format!("检查 {} 服务配置", service),
                "稍后重试".to_string(),
                "联系管理员".to_string(),
            ],
            AppError::TranslationError { .. } => vec![
                "检查文本内容".to_string(),
                "尝试分段翻译".to_string(),
                "检查API配置".to_string(),
            ],
            AppError::ExtractionError { .. } => vec![
                "验证URL格式".to_string(),
                "检查网页是否可访问".to_string(),
                "尝试其他URL".to_string(),
            ],
            AppError::ConfigError { .. } => vec![
                "前往设置页面检查配置".to_string(),
                "重置为默认配置".to_string(),
            ],
            AppError::ValidationError { .. } => vec![
                "检查输入格式".to_string(),
                "参考示例输入".to_string(),
            ],
            AppError::RateLimitError { .. } => vec![
                "等待一段时间后重试".to_string(),
                "降低请求频率".to_string(),
            ],
            AppError::FileError { .. } => vec![
                "检查文件权限".to_string(),
                "尝试重新操作".to_string(),
            ],
            AppError::ParseError { .. } => vec![
                "检查数据格式".to_string(),
                "刷新页面重试".to_string(),
            ],
            AppError::Unknown { .. } => vec![
                "刷新页面重试".to_string(),
                "联系技术支持".to_string(),
            ],
        }
    }
    
    /// 判断错误是否可以重试
    pub fn is_retryable(&self) -> bool {
        match self {
            AppError::NetworkError { .. } => true,
            AppError::ApiError { .. } => true,
            AppError::TranslationError { .. } => true,
            AppError::ExtractionError { .. } => false,
            AppError::ConfigError { .. } => false,
            AppError::ValidationError { .. } => false,
            AppError::RateLimitError { .. } => true,
            AppError::FileError { .. } => true,
            AppError::ParseError { .. } => false,
            AppError::Unknown { .. } => true,
        }
    }
}

impl ErrorContext {
    pub fn new(error: AppError) -> Self {
        let severity = error.severity();
        let user_message = error.user_message();
        let suggested_actions = error.suggested_actions();
        
        Self {
            error,
            severity,
            timestamp: js_sys::Date::now(),
            retry_count: 0,
            user_message,
            technical_details: None,
            suggested_actions,
        }
    }
    
    pub fn with_technical_details(mut self, details: impl Into<String>) -> Self {
        self.technical_details = Some(details.into());
        self
    }
    
    pub fn increment_retry(mut self) -> Self {
        self.retry_count += 1;
        self
    }
    
    pub fn can_retry(&self, max_retries: u32) -> bool {
        self.error.is_retryable() && self.retry_count < max_retries
    }
}

// 便于从其他错误类型转换
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::network("请求超时")
        } else if err.is_connect() {
            AppError::network("连接失败")
        } else {
            AppError::network(format!("网络错误: {}", err))
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::parse(format!("JSON解析失败: {}", err))
    }
}

impl From<gloo_storage::errors::StorageError> for AppError {
    fn from(err: gloo_storage::errors::StorageError) -> Self {
        AppError::config(format!("存储错误: {}", err))
    }
}

pub type AppResult<T> = Result<T, AppError>;