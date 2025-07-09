use crate::error::{use_error_handler, AppError};
use crate::services::{
    config_service::ConfigService,
    preview_service::{PreviewContent, PreviewOptions, PreviewService},
};
use leptos::*;
use wasm_bindgen_futures::spawn_local;

pub struct UsePreviewReturn {
    pub is_loading: ReadSignal<bool>,
    pub preview_content: ReadSignal<Option<PreviewContent>>,
    pub generate_preview: WriteSignal<Option<(String, PreviewOptions)>>,
    pub clear_preview: WriteSignal<bool>,
}

pub fn use_preview() -> UsePreviewReturn {
    let error_handler = use_error_handler();

    let (is_loading, set_is_loading) = create_signal(false);
    let (preview_content, set_preview_content) = create_signal(None::<PreviewContent>);
    let (generate_trigger, set_generate_trigger) = create_signal(None::<(String, PreviewOptions)>);
    let (clear_trigger, set_clear_trigger) = create_signal(false);

    // 清除预览效果
    create_effect({
        let set_preview_content = set_preview_content.clone();
        move |_| {
            if clear_trigger.get() {
                set_preview_content.set(None);
            }
        }
    });

    // 预览生成效果
    create_effect({
        let set_is_loading = set_is_loading.clone();
        let set_preview_content = set_preview_content.clone();

        move |_| {
            if let Some((url, options)) = generate_trigger.get() {
                if url.is_empty() {
                    error_handler.handle_error(AppError::validation("URL", "请输入有效的URL"));
                    return;
                }

                set_is_loading.set(true);
                set_preview_content.set(None);

                let set_is_loading_clone = set_is_loading.clone();
                let set_preview_content_clone = set_preview_content.clone();

                spawn_local(async move {
                    web_sys::console::log_1(&"=== 开始生成预览 ===".into());
                    web_sys::console::log_1(&format!("URL: {}", url).into());

                    let config_service = ConfigService::new();

                    match config_service.get_config() {
                        Ok(config) => {
                            web_sys::console::log_1(&"配置加载成功".into());
                            let preview_service = PreviewService::new(&config);

                            match preview_service
                                .generate_preview(&url, &config, &options)
                                .await
                            {
                                Ok(content) => {
                                    web_sys::console::log_1(
                                        &format!(
                                            "预览生成成功 - 原文: {} 字符, 译文: {} 字符",
                                            content.character_count,
                                            content.translated_text.chars().count()
                                        )
                                        .into(),
                                    );

                                    set_preview_content_clone.set(Some(content));
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("预览生成失败: {}", e).into());
                                    error_handler.handle_error(AppError::translation(format!(
                                        "预览生成失败: {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("配置加载失败: {}", e).into());
                            error_handler
                                .handle_error(AppError::config(format!("配置加载失败: {}", e)));
                        }
                    }

                    set_is_loading_clone.set(false);
                });
            }
        }
    });

    UsePreviewReturn {
        is_loading,
        preview_content,
        generate_preview: set_generate_trigger,
        clear_preview: set_clear_trigger,
    }
}

/// 快速验证Hook
pub fn use_quick_validation() -> (
    ReadSignal<bool>,
    WriteSignal<Option<String>>,
    ReadSignal<Option<String>>,
) {
    let error_handler = use_error_handler();

    let (is_validating, set_is_validating) = create_signal(false);
    let (validation_trigger, set_validation_trigger) = create_signal(None::<String>);
    let (validation_result, set_validation_result) = create_signal(None::<String>);

    create_effect({
        let set_is_validating = set_is_validating.clone();
        let set_validation_result = set_validation_result.clone();

        move |_| {
            if let Some(url) = validation_trigger.get() {
                if url.is_empty() {
                    return;
                }

                set_is_validating.set(true);
                set_validation_result.set(None);

                let set_is_validating_clone = set_is_validating.clone();
                let set_validation_result_clone = set_validation_result.clone();

                spawn_local(async move {
                    web_sys::console::log_1(&"=== 快速验证开始 ===".into());

                    let config_service = ConfigService::new();

                    match config_service.get_config() {
                        Ok(config) => {
                            let preview_service = PreviewService::new(&config);

                            match preview_service.quick_validate(&url, &config).await {
                                Ok(result) => {
                                    web_sys::console::log_1(&"快速验证成功".into());
                                    set_validation_result_clone.set(Some(result));
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("快速验证失败: {}", e).into());
                                    error_handler.handle_error(AppError::validation("URL", e));
                                }
                            }
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("配置加载失败: {}", e).into());
                            error_handler.handle_error(AppError::config(format!("{}", e)));
                        }
                    }

                    set_is_validating_clone.set(false);
                });
            }
        }
    });

    (is_validating, set_validation_trigger, validation_result)
}
