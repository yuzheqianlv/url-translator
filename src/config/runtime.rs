//! 运行时配置管理
//! 
//! 处理运行时可变的配置项，支持本地存储持久化

use leptos::*;
use crate::config::{EnvConfig, FeatureFlags, Feature};
use crate::services::config_service::ConfigService;

/// 运行时配置项
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    /// 当前主题
    pub current_theme: String,
    /// API基础URL（可在运行时修改）
    pub api_base_url: String,
    /// API超时时间
    pub api_timeout_seconds: u64,
    /// 是否启用自动保存
    pub auto_save_enabled: bool,
    /// 自动保存间隔（秒）
    pub auto_save_interval: u64,
    /// 最大文件大小（MB）
    pub max_file_size_mb: u32,
    /// 是否启用键盘快捷键
    pub keyboard_shortcuts_enabled: bool,
    /// 是否启用通知
    pub notifications_enabled: bool,
    /// 语言设置
    pub language: String,
    /// 时区设置
    pub timezone: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let env_config = EnvConfig::global();
        Self {
            current_theme: env_config.default_theme.clone(),
            api_base_url: env_config.api_base_url.clone(),
            api_timeout_seconds: env_config.api_timeout_seconds,
            auto_save_enabled: true,
            auto_save_interval: 30,
            max_file_size_mb: 10,
            keyboard_shortcuts_enabled: true,
            notifications_enabled: true,
            language: "zh-CN".to_string(),
            timezone: "Asia/Shanghai".to_string(),
        }
    }
}

impl RuntimeConfig {
    /// 创建新的运行时配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 从环境配置创建运行时配置
    pub fn from_env(env_config: &EnvConfig) -> Self {
        Self {
            current_theme: env_config.default_theme.clone(),
            api_base_url: env_config.api_base_url.clone(),
            api_timeout_seconds: env_config.api_timeout_seconds,
            ..Self::default()
        }
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.api_base_url.is_empty() {
            return Err("API基础URL不能为空".to_string());
        }

        if !self.api_base_url.starts_with("http://") && !self.api_base_url.starts_with("https://") {
            return Err("API基础URL格式无效".to_string());
        }

        if self.api_timeout_seconds == 0 || self.api_timeout_seconds > 300 {
            return Err("API超时时间应在1-300秒之间".to_string());
        }

        if self.auto_save_interval < 5 || self.auto_save_interval > 300 {
            return Err("自动保存间隔应在5-300秒之间".to_string());
        }

        if self.max_file_size_mb == 0 || self.max_file_size_mb > 100 {
            return Err("文件大小限制应在1-100MB之间".to_string());
        }

        let valid_themes = ["latte", "frappe", "macchiato", "mocha"];
        if !valid_themes.contains(&self.current_theme.as_str()) {
            return Err(format!("无效的主题: {}", self.current_theme));
        }

        Ok(())
    }

    /// 应用主题更改
    pub fn set_theme(&mut self, theme: String) -> Result<(), String> {
        let valid_themes = ["latte", "frappe", "macchiato", "mocha"];
        if !valid_themes.contains(&theme.as_str()) {
            return Err(format!("无效的主题: {}", theme));
        }
        self.current_theme = theme;
        Ok(())
    }

    /// 更新API配置
    pub fn update_api_config(&mut self, base_url: String, timeout_seconds: u64) -> Result<(), String> {
        if base_url.is_empty() {
            return Err("API基础URL不能为空".to_string());
        }
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err("API基础URL格式无效".to_string());
        }
        if timeout_seconds == 0 || timeout_seconds > 300 {
            return Err("API超时时间应在1-300秒之间".to_string());
        }

        self.api_base_url = base_url;
        self.api_timeout_seconds = timeout_seconds;
        Ok(())
    }

    /// 转换为JSON字符串
    pub fn to_json(&self) -> String {
        format!(
            r#"{{
                "current_theme": "{}",
                "api_base_url": "{}",
                "api_timeout_seconds": {},
                "auto_save_enabled": {},
                "auto_save_interval": {},
                "max_file_size_mb": {},
                "keyboard_shortcuts_enabled": {},
                "notifications_enabled": {},
                "language": "{}",
                "timezone": "{}"
            }}"#,
            self.current_theme,
            self.api_base_url,
            self.api_timeout_seconds,
            self.auto_save_enabled,
            self.auto_save_interval,
            self.max_file_size_mb,
            self.keyboard_shortcuts_enabled,
            self.notifications_enabled,
            self.language,
            self.timezone
        )
    }

    /// 从JSON字符串解析配置
    pub fn from_json(json: &str) -> Result<Self, String> {
        // 简化的JSON解析实现
        // 实际项目中建议使用serde_json
        let mut config = Self::default();
        
        if json.contains("\"current_theme\"") {
            if let Some(theme) = extract_json_string_value(json, "current_theme") {
                config.current_theme = theme;
            }
        }
        
        if json.contains("\"api_base_url\"") {
            if let Some(url) = extract_json_string_value(json, "api_base_url") {
                config.api_base_url = url;
            }
        }

        if json.contains("\"api_timeout_seconds\"") {
            if let Some(timeout) = extract_json_number_value(json, "api_timeout_seconds") {
                config.api_timeout_seconds = timeout as u64;
            }
        }

        if json.contains("\"auto_save_enabled\"") {
            if let Some(enabled) = extract_json_bool_value(json, "auto_save_enabled") {
                config.auto_save_enabled = enabled;
            }
        }

        if json.contains("\"auto_save_interval\"") {
            if let Some(interval) = extract_json_number_value(json, "auto_save_interval") {
                config.auto_save_interval = interval as u64;
            }
        }

        config.validate()?;
        Ok(config)
    }

    /// 生成配置摘要
    pub fn summary(&self) -> String {
        format!(
            "运行时配置:\n\
            - 当前主题: {}\n\
            - API地址: {}\n\
            - API超时: {}秒\n\
            - 自动保存: {} ({}秒间隔)\n\
            - 文件大小限制: {}MB\n\
            - 键盘快捷键: {}\n\
            - 通知: {}\n\
            - 语言: {}\n\
            - 时区: {}",
            self.current_theme,
            self.api_base_url,
            self.api_timeout_seconds,
            if self.auto_save_enabled { "启用" } else { "禁用" },
            self.auto_save_interval,
            self.max_file_size_mb,
            if self.keyboard_shortcuts_enabled { "启用" } else { "禁用" },
            if self.notifications_enabled { "启用" } else { "禁用" },
            self.language,
            self.timezone
        )
    }
}

/// 应用配置管理器，整合环境配置、功能开关和运行时配置
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 环境配置（只读）
    pub env: EnvConfig,
    /// 功能开关
    pub features: FeatureFlags,
    /// 运行时配置
    pub runtime: RuntimeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        let env = EnvConfig::global().clone();
        let features = FeatureFlags::new();
        let runtime = RuntimeConfig::from_env(&env);

        Self { env, features, runtime }
    }
}

impl AppConfig {
    /// 创建新的应用配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 从本地存储加载配置
    pub async fn load_from_storage() -> Self {
        let mut config = Self::new();
        
        // 尝试从本地存储加载运行时配置
        if let Ok(runtime_json) = ConfigService::get_string("runtime_config").await {
            if let Ok(runtime) = RuntimeConfig::from_json(&runtime_json) {
                config.runtime = runtime;
            }
        }

        // 尝试从本地存储加载功能开关
        if let Ok(features_json) = ConfigService::get_string("feature_flags").await {
            let _ = config.features.from_json(&features_json);
        }

        config
    }

    /// 保存配置到本地存储
    pub async fn save_to_storage(&self) -> Result<(), String> {
        ConfigService::set_string("runtime_config", &self.runtime.to_json()).await
            .map_err(|e| e.to_string())?;
        ConfigService::set_string("feature_flags", &self.features.to_json()).await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 重置配置为默认值
    pub fn reset_to_defaults(&mut self) {
        self.features.reset_to_defaults();
        self.runtime = RuntimeConfig::from_env(&self.env);
    }

    /// 验证所有配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        self.env.validate()?;
        self.runtime.validate()?;
        Ok(())
    }

    /// 检查功能是否启用且可用
    pub fn is_feature_available(&self, feature: &Feature) -> bool {
        self.features.is_enabled(feature)
    }

    /// 生成完整的配置报告
    pub fn full_report(&self) -> String {
        format!(
            "=== 应用配置报告 ===\n\n\
            {}\n\n\
            {}\n\n\
            {}",
            self.env.summary(),
            self.features.summary(),
            self.runtime.summary()
        )
    }
}

/// Leptos Hook：使用应用配置
pub fn use_app_config() -> (ReadSignal<AppConfig>, WriteSignal<AppConfig>) {
    let (config, set_config) = create_signal(AppConfig::new());

    // 初始化时从本地存储加载配置
    create_effect(move |_| {
        spawn_local(async move {
            let loaded_config = AppConfig::load_from_storage().await;
            set_config.set(loaded_config);
        });
    });

    (config, set_config)
}

/// Leptos Hook：使用运行时配置
pub fn use_runtime_config() -> (ReadSignal<RuntimeConfig>, WriteSignal<RuntimeConfig>) {
    let (config, _set_config) = use_app_config();
    
    let (runtime_config, set_runtime_config) = create_signal(config.get().runtime);
    
    // 监听配置变化并更新运行时配置
    create_effect({
        let set_runtime_config = set_runtime_config;
        move |_| {
            set_runtime_config.set(config.get().runtime);
        }
    });
    
    // 这里我们返回 set_runtime_config 用于外部直接更新
    // 当外部调用 set_runtime_config 时，同时更新主配置

    (runtime_config, set_runtime_config)
}

// JSON解析辅助函数
fn extract_json_string_value(json: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":", key);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(quote_start) = json[value_start..].find('"') {
            let quote_start = value_start + quote_start + 1;
            if let Some(quote_end) = json[quote_start..].find('"') {
                return Some(json[quote_start..quote_start + quote_end].to_string());
            }
        }
    }
    None
}

fn extract_json_number_value(json: &str, key: &str) -> Option<f64> {
    let pattern = format!("\"{}\":", key);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        let remaining = &json[value_start..].trim_start();
        let mut end = 0;
        for (i, c) in remaining.char_indices() {
            if c.is_ascii_digit() || c == '.' {
                end = i + 1;
            } else {
                break;
            }
        }
        if end > 0 {
            return remaining[..end].parse().ok();
        }
    }
    None
}

fn extract_json_bool_value(json: &str, key: &str) -> Option<bool> {
    let pattern = format!("\"{}\":", key);
    if let Some(start) = json.find(&pattern) {
        let value_start = start + pattern.len();
        let remaining = &json[value_start..].trim_start();
        if remaining.starts_with("true") {
            return Some(true);
        } else if remaining.starts_with("false") {
            return Some(false);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_config_validation() {
        let mut config = RuntimeConfig::new();
        assert!(config.validate().is_ok());

        config.api_base_url = "invalid-url".to_string();
        assert!(config.validate().is_err());

        config.api_base_url = "http://localhost:3002".to_string();
        config.api_timeout_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_runtime_config_theme_setting() {
        let mut config = RuntimeConfig::new();
        assert!(config.set_theme("mocha".to_string()).is_ok());
        assert_eq!(config.current_theme, "mocha");

        assert!(config.set_theme("invalid-theme".to_string()).is_err());
    }

    #[test]
    fn test_app_config_creation() {
        let config = AppConfig::new();
        assert!(config.validate().is_ok());
        assert!(config.is_feature_available(&Feature::UserAuthentication));
    }

    #[test]
    fn test_json_parsing() {
        let json = r#"{"current_theme": "mocha", "api_timeout_seconds": 60}"#;
        let config = RuntimeConfig::from_json(json).unwrap();
        assert_eq!(config.current_theme, "mocha");
        assert_eq!(config.api_timeout_seconds, 60);
    }
}