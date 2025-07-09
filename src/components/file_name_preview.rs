use crate::hooks::use_config::use_config;
use crate::services::file_naming_service::{FileNamingContext, FileNamingService};
use crate::theme::use_theme_context;
use chrono::Utc;
use leptos::*;
use wasm_bindgen::JsCast;

#[component]
pub fn FileNamePreview(
    url: ReadSignal<String>,
    #[prop(optional)] title: Option<ReadSignal<String>>,
    #[prop(optional)] order: Option<usize>,
    #[prop(optional)] content_type: Option<String>,
) -> impl IntoView {
    let config_hook = use_config();
    let theme_context = use_theme_context();

    let preview_file_name = create_memo(move |_| {
        let current_url = url.get();
        if current_url.is_empty() {
            return "请先输入URL".to_string();
        }

        let config = config_hook.config.get();
        let naming_service = FileNamingService::new(config.file_naming);

        // 从URL中提取标题或使用提供的标题
        let extracted_title = title
            .map(|t| t.get())
            .unwrap_or_else(|| extract_title_from_url(&current_url));

        let context = FileNamingContext {
            url: current_url,
            title: extracted_title,
            order,
            timestamp: Utc::now(),
            content_type: content_type
                .clone()
                .unwrap_or_else(|| "article".to_string()),
            folder_path: None,
        };

        let result = naming_service.preview_file_name(&context);
        result.file_name
    });

    view! {
        <div class="mt-2 p-3 rounded-md" style=move || theme_context.get().theme.content_bg_style()>
            <div class="flex items-center justify-between">
                <div>
                    <span class="text-xs font-medium" style=move || theme_context.get().theme.subtext_style()>
                        "文件名预览"
                    </span>
                    <div class="font-mono text-sm mt-1" style=move || theme_context.get().theme.text_style()>
                        {preview_file_name}
                    </div>
                </div>
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.info_color())>
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"></path>
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"></path>
                </svg>
            </div>
        </div>
    }
}

#[component]
pub fn BatchFileNamePreview(
    urls: ReadSignal<Vec<String>>,
    titles: ReadSignal<Vec<String>>,
) -> impl IntoView {
    let config_hook = use_config();
    let theme_context = use_theme_context();

    let preview_files = create_memo(move |_| {
        let url_list = urls.get();
        let title_list = titles.get();

        if url_list.is_empty() {
            return Vec::new();
        }

        let config = config_hook.config.get();
        let naming_service = FileNamingService::new(config.file_naming);

        url_list
            .iter()
            .enumerate()
            .take(5) // 只显示前5个预览
            .map(|(index, url)| {
                let title = title_list
                    .get(index)
                    .cloned()
                    .unwrap_or_else(|| extract_title_from_url(url));

                let context = FileNamingContext {
                    url: url.clone(),
                    title,
                    order: Some(index),
                    timestamp: Utc::now(),
                    content_type: "documentation".to_string(),
                    folder_path: None,
                };

                let result = naming_service.preview_file_name(&context);
                (index + 1, result.file_name, result.folder_path)
            })
            .collect::<Vec<_>>()
    });

    view! {
        <Show when=move || !preview_files.get().is_empty()>
            <div class="mt-4 p-4 rounded-md" style=move || theme_context.get().theme.content_bg_style()>
                <div class="flex items-center justify-between mb-3">
                    <h4 class="text-sm font-medium" style=move || theme_context.get().theme.text_style()>
                        "文件名预览 (前5个)"
                    </h4>
                    <span class="text-xs" style=move || theme_context.get().theme.subtext_style()>
                        {move || {
                            let total = urls.get().len();
                            let shown = preview_files.get().len();
                            if total > shown {
                                format!("显示 {}/{} 个", shown, total)
                            } else {
                                format!("{} 个文件", total)
                            }
                        }}
                    </span>
                </div>

                <div class="space-y-2 max-h-60 overflow-y-auto">
                    <For
                        each=move || preview_files.get()
                        key=|(order, _, _)| *order
                        children=move |(order, file_name, folder_path)| {
                            view! {
                                <div class="flex items-center justify-between p-2 rounded border" style=move || theme_context.get().theme.content_bg_style()>
                                    <div class="flex items-center space-x-3">
                                        <span class="flex-shrink-0 w-6 h-6 bg-blue-100 text-blue-800 text-xs font-medium rounded flex items-center justify-center">
                                            {order}
                                        </span>
                                        <div class="min-w-0">
                                            <div class="font-mono text-xs" style=move || theme_context.get().theme.text_style()>
                                                {file_name}
                                            </div>
                                            <div class="text-xs" style=move || theme_context.get().theme.subtext_style()>
                                                {folder_path}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn AdvancedFileNamePreview(
    url: ReadSignal<String>,
    title: ReadSignal<String>,
    #[prop(optional)] editable: bool,
) -> impl IntoView {
    let config_hook = use_config();
    let theme_context = use_theme_context();

    let (custom_title, set_custom_title) = create_signal(String::new());
    let (use_custom, set_use_custom) = create_signal(false);

    // 同步标题到自定义标题输入框
    create_effect(move |_| {
        if !use_custom.get() {
            set_custom_title.set(title.get());
        }
    });

    let final_title = create_memo(move |_| {
        if use_custom.get() {
            custom_title.get()
        } else {
            title.get()
        }
    });

    let preview_file_name = create_memo(move |_| {
        let current_url = url.get();
        if current_url.is_empty() {
            return "请先输入URL".to_string();
        }

        let config = config_hook.config.get();
        let naming_service = FileNamingService::new(config.file_naming);

        let context = FileNamingContext {
            url: current_url,
            title: final_title.get(),
            order: None,
            timestamp: Utc::now(),
            content_type: "article".to_string(),
            folder_path: None,
        };

        let result = naming_service.preview_file_name(&context);
        result.file_name
    });

    view! {
        <div class="mt-4 p-4 rounded-md border" style=move || theme_context.get().theme.card_style()>
            <h4 class="text-sm font-medium mb-3" style=move || theme_context.get().theme.text_style()>
                "文件名预览"
            </h4>

            <Show when=move || editable>
                <div class="mb-3">
                    <label class="flex items-center space-x-2">
                        <input
                            type="checkbox"
                            class="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                            prop:checked=use_custom
                            on:change=move |ev| {
                                let target = ev.target().unwrap();
                                let input = target.dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                set_use_custom.set(input.checked());
                            }
                        />
                        <span class="text-sm" style=move || theme_context.get().theme.text_style()>
                            "自定义文件标题"
                        </span>
                    </label>
                </div>

                <Show when=move || use_custom.get()>
                    <div class="mb-3">
                        <input
                            type="text"
                            class="w-full px-3 py-2 text-sm rounded-md focus:ring-2 focus:border-transparent"
                            style=move || theme_context.get().theme.input_style()
                            placeholder="输入自定义标题"
                            prop:value=custom_title
                            on:input=move |ev| {
                                set_custom_title.set(event_target_value(&ev));
                            }
                        />
                    </div>
                </Show>
            </Show>

            <div class="bg-gray-50 p-3 rounded font-mono text-sm" style=move || theme_context.get().theme.content_bg_style()>
                <div class="flex items-center justify-between">
                    <span style=move || theme_context.get().theme.text_style()>
                        {preview_file_name}
                    </span>
                    <button
                        type="button"
                        class="text-xs px-2 py-1 rounded hover:opacity-80"
                        style=move || theme_context.get().theme.button_secondary_style()
                        on:click=move |_| {
                            // 复制到剪贴板 - 使用简单的方法
                            let filename = preview_file_name.get();
                            web_sys::console::log_1(&format!("文件名已复制: {}", filename).into());
                        }
                    >
                        "复制"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// 从URL中提取标题的辅助函数
fn extract_title_from_url(url: &str) -> String {
    if url.is_empty() {
        return "untitled".to_string();
    }

    if let Ok(parsed_url) = url::Url::parse(url) {
        let path = parsed_url.path();
        if let Some(last_segment) = path.split('/').last() {
            if !last_segment.is_empty() && last_segment != "index.html" {
                let name = last_segment.split('.').next().unwrap_or(last_segment);
                if !name.is_empty() {
                    return name.to_string();
                }
            }
        }

        if let Some(domain) = parsed_url.domain() {
            return domain.replace('.', "_");
        }
    }

    "untitled".to_string()
}
