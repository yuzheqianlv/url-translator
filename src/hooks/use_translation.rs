use crate::error::{use_error_handler, AppError};
use crate::services::{
    config_service::ConfigService,
    deeplx_service::DeepLXService, history_service::HistoryService, jina_service::JinaService,
};
use crate::types::history::HistoryEntry;
use leptos::*;
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Debug)]
pub enum TranslationStatus {
    Idle,
    ExtractingContent,
    Translating,
    Completed,
    Failed(String),
}

pub struct UseTranslationReturn {
    pub is_loading: ReadSignal<bool>,
    pub translation_result: ReadSignal<String>,
    pub original_content: ReadSignal<String>,
    pub progress_message: ReadSignal<String>,
    pub status: ReadSignal<TranslationStatus>,
    pub translate: WriteSignal<Option<String>>,
}

pub fn use_translation() -> UseTranslationReturn {
    let error_handler = use_error_handler();

    let (is_loading, set_is_loading) = create_signal(false);
    let (translation_result, set_translation_result) = create_signal(String::new());
    let (original_content, set_original_content) = create_signal(String::new());
    let (progress_message, set_progress_message) = create_signal(String::new());
    let (status, set_status) = create_signal(TranslationStatus::Idle);
    let (translate_trigger, set_translate_trigger) = create_signal(None::<String>);

    // Effect to handle translation when trigger changes
    create_effect({
        let set_is_loading = set_is_loading;
        let set_translation_result = set_translation_result;
        let set_original_content = set_original_content;
        let set_progress_message = set_progress_message;
        let set_status = set_status;

        move |_| {
            if let Some(url) = translate_trigger.get() {
                if url.is_empty() {
                    error_handler.handle_error(AppError::validation("URL", "请输入有效的URL"));
                    return;
                }

                set_is_loading.set(true);
                set_translation_result.set(String::new());
                set_original_content.set(String::new());
                set_status.set(TranslationStatus::ExtractingContent);
                set_progress_message.set("正在提取网页内容...".to_string());

                let set_progress_clone = set_progress_message;
                let set_loading_clone = set_is_loading;
                let set_result_clone = set_translation_result;
                let set_original_clone = set_original_content;
                let set_status_clone = set_status;

                spawn_local(async move {
                    web_sys::console::log_1(&"=== 开始翻译流程 ===".into());
                    web_sys::console::log_1(&format!("URL: {url}").into());

                    let config_service = ConfigService::new();
                    web_sys::console::log_1(&"配置服务已创建".into());

                    match config_service.get_config() {
                        Ok(config) => {
                            web_sys::console::log_1(&"配置加载成功，创建服务...".into());
                            let jina_service = JinaService::new(&config);
                            let deeplx_service = DeepLXService::new(&config);

                            // 步骤1: 提取内容
                            web_sys::console::log_1(&"=== 步骤1: 开始提取网页内容 ===".into());
                            set_progress_clone.set("正在提取网页内容...".to_string());

                            match jina_service.extract_content(&url, &config).await {
                                Ok(content) => {
                                    web_sys::console::log_1(
                                        &format!("内容提取成功，长度: {} 字符", content.len())
                                            .into(),
                                    );
                                    
                                    // 存储原文内容
                                    set_original_clone.set(content.clone());

                                    // 步骤2: 直接翻译内容（取消保护机制）
                                    web_sys::console::log_1(&"=== 步骤2: 开始翻译内容 ===".into());
                                    set_status_clone.set(TranslationStatus::Translating);
                                    set_progress_clone.set("正在翻译内容...".to_string());

                                    web_sys::console::log_1(&format!("原始内容长度: {} 字符", content.len()).into());

                                    match deeplx_service
                                        .translate(
                                            &content,
                                            &config.default_source_lang,
                                            &config.default_target_lang,
                                            &config,
                                        )
                                        .await
                                    {
                                        Ok(final_translated_content) => {
                                            web_sys::console::log_1(
                                                &format!(
                                                    "翻译成功，长度: {} 字符",
                                                    final_translated_content.len()
                                                )
                                                .into(),
                                            );

                                            set_result_clone.set(final_translated_content.clone());
                                            set_status_clone.set(TranslationStatus::Completed);
                                            set_progress_clone.set("翻译完成".to_string());

                                            // 保存到历史记录
                                            let history_service = HistoryService::new();
                                            let title = extract_title_from_content(&content);
                                            let history_entry = HistoryEntry::new(
                                                url.clone(),
                                                title,
                                                config.default_source_lang.clone(),
                                                config.default_target_lang.clone(),
                                                content,
                                                final_translated_content,
                                            );

                                            if let Err(e) = history_service.add_entry(history_entry)
                                            {
                                                web_sys::console::log_1(
                                                    &format!("保存历史记录失败: {e}").into(),
                                                );
                                            } else {
                                                web_sys::console::log_1(&"历史记录保存成功".into());
                                            }

                                            web_sys::console::log_1(&"=== 翻译流程完成 ===".into());
                                        }
                                        Err(e) => {
                                            web_sys::console::log_1(
                                                &format!("翻译失败: {e}").into(),
                                            );
                                            let error_msg =
                                                format!("翻译失败: {e}。请检查DeepLX API配置。");
                                            set_status_clone
                                                .set(TranslationStatus::Failed(error_msg.clone()));
                                            error_handler
                                                .handle_error(AppError::translation(error_msg));
                                        }
                                    }
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("内容提取失败: {e}").into());
                                    let error_msg = format!(
                                        "内容提取失败: {e}。请检查URL是否有效，或Jina API是否可用。"
                                    );
                                    set_status_clone
                                        .set(TranslationStatus::Failed(error_msg.clone()));
                                    error_handler.handle_error(AppError::extraction(error_msg));
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("配置加载失败: {e}").into());
                            let error_msg = format!("配置加载失败: {e}");
                            set_status_clone.set(TranslationStatus::Failed(error_msg.clone()));
                            error_handler.handle_error(AppError::config(error_msg));
                        }
                    }

                    set_loading_clone.set(false);
                    set_progress_clone.set(String::new());
                });
            }
        }
    });

    UseTranslationReturn {
        is_loading,
        translation_result,
        original_content,
        progress_message,
        status,
        translate: set_translate_trigger,
    }
}

fn extract_title_from_content(content: &str) -> String {
    // 尝试从内容中提取标题
    let lines: Vec<&str> = content.lines().collect();

    for line in lines.iter().take(5) {
        let trimmed = line.trim();
        if !trimmed.is_empty() && trimmed.len() > 10 && trimmed.len() < 200 {
            // 移除 markdown 标题标记
            let title = trimmed.trim_start_matches('#').trim();
            if !title.is_empty() {
                return title.to_string();
            }
        }
    }

    // 如果找不到合适的标题，使用内容的前50个字符
    let content_preview = content
        .chars()
        .take(50)
        .collect::<String>()
        .trim()
        .to_string();

    if content_preview.is_empty() {
        "无标题".to_string()
    } else {
        format!("{content_preview}...")
    }
}
