use crate::services::file_naming_service::FileNamingConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLXRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLXResponse {
    pub code: i32,
    pub data: String,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub deeplx_api_url: String,
    pub jina_api_url: String,
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub max_requests_per_second: u32,
    pub max_text_length: usize,
    pub max_paragraphs_per_request: usize,
    pub file_naming: FileNamingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            // 使用环境变量或默认的本地API地址，避免硬编码敏感信息
            deeplx_api_url: Self::get_env_or_default("DEEPLX_API_URL", "http://localhost:1188/translate"),
            jina_api_url: Self::get_env_or_default("JINA_API_URL", "https://r.jina.ai"),
            default_source_lang: Self::get_env_or_default("DEFAULT_SOURCE_LANG", "auto"),
            default_target_lang: Self::get_env_or_default("DEFAULT_TARGET_LANG", "ZH"),
            max_requests_per_second: Self::get_env_or_default("MAX_REQUESTS_PER_SECOND", "10").parse().unwrap_or(10),
            max_text_length: Self::get_env_or_default("MAX_TEXT_LENGTH", "5000").parse().unwrap_or(5000),
            max_paragraphs_per_request: Self::get_env_or_default("MAX_PARAGRAPHS_PER_REQUEST", "10").parse().unwrap_or(10),
            file_naming: FileNamingConfig::default(),
        }
    }
}

impl AppConfig {
    /// 从环境变量获取值，如果不存在则使用默认值
    /// 注意：在WASM环境中，环境变量需要在编译时设置
    fn get_env_or_default(key: &str, default: &str) -> String {
        // 在WASM环境中，我们无法在运行时读取环境变量
        // 可以考虑使用编译时环境变量或从localStorage读取配置
        #[cfg(target_arch = "wasm32")]
        {
            // WASM环境：优先从localStorage读取，然后使用默认值
            if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                if let Ok(Some(value)) = storage.get_item(&format!("config_{}", key.to_lowercase())) {
                    if !value.is_empty() {
                        return value;
                    }
                }
            }
            default.to_string()
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 非WASM环境：从环境变量读取
            std::env::var(key).unwrap_or_else(|_| default.to_string())
        }
    }
    
    /// 保存配置到localStorage（WASM环境）
    #[cfg(target_arch = "wasm32")]
    pub fn save_to_storage(&self) {
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            let _ = storage.set_item("config_deeplx_api_url", &self.deeplx_api_url);
            let _ = storage.set_item("config_jina_api_url", &self.jina_api_url);
            let _ = storage.set_item("config_default_source_lang", &self.default_source_lang);
            let _ = storage.set_item("config_default_target_lang", &self.default_target_lang);
            let _ = storage.set_item("config_max_requests_per_second", &self.max_requests_per_second.to_string());
            let _ = storage.set_item("config_max_text_length", &self.max_text_length.to_string());
            let _ = storage.set_item("config_max_paragraphs_per_request", &self.max_paragraphs_per_request.to_string());
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub url: String,
    pub source_lang: String,
    pub target_lang: String,
    pub config: AppConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResult {
    pub original_url: String,
    pub title: String,
    pub content: String,
    pub source_lang: String,
    pub target_lang: String,
    pub translated_at: String,
}
