use crate::hooks::use_preview::use_preview;
use crate::services::preview_service::{PreviewContent, PreviewOptions};
use leptos::*;

#[component]
pub fn PreviewPanel(
    #[prop(into)] url: Signal<String>,
    #[prop(into, optional)] show_preview: Signal<bool>,
) -> impl IntoView {
    let preview = use_preview();

    let (preview_options, set_preview_options) = create_signal(PreviewOptions::default());

    let handle_generate_preview = move |_| {
        let current_url = url.get();
        if !current_url.is_empty() {
            preview
                .generate_preview
                .set(Some((current_url, preview_options.get())));
        }
    };

    let handle_clear_preview = move |_| {
        preview.clear_preview.set(true);
    };

    view! {
        <Show when=move || show_preview.get()>
            <div class="mt-6 p-4 bg-blue-50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-lg">
                <div class="flex items-center justify-between mb-4">
                    <h3 class="text-lg font-semibold text-blue-800 dark:text-blue-200">
                        "翻译预览"
                    </h3>
                    <div class="flex gap-2">
                        <button
                            type="button"
                            class="px-3 py-1 text-sm bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white rounded transition-colors"
                            on:click=handle_generate_preview
                            prop:disabled=move || preview.is_loading.get() || url.get().is_empty()
                        >
                            {move || if preview.is_loading.get() { "生成中..." } else { "生成预览" }}
                        </button>
                        <button
                            type="button"
                            class="px-3 py-1 text-sm bg-gray-500 hover:bg-gray-600 text-white rounded transition-colors"
                            on:click=handle_clear_preview
                            prop:disabled=move || preview.preview_content.get().is_none()
                        >
                            "清除"
                        </button>
                    </div>
                </div>

                // 预览选项
                <PreviewOptionsPanel
                    options=preview_options
                    set_options=set_preview_options
                />

                // 加载状态
                <Show when=move || preview.is_loading.get()>
                    <div class="my-4 p-3 bg-blue-100 dark:bg-blue-800 rounded text-blue-800 dark:text-blue-200 text-sm">
                        <div class="flex items-center gap-2">
                            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
                            "正在生成预览，请稍候..."
                        </div>
                    </div>
                </Show>

                // 预览内容
                <Show when=move || preview.preview_content.get().is_some()>
                    <PreviewContentDisplay content=preview.preview_content />
                </Show>
            </div>
        </Show>
    }
}

#[component]
fn PreviewOptionsPanel(
    options: ReadSignal<PreviewOptions>,
    set_options: WriteSignal<PreviewOptions>,
) -> impl IntoView {
    view! {
        <div class="mb-4 p-3 bg-white dark:bg-gray-800 rounded border">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                "预览选项"
            </h4>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-3 text-sm">
                <div>
                    <label class="block text-gray-600 dark:text-gray-400 mb-1">
                        "最大段落数"
                    </label>
                    <input
                        type="number"
                        min="1"
                        max="10"
                        class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded text-gray-900 dark:text-white dark:bg-gray-700"
                        prop:value=move || options.get().max_paragraphs.to_string()
                        on:input=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<usize>() {
                                let mut opts = options.get();
                                opts.max_paragraphs = value.clamp(1, 10);
                                set_options.set(opts);
                            }
                        }
                    />
                </div>
                <div>
                    <label class="block text-gray-600 dark:text-gray-400 mb-1">
                        "最大字符数"
                    </label>
                    <input
                        type="number"
                        min="200"
                        max="2000"
                        step="100"
                        class="w-full px-2 py-1 border border-gray-300 dark:border-gray-600 rounded text-gray-900 dark:text-white dark:bg-gray-700"
                        prop:value=move || options.get().max_characters.to_string()
                        on:input=move |ev| {
                            if let Ok(value) = event_target_value(&ev).parse::<usize>() {
                                let mut opts = options.get();
                                opts.max_characters = value.clamp(200, 2000);
                                set_options.set(opts);
                            }
                        }
                    />
                </div>
                <div class="flex items-center">
                    <label class="flex items-center gap-2 text-gray-600 dark:text-gray-400">
                        <input
                            type="checkbox"
                            class="rounded"
                            prop:checked=move || options.get().include_title
                            on:change=move |ev| {
                                let mut opts = options.get();
                                opts.include_title = event_target_checked(&ev);
                                set_options.set(opts);
                            }
                        />
                        "包含标题"
                    </label>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PreviewContentDisplay(content: ReadSignal<Option<PreviewContent>>) -> impl IntoView {
    view! {
        <Show when=move || content.get().is_some()>
            {move || {
                content.get().map(|preview| {
                    view! {
                        <div class="space-y-4">
                            // 统计信息
                            <div class="flex flex-wrap gap-4 text-sm text-blue-700 dark:text-blue-300 bg-white dark:bg-gray-800 p-3 rounded">
                                <span>"字符数: " {preview.character_count}</span>
                                <span>"词数: " {preview.word_count}</span>
                                <span>"预览长度: " {preview.preview_length}</span>
                            </div>

                            // 对比显示
                            <div class="grid md:grid-cols-2 gap-4">
                                // 原文
                                <div class="bg-white dark:bg-gray-800 p-4 rounded border">
                                    <h4 class="font-semibold text-gray-800 dark:text-gray-200 mb-2 flex items-center gap-2">
                                        <svg class="w-4 h-4 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
                                            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                        </svg>
                                        "原文预览"
                                    </h4>
                                    <div class="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap max-h-60 overflow-y-auto">
                                        {preview.original_text.clone()}
                                    </div>
                                </div>

                                // 译文
                                <div class="bg-white dark:bg-gray-800 p-4 rounded border">
                                    <h4 class="font-semibold text-gray-800 dark:text-gray-200 mb-2 flex items-center gap-2">
                                        <svg class="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                            <path d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                                        </svg>
                                        "译文预览"
                                    </h4>
                                    <div class="text-sm text-gray-700 dark:text-gray-300 whitespace-pre-wrap max-h-60 overflow-y-auto">
                                        {preview.translated_text.clone()}
                                    </div>
                                </div>
                            </div>

                            // 提示信息
                            <div class="text-xs text-blue-600 dark:text-blue-400 bg-blue-100 dark:bg-blue-900/30 p-2 rounded">
                                "💡 这只是前几段的预览。完整翻译会处理整个页面内容。"
                            </div>
                        </div>
                    }
                })
            }}
        </Show>
    }
}
