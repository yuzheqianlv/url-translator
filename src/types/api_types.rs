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
            deeplx_api_url: "https://deepl3.fileaiwork.online/dptrans?token=ej0ab47388ed86e843de9f499e52e6e664ae1m491cad7bf1.bIrYaAAAAAA=.b9c326068ac3c37ff36b8fea77867db51ddf235150945d7ad43472d68581e6c4pd14&newllm=1".to_string(),
            jina_api_url: "https://r.jina.ai".to_string(),
            default_source_lang: "auto".to_string(),
            default_target_lang: "ZH".to_string(),
            max_requests_per_second: 10, // 提高到每秒10个请求
            max_text_length: 5000, // 提高到5000字符
            max_paragraphs_per_request: 10, // 提高到10个段落
            file_naming: FileNamingConfig::default(),
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
