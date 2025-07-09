use crate::hooks::use_batch_translation::use_batch_translation;
use crate::services::batch_service::BatchStatus;
use leptos::*;
use wasm_bindgen::JsCast;

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
fn BatchProgress(
    progress: ReadSignal<crate::services::batch_service::BatchProgress>,
) -> impl IntoView {
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
fn DocumentList(
    documents: ReadSignal<Vec<crate::services::batch_service::DocumentLink>>,
) -> impl IntoView {
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
fn TranslationResults(
    docs: ReadSignal<Vec<crate::services::batch_service::TranslatedDocument>>,
) -> impl IntoView {
    let (select_all, set_select_all) = create_signal(true);

    // 创建文档选择状态的 RwSignal
    let docs_selection = create_rw_signal(docs.get_untracked());

    // 更新文档选择状态
    create_effect(move |_| {
        docs_selection.set(docs.get());
    });

    // 全选/取消全选逻辑
    let toggle_select_all = move |_| {
        let new_select_all = !select_all.get();
        set_select_all.set(new_select_all);

        docs_selection.update(|docs| {
            for doc in docs.iter_mut() {
                doc.selected = new_select_all;
            }
        });
    };

    // 单个文档选择切换
    let toggle_doc_selection = move |index: usize| {
        docs_selection.update(|docs| {
            if let Some(doc) = docs.get_mut(index) {
                doc.selected = !doc.selected;
            }
        });

        // 检查是否需要更新全选状态
        let all_selected = docs_selection.get().iter().all(|doc| doc.selected);
        let none_selected = docs_selection.get().iter().all(|doc| !doc.selected);

        if all_selected {
            set_select_all.set(true);
        } else if none_selected {
            set_select_all.set(false);
        }
    };

    // 下载选中文档
    let download_selected = move |_| {
        let selected_docs: Vec<_> = docs_selection
            .get()
            .into_iter()
            .filter(|doc| doc.selected)
            .collect();

        if selected_docs.is_empty() {
            web_sys::console::log_1(&"没有选中任何文档".into());
            return;
        }

        // 创建批量翻译服务实例并生成ZIP
        use crate::services::batch_service::BatchTranslationService;
        use crate::types::api_types::AppConfig;

        let config = AppConfig::default();
        let service = BatchTranslationService::new(&config);

        match service.create_compressed_archive(&selected_docs) {
            Ok(zip_data) => {
                // 触发下载
                let array = js_sys::Array::new();
                array.push(&js_sys::Uint8Array::from(&zip_data[..]));
                let blob = web_sys::Blob::new_with_u8_array_sequence(&array).unwrap();

                let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                let document = web_sys::window().unwrap().document().unwrap();
                let anchor = document
                    .create_element("a")
                    .unwrap()
                    .dyn_into::<web_sys::HtmlAnchorElement>()
                    .unwrap();

                // 生成智能压缩文件名
                let archive_name = if let Some(first_doc) = selected_docs.first() {
                    generate_smart_archive_name(&first_doc.link.url)
                } else {
                    format!(
                        "translated_docs_{}.tar.gz",
                        js_sys::Date::new_0().get_time() as u64
                    )
                };

                anchor.set_href(&url);
                anchor.set_download(&archive_name);
                anchor.click();

                web_sys::Url::revoke_object_url(&url).unwrap();
                web_sys::console::log_1(
                    &format!("已下载 {} 个选中文档", selected_docs.len()).into(),
                );
            }
            Err(e) => {
                web_sys::console::log_1(&format!("下载失败: {}", e).into());
            }
        }
    };

    view! {
        <div class="mb-6">
            <div class="flex items-center justify-between mb-3">
                <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                    "翻译结果 (" {move || docs_selection.get().len()} " 个文档)"
                </h3>
                <div class="flex items-center gap-3">
                    <label class="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
                        <input
                            type="checkbox"
                            class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500"
                            prop:checked=move || select_all.get()
                            on:change=toggle_select_all
                        />
                        "全选"
                    </label>
                    <button
                        type="button"
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-400 text-white text-sm font-medium rounded-lg transition-colors"
                        on:click=download_selected
                        prop:disabled=move || docs_selection.get().iter().all(|doc| !doc.selected)
                    >
                        "下载选中文档"
                    </button>
                </div>
            </div>

            <div class="bg-green-50 dark:bg-green-900/20 border border-green-200 dark:border-green-800 rounded-lg p-4">
                <div class="flex items-center gap-2 mb-4">
                    <svg class="w-5 h-5 text-green-600 dark:text-green-400" fill="currentColor" viewBox="0 0 20 20">
                        <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clip-rule="evenodd"></path>
                    </svg>
                    <span class="text-green-800 dark:text-green-200 font-medium">
                        "批量翻译完成"
                    </span>
                </div>
                <p class="text-green-700 dark:text-green-300 text-sm mb-4">
                    "选择要下载的文档，系统将按索引顺序打包为tar.gz文件。所有文件统一放在documents文件夹中，文件名包含路径和序号信息。"
                </p>

                // 显示文档列表（按序号排序）
                <DocumentFolderView docs_selection=docs_selection toggle_doc_selection=toggle_doc_selection />
            </div>
        </div>
    }
}

#[component]
fn DocumentFolderView(
    docs_selection: RwSignal<Vec<crate::services::batch_service::TranslatedDocument>>,
    toggle_doc_selection: impl Fn(usize) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
            <div class="bg-gray-50 dark:bg-gray-800 px-4 py-2 border-b border-gray-200 dark:border-gray-700">
                <h4 class="text-sm font-medium text-gray-900 dark:text-white flex items-center gap-2">
                    <svg class="w-4 h-4 text-gray-500" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                    </svg>
                    "翻译文档"
                    <span class="text-xs text-gray-500 ml-1">
                        "(" {move || docs_selection.get().len()} " 个文件)"
                    </span>
                </h4>
            </div>
            <div class="divide-y divide-gray-200 dark:divide-gray-700 max-h-96 overflow-y-auto">
                <For
                    each=move || {
                        let docs = docs_selection.get();
                        // 按序号排序，不分组
                        docs.into_iter().enumerate().collect::<Vec<_>>()
                    }
                    key=|(index, _doc)| *index
                    children=move |(index, doc)| {
                        view! {
                            <div class="p-3 hover:bg-gray-50 dark:hover:bg-gray-700">
                                <div class="flex items-center gap-3">
                                    <input
                                        type="checkbox"
                                        class="rounded border-gray-300 dark:border-gray-600 text-blue-600 focus:ring-blue-500"
                                        prop:checked=move || docs_selection.get().get(index).map(|d| d.selected).unwrap_or(false)
                                        on:change=move |_| toggle_doc_selection(index)
                                    />
                                    <div class="flex items-center gap-3 flex-1 min-w-0">
                                        <span class="flex-shrink-0 w-8 h-6 bg-blue-100 dark:bg-blue-900 text-blue-800 dark:text-blue-200 text-xs font-medium rounded flex items-center justify-center">
                                            {doc.link.order + 1}
                                        </span>
                                        <div class="flex-1 min-w-0">
                                            <p class="text-sm font-medium text-gray-900 dark:text-white truncate">
                                                {doc.link.title.clone()}
                                            </p>
                                            <div class="flex items-center gap-4 mt-1">
                                                <span class="text-xs text-gray-500 dark:text-gray-400 font-mono">
                                                    {doc.file_name.clone()}
                                                </span>
                                                <button
                                                    type="button"
                                                    class="text-xs text-blue-600 dark:text-blue-400 hover:text-blue-800 dark:hover:text-blue-200"
                                                    on:click=move |_| {
                                                        // 下载单个文件
                                                        use crate::services::batch_service::BatchTranslationService;
                                                        use crate::types::api_types::AppConfig;

                                                        let config = AppConfig::default();
                                                        let service = BatchTranslationService::new(&config);
                                                        let file_data = service.create_single_file_download(&doc);

                                                        let array = js_sys::Array::new();
                                                        array.push(&js_sys::Uint8Array::from(&file_data[..]));
                                                        let blob = web_sys::Blob::new_with_u8_array_sequence(&array).unwrap();

                                                        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
                                                        let document = web_sys::window().unwrap().document().unwrap();
                                                        let anchor = document.create_element("a").unwrap()
                                                            .dyn_into::<web_sys::HtmlAnchorElement>().unwrap();

                                                        anchor.set_href(&url);
                                                        let download_filename = format!("{:03}_{}", doc.link.order + 1, doc.file_name);
                                                        anchor.set_download(&download_filename);
                                                        anchor.click();

                                                        web_sys::Url::revoke_object_url(&url).unwrap();
                                                    }
                                                >
                                                    "单独下载"
                                                </button>
                                            </div>
                                        </div>
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

/// 生成智能压缩文件名：域名_时间戳.tar.gz
fn generate_smart_archive_name(url: &str) -> String {
    let domain = extract_domain_from_url(url);
    let clean_domain = clean_domain_name(&domain);

    // 生成简洁的时间戳：YYYYMMDD_HHMMSS
    let date = js_sys::Date::new_0();
    let year = date.get_full_year();
    let month = date.get_month() + 1; // getMonth() 返回 0-11
    let day = date.get_date();
    let hours = date.get_hours();
    let minutes = date.get_minutes();
    let seconds = date.get_seconds();

    let timestamp = format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hours, minutes, seconds
    );

    format!("{}_{}.tar.gz", clean_domain, timestamp)
}

/// 清理域名，移除特殊字符
fn clean_domain_name(domain: &str) -> String {
    domain
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

/// 从URL提取域名
fn extract_domain_from_url(url: &str) -> String {
    // 尝试使用URL解析
    if let Some(start) = url.find("://") {
        let after_protocol = &url[start + 3..];
        if let Some(end) = after_protocol.find('/') {
            after_protocol[..end].to_string()
        } else if let Some(end) = after_protocol.find(':') {
            after_protocol[..end].to_string()
        } else {
            after_protocol.to_string()
        }
    } else {
        "docs".to_string()
    }
}
