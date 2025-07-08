use leptos::*;
use crate::hooks::use_batch_translation::use_batch_translation;
use crate::services::batch_service::BatchStatus;

#[component]
pub fn BatchTranslation() -> impl IntoView {
    let batch_translation = use_batch_translation();
    
    let (index_url, set_index_url) = create_signal(String::new());

    let handle_submit = move |_| {
        let url = index_url.get_untracked();
        if !url.is_empty() {
            batch_translation.start_batch_translation.set(Some(url));
        }
    };

    view! {
        <div class="max-w-4xl mx-auto p-6 bg-white dark:bg-gray-800 rounded-lg shadow-lg">
            <div class="mb-6">
                <h2 class="text-2xl font-bold text-gray-900 dark:text-white mb-2">
                    "批量翻译文档网站"
                </h2>
                <p class="text-gray-600 dark:text-gray-400">
                    "输入文档网站的首页URL，系统将自动解析目录结构并批量翻译所有页面"
                </p>
            </div>

            // 输入区域
            <div class="mb-6">
                <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    "文档网站首页URL"
                </label>
                <div class="flex gap-3">
                    <input
                        type="url"
                        class="flex-1 px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-transparent dark:bg-gray-700 dark:text-white"
                        placeholder="https://example.com/docs/"
                        prop:value=index_url
                        on:input=move |ev| {
                            set_index_url.set(event_target_value(&ev));
                        }
                        prop:disabled=move || batch_translation.is_processing.get()
                    />
                    <button
                        type="button"
                        class="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white font-medium rounded-lg transition-colors"
                        on:click=handle_submit
                        prop:disabled=move || batch_translation.is_processing.get() || index_url.get().is_empty()
                    >
                        "开始批量翻译"
                    </button>
                </div>
            </div>

            // 进度显示
            <Show when=move || batch_translation.is_processing.get()>
                <BatchProgress progress=batch_translation.progress />
            </Show>

            // 文档列表预览
            <Show when=move || !batch_translation.documents.get().is_empty()>
                <DocumentList documents=batch_translation.documents />
            </Show>

            // 翻译结果
            <Show when=move || !batch_translation.translated_docs.get().is_empty()>
                <TranslationResults docs=batch_translation.translated_docs />
            </Show>
        </div>
    }
}

#[component]
fn BatchProgress(progress: ReadSignal<crate::services::batch_service::BatchProgress>) -> impl IntoView {
    view! {
        <div class="mb-6 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg border border-blue-200 dark:border-blue-800">
            <div class="flex items-center justify-between mb-2">
                <span class="text-sm font-medium text-blue-800 dark:text-blue-200">
                    {move || {
                        match progress.get().status {
                            BatchStatus::Parsing => "解析文档索引",
                            BatchStatus::Translating => "批量翻译中",
                            BatchStatus::Packaging => "打包文件",
                            BatchStatus::Completed => "翻译完成",
                            BatchStatus::Failed(_) => "翻译失败",
                            BatchStatus::Idle => "等待中",
                        }
                    }}
                </span>
                <span class="text-sm text-blue-600 dark:text-blue-300">
                    {move || {
                        let p = progress.get();
                        if p.total > 0 {
                            format!("{}/{}", p.completed, p.total)
                        } else {
                            "处理中...".to_string()
                        }
                    }}
                </span>
            </div>
            
            <div class="w-full bg-blue-200 dark:bg-blue-800 rounded-full h-2 mb-2">
                <div 
                    class="bg-blue-600 dark:bg-blue-400 h-2 rounded-full transition-all duration-300"
                    style:width=move || {
                        let p = progress.get();
                        if p.total > 0 {
                            format!("{}%", (p.completed * 100) / p.total)
                        } else {
                            "0%".to_string()
                        }
                    }
                ></div>
            </div>
            
            <p class="text-sm text-blue-700 dark:text-blue-300">
                {move || progress.get().current_task}
            </p>
            
            <Show when=move || { progress.get().failed_count > 0 }>
                <p class="text-sm text-red-600 dark:text-red-400 mt-1">
                    {move || format!("失败: {} 个文档", progress.get().failed_count)}
                </p>
            </Show>
        </div>
    }
}

#[component]
fn DocumentList(documents: ReadSignal<Vec<crate::services::batch_service::DocumentLink>>) -> impl IntoView {
    view! {
        <div class="mb-6">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                "发现的文档 (" {move || documents.get().len()} " 个)"
            </h3>
            <div class="max-h-60 overflow-y-auto border border-gray-200 dark:border-gray-700 rounded-lg">
                <For
                    each=move || documents.get()
                    key=|doc| doc.order
                    children=move |doc| {
                        view! {
                            <div class="p-3 border-b border-gray-100 dark:border-gray-700 last:border-b-0 hover:bg-gray-50 dark:hover:bg-gray-700">
                                <div class="flex items-start gap-3">
                                    <span class="flex-shrink-0 w-8 h-6 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs font-medium rounded flex items-center justify-center">
                                        {doc.order + 1}
                                    </span>
                                    <div class="flex-1 min-w-0">
                                        <p class="text-sm font-medium text-gray-900 dark:text-white truncate" style:margin-left=format!("{}rem", doc.level as f32 * 1.0)>
                                            {doc.title.clone()}
                                        </p>
                                        <p class="text-xs text-gray-500 dark:text-gray-400 truncate">
                                            {doc.url.clone()}
                                        </p>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
fn TranslationResults(docs: ReadSignal<Vec<crate::services::batch_service::TranslatedDocument>>) -> impl IntoView {
    view! {
        <div class="mb-6">
            <h3 class="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                "翻译结果 (" {move || docs.get().len()} " 个文档)"
            </h3>
            <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
                <div class="flex items-center gap-2 mb-2">
                    <svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                    </svg>
                    <span class="text-green-800 dark:text-green-200 font-medium">
                        "批量翻译完成"
                    </span>
                </div>
                <p class="text-green-700 dark:text-green-300 text-sm mb-3">
                    "所有文档已成功翻译并打包为ZIP文件。文件下载应该已自动开始。"
                </p>
                
                <div class="space-y-2">
                    <For
                        each=move || docs.get()
                        key=|doc| doc.link.order
                        children=move |doc| {
                            view! {
                                <div class="flex items-center justify-between text-sm">
                                    <span class="text-green-700 dark:text-green-300">
                                        {doc.link.title.clone()}
                                    </span>
                                    <span class="text-green-600 dark:text-green-400 font-mono">
                                        {doc.file_name.clone()}
                                    </span>
                                </div>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}