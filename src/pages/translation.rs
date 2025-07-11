//! Translation page - dedicated page for single URL and batch translation

use crate::components::{
    AsyncTranslationDisplay, AuthRequired, BilingualDisplay, DisplayMode, FileNamePreview, PreviewPanel, ProgressIndicator, UrlInput,
};
use crate::hooks::use_config::use_config;
use crate::hooks::use_backend_translation::use_backend_translation;
use crate::hooks::use_async_translation::use_async_translation;
use crate::hooks::use_auth::{use_auth, AuthStatus};
use crate::services::file_naming_service::{FileNamingContext, FileNamingService};
use crate::services::api_client::TranslateUrlRequest;
use crate::theme::use_theme_context;
use chrono::Utc;
use leptos::*;
use leptos_router::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, Blob, Url};

#[component]
pub fn TranslationPage() -> impl IntoView {
    let translation = use_backend_translation();
    let async_translation = use_async_translation();
    let auth = use_auth();
    let theme_context = use_theme_context();
    let config_hook = use_config();
    let (url, set_url) = create_signal(String::new());
    let (show_preview, set_show_preview) = create_signal(false);
    let (display_mode, set_display_mode) = create_signal(DisplayMode::Bilingual);
    let (use_async_mode, set_use_async_mode) = create_signal(true); // 默认使用异步模式
    
    // 从翻译结果派生的内容信号
    let (original_content, set_original_content) = create_signal(String::new());
    let (translated_content, set_translated_content) = create_signal(String::new());
    
    // 创建loading状态的ReadSignal
    let (is_loading, set_is_loading) = create_signal(false);
    
    // 监听翻译结果变化并更新内容
    create_effect(move |_| {
        if let Some(response) = translation.translation_result.get() {
            set_original_content.set(response.original_content);
            set_translated_content.set(response.translated_content);
        }
    });
    
    // 监听loading状态变化
    create_effect(move |_| {
        set_is_loading.set(translation.is_loading.get());
    });

    let handle_translate = move |_| {
        let url_value = url.get();
        if url_value.is_empty() {
            return;
        }
        
        // 检查用户认证状态
        match auth.auth_status.get() {
            AuthStatus::Authenticated(_) => {
                let config = config_hook.config.get();
                let request = TranslateUrlRequest {
                    url: url_value,
                    source_lang: config.default_source_lang,
                    target_lang: config.default_target_lang,
                    project_id: None, // 主页单页翻译不关联项目
                };
                
                if use_async_mode.get() {
                    // 使用异步模式
                    async_translation.submit_task.set(Some(request));
                } else {
                    // 使用同步模式（原有的后端翻译）
                    translation.translate.set(Some(request));
                }
            }
            _ => {
                // 用户未登录，显示登录提示或跳转到登录页面
                web_sys::console::log_1(&"需要登录后才能使用翻译功能".into());
            }
        }
    };

    let download_markdown = move |_| {
        // 从后端翻译结果获取内容
        let translation_response = translation.translation_result.get();
        let Some(response) = translation_response else {
            return;
        };
        
        // 根据显示模式决定下载内容
        let content = match display_mode.get() {
            DisplayMode::TranslationOnly => response.translated_content.clone(),
            DisplayMode::OriginalOnly => response.original_content.clone(),
            DisplayMode::Bilingual => {
                // 创建双语对照内容
                create_bilingual_markdown(&response.original_content, &response.translated_content)
            }
        };
        
        if content.is_empty() {
            return;
        }

        // 使用智能文件命名服务生成文件名
        let current_url = url.get();
        let config = config_hook.config.get();
        let mut naming_service = FileNamingService::new(config.file_naming);

        // 从翻译结果中提取标题（如果有的话）
        let title = extract_title_from_content(&content).unwrap_or_else(|| {
            // 如果无法从内容中提取标题，尝试从URL中提取
            extract_title_from_url(&current_url)
        });

        let context = FileNamingContext {
            url: current_url,
            title,
            order: None,
            timestamp: Utc::now(),
            content_type: "article".to_string(),
            folder_path: None,
        };

        let naming_result = naming_service.generate_file_name(&context);
        let _ = create_and_download_file(&content, &naming_result.file_name, "text/markdown");
    };

    let navigate_to_batch = move |_| {
        let navigate = use_navigate();
        navigate("/batch", Default::default());
    };

    view! {
        <div class="max-w-4xl mx-auto space-y-6">
            <div class="text-center">
                <h1 class="text-3xl font-bold mb-4" style=move || theme_context.get().theme.text_style()>
                    "🌐 翻译工具"
                </h1>
                <p class="mb-6" style=move || theme_context.get().theme.subtext_style()>
                    "智能翻译网页内容，支持代码块保护，提供单页和批量翻译模式"
                </p>
            </div>

            // 翻译模式选择卡片
            <div class="grid md:grid-cols-2 gap-6 mb-8">
                // 单页翻译卡片
                <div class="rounded-lg shadow-lg p-6 transition-all hover:shadow-xl" style=move || theme_context.get().theme.card_style()>
                    <div class="flex items-center mb-4">
                        <div class="p-2 rounded-lg mr-3" style=move || theme_context.get().theme.content_bg_style()>
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.info_color())>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold" style=move || theme_context.get().theme.text_style()>"单页翻译"</h3>
                            <p class="text-sm" style=move || theme_context.get().theme.subtext_style()>"翻译单个网页内容"</p>
                        </div>
                    </div>
                    <p class="text-sm mb-4" style=move || theme_context.get().theme.subtext_style()>
                        "输入网页URL，快速翻译单个页面的内容。支持自动提取正文、保护代码块、生成Markdown文件。"
                    </p>
                    <div class="text-sm font-medium" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                        "✓ 当前模式"
                    </div>
                </div>

                // 批量翻译卡片
                <div class="rounded-lg shadow-lg p-6 transition-all hover:shadow-xl cursor-pointer"
                     style=move || theme_context.get().theme.card_style()
                     on:click=navigate_to_batch>
                    <div class="flex items-center mb-4">
                        <div class="p-2 rounded-lg mr-3" style=move || theme_context.get().theme.content_bg_style()>
                            <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"></path>
                            </svg>
                        </div>
                        <div>
                            <h3 class="text-lg font-semibold" style=move || theme_context.get().theme.text_style()>"批量翻译"</h3>
                            <p class="text-sm" style=move || theme_context.get().theme.subtext_style()>"翻译整个文档网站"</p>
                        </div>
                    </div>
                    <p class="text-sm mb-4" style=move || theme_context.get().theme.subtext_style()>
                        "输入文档网站首页，自动解析目录结构，批量翻译所有页面并打包下载。适合翻译完整的技术文档。"
                    </p>
                    <div class="text-sm font-medium" style=move || format!("color: {};", theme_context.get().theme.success_color())>
                        "→ 点击切换到批量翻译"
                    </div>
                </div>
            </div>

            <AuthRequired message="请先登录后使用翻译功能，翻译结果将保存到您的账户中".to_string()>
                <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
                    <div class="space-y-4">
                        <UrlInput
                            url=url
                            set_url=set_url
                            on_submit=handle_translate
                            is_loading=is_loading
                        />

                        // 预览切换按钮和模式切换
                        <div class="flex items-center gap-4 flex-wrap">
                            <button
                            type="button"
                            class="inline-flex items-center px-3 py-2 text-sm font-medium rounded-md border transition-colors"
                            class:bg-blue-100=move || show_preview.get()
                            class:border-blue-300=move || show_preview.get()
                            class:text-blue-800=move || show_preview.get()
                            class:dark:bg-blue-900=move || show_preview.get()
                            class:dark:border-blue-700=move || show_preview.get()
                            class:dark:text-blue-200=move || show_preview.get()
                            class:bg-gray-100=move || !show_preview.get()
                            class:border-gray-300=move || !show_preview.get()
                            class:text-gray-700=move || !show_preview.get()
                            class:dark:bg-gray-700=move || !show_preview.get()
                            class:dark:border-gray-600=move || !show_preview.get()
                            class:dark:text-gray-300=move || !show_preview.get()
                            on:click=move |_| set_show_preview.update(|show| *show = !*show)
                        >
                            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                            </svg>
                            {move || if show_preview.get() { "隐藏预览" } else { "显示预览" }}
                        </button>

                        <button
                            type="button"
                            class="inline-flex items-center px-3 py-2 text-sm font-medium rounded-md border transition-colors"
                            class:bg-green-100=move || use_async_mode.get()
                            class:border-green-300=move || use_async_mode.get()
                            class:text-green-800=move || use_async_mode.get()
                            class:dark:bg-green-900=move || use_async_mode.get()
                            class:dark:border-green-700=move || use_async_mode.get()
                            class:dark:text-green-200=move || use_async_mode.get()
                            class:bg-gray-100=move || !use_async_mode.get()
                            class:border-gray-300=move || !use_async_mode.get()
                            class:text-gray-700=move || !use_async_mode.get()
                            class:dark:bg-gray-700=move || !use_async_mode.get()
                            class:dark:border-gray-600=move || !use_async_mode.get()
                            class:dark:text-gray-300=move || !use_async_mode.get()
                            on:click=move |_| set_use_async_mode.update(|async_mode| *async_mode = !*async_mode)
                        >
                            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"></path>
                            </svg>
                            {move || if use_async_mode.get() { "异步模式" } else { "同步模式" }}
                        </button>

                        <div class="text-xs" style=move || theme_context.get().theme.subtext_style()>
                            <span>
                                "预览功能可以在正式翻译前查看前几段的翻译效果"
                            </span>
                            <br/>
                            <span>
                                {move || if use_async_mode.get() { "异步模式：翻译任务在后台处理，不会因页面刷新而中断" } else { "同步模式：直接返回翻译结果，页面刷新会中断翻译" }}
                            </span>
                        </div>
                    </div>

                    <ProgressIndicator
                        is_loading=translation.is_loading
                        progress_message=translation.progress_message
                        status=translation.status
                    />

                    // 文件名预览
                    <Show when=move || !url.get().is_empty()>
                        <FileNamePreview
                            url=url
                        />
                    </Show>
                    </div>
                </div>

                // 预览面板
                    <PreviewPanel
                        url=url
                        show_preview=show_preview
                    />

                    // 异步翻译显示组件
                    <AsyncTranslationDisplay />

                    <BilingualDisplay
                    original_content=original_content
                    translated_content=translated_content
                    display_mode=display_mode
                    on_download=download_markdown
                    on_mode_change=move |mode| set_display_mode.set(mode)
                />
            </AuthRequired>
        </div>
    }
}

fn create_and_download_file(
    content: &str,
    filename: &str,
    _mime_type: &str,
) -> Result<(), JsValue> {
    let window = window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&JsValue::from_str(content));

    let blob = Blob::new_with_str_sequence(&blob_parts)?;
    let url = Url::create_object_url_with_blob(&blob)?;

    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", &url)?;
    anchor.set_attribute("download", filename)?;
    anchor.set_attribute("style", "display: none")?;

    document.body().unwrap().append_child(&anchor)?;

    let html_anchor = anchor.dyn_ref::<web_sys::HtmlAnchorElement>().unwrap();
    html_anchor.click();

    document.body().unwrap().remove_child(&anchor)?;
    Url::revoke_object_url(&url)?;

    Ok(())
}

/// 从Markdown内容中提取标题
fn extract_title_from_content(content: &str) -> Option<String> {
    // 寻找第一个一级标题
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("# ") && line.len() > 2 {
            return Some(line[2..].trim().to_string());
        }
    }

    // 寻找第一个二级标题
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("## ") && line.len() > 3 {
            return Some(line[3..].trim().to_string());
        }
    }

    None
}

/// 从URL中提取标题
fn extract_title_from_url(url: &str) -> String {
    if url.is_empty() {
        return "translated_content".to_string();
    }

    // 尝试解析URL
    if let Ok(parsed_url) = url::Url::parse(url) {
        // 获取路径的最后一部分
        let path = parsed_url.path();
        if let Some(last_segment) = path.split('/').next_back() {
            if !last_segment.is_empty() && last_segment != "index.html" {
                // 移除文件扩展名
                let name = last_segment.split('.').next().unwrap_or(last_segment);
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }

        // 如果路径不能提供有用信息，使用域名
        if let Some(domain) = parsed_url.domain() {
            return domain.replace('.', "_");
        }
    }

    "translated_content".to_string()
}

/// 创建双语对照的Markdown内容
fn create_bilingual_markdown(original: &str, translated: &str) -> String {
    if original.is_empty() || translated.is_empty() {
        return if !translated.is_empty() {
            translated.to_string()
        } else {
            original.to_string()
        };
    }

    let original_lines: Vec<&str> = original.lines().collect();
    let translated_lines: Vec<&str> = translated.lines().collect();
    
    let mut result = Vec::new();
    let max_len = original_lines.len().max(translated_lines.len());
    
    // 添加双语模式说明
    result.push("<!-- 双语对照模式：译文在前，原文在后 -->".to_string());
    result.push("".to_string());
    
    for i in 0..max_len {
        let original_line = original_lines.get(i).unwrap_or(&"").trim();
        let translated_line = translated_lines.get(i).unwrap_or(&"").trim();
        
        // 如果是空行，只添加一个空行
        if original_line.is_empty() && translated_line.is_empty() {
            result.push("".to_string());
            continue;
        }
        
        // 如果只有一个是空行，添加非空的那一行
        if original_line.is_empty() && !translated_line.is_empty() {
            result.push(translated_line.to_string());
            continue;
        }
        
        if translated_line.is_empty() && !original_line.is_empty() {
            result.push(original_line.to_string());
            continue;
        }
        
        // 两行都不为空，按译文、原文的顺序添加
        if !translated_line.is_empty() {
            result.push(translated_line.to_string());
        }
        
        if !original_line.is_empty() {
            // 为原文添加引用格式以便区分
            result.push(format!("> {}", original_line));
        }
        
        // 段落之间添加空行
        result.push("".to_string());
    }
    
    result.join("\n")
}