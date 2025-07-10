//! 环境变量管理
//! 
//! 提供编译时和运行时环境变量访问

use std::str::FromStr;

/// 获取编译时环境变量，带有默认值
macro_rules! env_var {
    ($name:expr, $default:expr) => {
        option_env!($name).unwrap_or($default)
    };
}

/// 环境变量配置结构
#[derive(Debug, Clone)]
pub struct EnvConfig {
    /// API基础URL
    pub api_base_url: String,
    /// API超时时间（秒）
    pub api_timeout_seconds: u64,
    /// 是否启用项目管理
    pub enable_project_management: bool,
    /// 是否启用历史记录
    pub enable_history: bool,
    /// 是否启用搜索功能
    pub enable_search: bool,
    /// 是否启用批量翻译
    pub enable_batch_translation: bool,
    /// 默认主题
    pub default_theme: String,
    /// 是否启用调试模式
    pub debug_mode: bool,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            api_base_url: env_var!("FRONTEND_API_BASE_URL", "http://localhost:3002/api/v1").to_string(),
            api_timeout_seconds: parse_env_var("FRONTEND_API_TIMEOUT_SECONDS", 30),
            enable_project_management: parse_env_var("ENABLE_PROJECT_MANAGEMENT", true),
            enable_history: parse_env_var("ENABLE_HISTORY", true),
            enable_search: parse_env_var("ENABLE_SEARCH", true),
            enable_batch_translation: parse_env_var("ENABLE_BATCH_TRANSLATION", true),
            default_theme: env_var!("DEFAULT_THEME", "latte").to_string(),
            debug_mode: parse_env_var("DEBUG_MODE", true),
        }
    }
}

impl EnvConfig {
    /// 创建新的环境配置实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取全局环境配置实例
    pub fn global() -> &'static EnvConfig {
        static CONFIG: std::sync::OnceLock<EnvConfig> = std::sync::OnceLock::new();
        CONFIG.get_or_init(EnvConfig::new)
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.api_base_url.is_empty() {
            return Err("API基础URL不能为空".to_string());
        }

        if !self.api_base_url.starts_with("http://") && !self.api_base_url.starts_with("https://") {
            return Err("API基础URL必须以http://或https://开头".to_string());
        }

        if self.api_timeout_seconds == 0 {
            return Err("API超时时间必须大于0".to_string());
        }

        if self.api_timeout_seconds > 300 {
            return Err("API超时时间不应超过300秒".to_string());
        }

        let valid_themes = ["latte", "frappe", "macchiato", "mocha"];
        if !valid_themes.contains(&self.default_theme.as_str()) {
            return Err(format!("无效的默认主题: {}，支持的主题: {:?}", self.default_theme, valid_themes));
        }

        Ok(())
    }

    /// 生成配置摘要用于调试
    pub fn summary(&self) -> String {
        format!(
            "环境配置摘要:\n\
            - API地址: {}\n\
            - API超时: {}秒\n\
            - 项目管理: {}\n\
            - 历史记录: {}\n\
            - 搜索功能: {}\n\
            - 批量翻译: {}\n\
            - 默认主题: {}\n\
            - 调试模式: {}",
            self.api_base_url,
            self.api_timeout_seconds,
            if self.enable_project_management { "启用" } else { "禁用" },
            if self.enable_history { "启用" } else { "禁用" },
            if self.enable_search { "启用" } else { "禁用" },
            if self.enable_batch_translation { "启用" } else { "禁用" },
            self.default_theme,
            if self.debug_mode { "启用" } else { "禁用" }
        )
    }
}

/// 解析环境变量为指定类型，失败时返回默认值
fn parse_env_var<T>(name: &'static str, default: T) -> T
where
    T: FromStr + Clone,
{
    match name {
        "FRONTEND_API_TIMEOUT_SECONDS" => {
            option_env!("FRONTEND_API_TIMEOUT_SECONDS")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        "ENABLE_PROJECT_MANAGEMENT" => {
            option_env!("ENABLE_PROJECT_MANAGEMENT")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        "ENABLE_HISTORY" => {
            option_env!("ENABLE_HISTORY")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        "ENABLE_SEARCH" => {
            option_env!("ENABLE_SEARCH")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        "ENABLE_BATCH_TRANSLATION" => {
            option_env!("ENABLE_BATCH_TRANSLATION")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        "DEBUG_MODE" => {
            option_env!("DEBUG_MODE")
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        _ => default,
    }
}

/// 编译时环境变量访问器
pub struct CompileTimeEnv;

impl CompileTimeEnv {
    /// 获取API基础URL
    pub fn api_base_url() -> &'static str {
        env_var!("FRONTEND_API_BASE_URL", "http://localhost:3002/api/v1")
    }

    /// 获取API超时时间
    pub fn api_timeout_seconds() -> u64 {
        parse_env_var("FRONTEND_API_TIMEOUT_SECONDS", 30)
    }

    /// 检查是否启用项目管理
    pub fn is_project_management_enabled() -> bool {
        parse_env_var("ENABLE_PROJECT_MANAGEMENT", true)
    }

    /// 检查是否启用历史记录
    pub fn is_history_enabled() -> bool {
        parse_env_var("ENABLE_HISTORY", true)
    }

    /// 检查是否启用搜索功能
    pub fn is_search_enabled() -> bool {
        parse_env_var("ENABLE_SEARCH", true)
    }

    /// 检查是否启用批量翻译
    pub fn is_batch_translation_enabled() -> bool {
        parse_env_var("ENABLE_BATCH_TRANSLATION", true)
    }

    /// 获取默认主题
    pub fn default_theme() -> &'static str {
        env_var!("DEFAULT_THEME", "latte")
    }

    /// 检查是否启用调试模式
    pub fn is_debug_mode() -> bool {
        parse_env_var("DEBUG_MODE", true)
    }
}

/// 运行时环境变量检查器
pub struct RuntimeEnv;

impl RuntimeEnv {
    /// 检查是否在开发环境
    pub fn is_development() -> bool {
        cfg!(debug_assertions) || CompileTimeEnv::is_debug_mode()
    }

    /// 检查是否在生产环境
    pub fn is_production() -> bool {
        !Self::is_development()
    }

    /// 获取构建信息
    pub fn build_info() -> String {
        format!(
            "构建信息:\n\
            - 版本: {}\n\
            - 构建时间: {}\n\
            - 构建模式: {}\n\
            - Target: wasm32-unknown-unknown",
            env!("CARGO_PKG_VERSION"),
            env!("BUILD_TIMESTAMP", "unknown"),
            if Self::is_development() { "开发" } else { "生产" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_config_default() {
        let config = EnvConfig::default();
        assert!(!config.api_base_url.is_empty());
        assert!(config.api_timeout_seconds > 0);
    }

    #[test]
    fn test_env_config_validation() {
        let mut config = EnvConfig::default();
        assert!(config.validate().is_ok());

        config.api_base_url = String::new();
        assert!(config.validate().is_err());

        config.api_base_url = "invalid-url".to_string();
        assert!(config.validate().is_err());

        config.api_base_url = "http://localhost:3002".to_string();
        config.api_timeout_seconds = 0;
        assert!(config.validate().is_err());

        config.api_timeout_seconds = 30;
        config.default_theme = "invalid-theme".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_compile_time_env() {
        assert!(!CompileTimeEnv::api_base_url().is_empty());
        assert!(CompileTimeEnv::api_timeout_seconds() > 0);
        assert!(!CompileTimeEnv::default_theme().is_empty());
    }

    #[test]
    fn test_runtime_env() {
        assert!(RuntimeEnv::is_development() || RuntimeEnv::is_production());
        assert!(!RuntimeEnv::build_info().is_empty());
    }
}