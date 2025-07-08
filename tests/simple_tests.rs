use url_translator::types::api_types::*;
use url_translator::theme::*;
use url_translator::hooks::use_translation::TranslationStatus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.default_source_lang, "auto");
        assert_eq!(config.default_target_lang, "ZH");
        assert_eq!(config.max_requests_per_second, 10);
        assert_eq!(config.max_text_length, 5000);
        assert_eq!(config.max_paragraphs_per_request, 10);
        assert!(!config.deeplx_api_url.is_empty());
        assert!(!config.jina_api_url.is_empty());
    }

    #[test]
    fn test_theme_variant_display() {
        assert_eq!(format!("{}", ThemeVariant::Latte), "Latte");
        assert_eq!(format!("{}", ThemeVariant::Frappe), "Frappe");
        assert_eq!(format!("{}", ThemeVariant::Macchiato), "Macchiato");
        assert_eq!(format!("{}", ThemeVariant::Mocha), "Mocha");
    }

    #[test]
    fn test_translation_status() {
        let status = TranslationStatus::Idle;
        assert!(matches!(status, TranslationStatus::Idle));
        
        let status = TranslationStatus::ExtractingContent;
        assert!(matches!(status, TranslationStatus::ExtractingContent));
        
        let status = TranslationStatus::Translating;
        assert!(matches!(status, TranslationStatus::Translating));
        
        let status = TranslationStatus::Completed;
        assert!(matches!(status, TranslationStatus::Completed));
        
        let status = TranslationStatus::Failed("error".to_string());
        assert!(matches!(status, TranslationStatus::Failed(_)));
    }

    #[test]
    fn test_config_validation() {
        let config = AppConfig::default();
        
        // 测试默认值
        assert!(config.max_requests_per_second > 0);
        assert!(config.max_text_length > 0);
        assert!(config.max_paragraphs_per_request > 0);
        assert!(!config.deeplx_api_url.is_empty());
        assert!(!config.jina_api_url.is_empty());
    }
    
    #[test]
    fn test_language_codes() {
        let config = AppConfig::default();
        
        // 测试语言代码格式
        assert!(config.default_source_lang == "auto" || config.default_source_lang.len() == 2);
        assert!(config.default_target_lang.len() == 2);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        
        // 测试序列化
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(!serialized.is_empty());
        
        // 测试反序列化
        let deserialized: AppConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.default_source_lang, config.default_source_lang);
        assert_eq!(deserialized.default_target_lang, config.default_target_lang);
        assert_eq!(deserialized.max_requests_per_second, config.max_requests_per_second);
    }

    #[test]
    fn test_translation_result_structure() {
        let result = TranslationResult {
            original_url: "https://example.com".to_string(),
            title: "Test Title".to_string(),
            content: "Test Content".to_string(),
            source_lang: "EN".to_string(),
            target_lang: "ZH".to_string(),
            translated_at: "2023-01-01T00:00:00Z".to_string(),
        };
        
        assert_eq!(result.original_url, "https://example.com");
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.content, "Test Content");
        assert_eq!(result.source_lang, "EN");
        assert_eq!(result.target_lang, "ZH");
        assert!(!result.translated_at.is_empty());
    }

    #[test]
    fn test_error_types() {
        use url_translator::error::AppError;
        
        let error = AppError::network("网络错误");
        assert!(matches!(error, AppError::NetworkError { .. }));
        
        let error = AppError::validation("field", "验证错误");
        assert!(matches!(error, AppError::ValidationError { .. }));
        
        let error = AppError::config("配置错误");
        assert!(matches!(error, AppError::ConfigError { .. }));
    }

    #[test]
    fn test_url_validation() {
        // 测试URL验证辅助函数
        fn is_valid_url(url: &str) -> bool {
            url.starts_with("https://") || url.starts_with("http://")
        }
        
        assert!(is_valid_url("https://example.com"));
        assert!(is_valid_url("http://example.com"));
        assert!(is_valid_url("https://www.example.com/path"));
        assert!(is_valid_url("https://subdomain.example.com"));
        
        assert!(!is_valid_url("not-a-url"));
        assert!(!is_valid_url("ftp://example.com"));
        assert!(!is_valid_url(""));
        assert!(!is_valid_url("javascript:alert('test')"));
    }

    #[test]
    fn test_language_code_validation() {
        fn is_valid_language_code(code: &str) -> bool {
            matches!(code, "auto" | "ZH" | "EN" | "JA" | "FR" | "DE" | "ES")
        }
        
        // 测试有效语言代码
        assert!(is_valid_language_code("auto"));
        assert!(is_valid_language_code("ZH"));
        assert!(is_valid_language_code("EN"));
        assert!(is_valid_language_code("JA"));
        assert!(is_valid_language_code("FR"));
        assert!(is_valid_language_code("DE"));
        assert!(is_valid_language_code("ES"));
        
        // 测试无效语言代码
        assert!(!is_valid_language_code(""));
        assert!(!is_valid_language_code("INVALID"));
        assert!(!is_valid_language_code("zh"));
        assert!(!is_valid_language_code("en"));
    }

    #[test]
    fn test_translation_status_transitions() {
        // 测试状态转换逻辑
        let mut status = TranslationStatus::Idle;
        assert!(matches!(status, TranslationStatus::Idle));
        
        status = TranslationStatus::ExtractingContent;
        assert!(matches!(status, TranslationStatus::ExtractingContent));
        
        status = TranslationStatus::Translating;
        assert!(matches!(status, TranslationStatus::Translating));
        
        status = TranslationStatus::Completed;
        assert!(matches!(status, TranslationStatus::Completed));
        
        status = TranslationStatus::Failed("Test error".to_string());
        if let TranslationStatus::Failed(msg) = status {
            assert_eq!(msg, "Test error");
        } else {
            panic!("Expected Failed status");
        }
    }

    #[test]
    fn test_config_ranges() {
        let config = AppConfig::default();
        
        // 测试配置值的合理范围
        assert!(config.max_requests_per_second >= 1 && config.max_requests_per_second <= 100);
        assert!(config.max_text_length >= 1000);
        assert!(config.max_paragraphs_per_request >= 1);
    }

    #[test]
    fn test_theme_variants() {
        // 测试所有主题变体
        let variants = [
            ThemeVariant::Latte,
            ThemeVariant::Frappe,
            ThemeVariant::Macchiato,
            ThemeVariant::Mocha,
        ];
        
        for variant in variants.iter() {
            // 确保每个变体都能正确显示
            let display_str = format!("{}", variant);
            assert!(!display_str.is_empty());
            
            // 确保每个变体都有对应的主题
            let theme = variant.theme();
            assert!(!theme.base.is_empty());
            assert!(!theme.text.is_empty());
        }
    }
}