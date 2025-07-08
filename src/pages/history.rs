use leptos::*;
use crate::hooks::use_history::use_history;
use crate::types::history::{HistoryFilter, HistorySortBy, HistoryEntryType};
use crate::services::history_service::{ExportFormat, HistoryService};
use crate::theme::use_theme_context;
use wasm_bindgen::JsCast;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let history = use_history();
    let theme_context = use_theme_context();
    let (search_term, set_search_term) = create_signal(String::new());
    let (selected_entry_id, set_selected_entry_id) = create_signal(None::<String>);
    
    // 搜索处理
    let handle_search = move |_| {
        let term = search_term.get();
        let filter = HistoryFilter {
            search_term: if term.is_empty() { None } else { Some(term) },
            ..Default::default()
        };
        history.set_filter.set(filter);
    };
    
    // 排序处理
    let handle_sort_change = move |sort: HistorySortBy| {
        history.set_sort.set(sort);
    };
    
    // 删除条目
    let handle_delete = move |id: String| {
        history.delete_entry.set(Some(id));
    };
    
    // 清空历史
    let handle_clear = move |_| {
        if web_sys::window()
            .and_then(|w| w.confirm_with_message("确定要清空所有历史记录吗？").ok())
            .unwrap_or(false)
        {
            history.clear_history.set(true);
        }
    };
    
    // 导出历史
    let handle_export = move |format: ExportFormat| {
        history.export_history.set(Some(format));
    };
    
    // 下载单页翻译
    let download_single_page = move |entry_id: String| {
        let history_service = HistoryService::new();
        match history_service.download_single_page(&entry_id) {
            Ok(file_data) => {
                let filename = format!("translation_{}.md", entry_id);
                if let Err(e) = trigger_download(&file_data, &filename) {
                    web_sys::console::log_1(&format!("下载失败: {}", e).into());
                }
            }
            Err(e) => {
                web_sys::console::log_1(&format!("生成下载文件失败: {}", e).into());
            }
        }
    };
    
    // 下载批量翻译
    let download_batch_translation = move |entry_id: String, selected_docs: Option<Vec<usize>>| {
        let history_service = HistoryService::new();
        match history_service.download_batch_translation(&entry_id, selected_docs) {
            Ok(file_data) => {
                let filename = format!("batch_translation_{}.tar.gz", entry_id);
                if let Err(e) = trigger_download(&file_data, &filename) {
                    web_sys::console::log_1(&format!("下载失败: {}", e).into());
                }
            }
            Err(e) => {
                web_sys::console::log_1(&format!("生成下载文件失败: {}", e).into());
            }
        }
    };
    
    view! {
        <div class="max-w-6xl mx-auto space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold" style=move || theme_context.get().theme.text_style()>
                    "翻译历史记录"
                </h1>
                
                <div class="flex space-x-2">
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90"
                        style=move || theme_context.get().theme.button_secondary_style()
                        on:click=move |_| handle_export(ExportFormat::Json)
                    >
                        "导出 JSON"
                    </button>
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90"
                        style=move || theme_context.get().theme.button_secondary_style()
                        on:click=move |_| handle_export(ExportFormat::Csv)
                    >
                        "导出 CSV"
                    </button>
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90"
                        style=move || theme_context.get().theme.button_danger_style()
                        on:click=handle_clear
                    >
                        "清空历史"
                    </button>
                </div>
            </div>
            
            // 统计信息卡片
            <Show when=move || history.statistics.get().is_some()>
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    {move || {
                        if let Some(stats) = history.statistics.get() {
                            let theme = theme_context.get().theme;
                            view! {
                                <div class="rounded-lg p-4" style=theme.card_style()>
                                    <div class="text-2xl font-bold" style=theme.text_style()>{stats.total_entries}</div>
                                    <div class="text-sm" style=theme.subtext_style()>"总翻译数"</div>
                                </div>
                                <div class="rounded-lg p-4" style=theme.card_style()>
                                    <div class="text-2xl font-bold" style=theme.text_style()>{stats.total_words}</div>
                                    <div class="text-sm" style=theme.subtext_style()>"总字数"</div>
                                </div>
                                <div class="rounded-lg p-4" style=theme.card_style()>
                                    <div class="text-lg font-bold" style=theme.text_style()>
                                        {stats.most_used_language_pair.unwrap_or_else(|| "无".to_string())}
                                    </div>
                                    <div class="text-sm" style=theme.subtext_style()>"常用语言对"</div>
                                </div>
                                <div class="rounded-lg p-4" style=theme.card_style()>
                                    <div class="text-lg font-bold" style=theme.text_style()>
                                        {stats.most_translated_domain.unwrap_or_else(|| "无".to_string())}
                                    </div>
                                    <div class="text-sm" style=theme.subtext_style()>"常翻译域名"</div>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}
                </div>
            </Show>
            
            // 搜索和过滤
            <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
                <div class="flex flex-col md:flex-row gap-4">
                    <div class="flex-1">
                        <input
                            type="text"
                            class="w-full px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                            style=move || theme_context.get().theme.input_style()
                            placeholder="搜索标题、URL或内容..."
                            prop:value=search_term
                            on:input=move |ev| {
                                set_search_term.set(event_target_value(&ev));
                            }
                            on:keypress=move |ev| {
                                if ev.key() == "Enter" {
                                    handle_search(());
                                }
                            }
                        />
                    </div>
                    <div class="flex space-x-2">
                        <button
                            class="px-4 py-2 rounded-md transition-colors hover:opacity-90"
                            style=move || theme_context.get().theme.button_primary_style()
                            on:click=move |_| handle_search(())
                        >
                            "搜索"
                        </button>
                        <select 
                            class="px-4 py-2 rounded-md"
                            style=move || theme_context.get().theme.input_style()
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                let sort = match value.as_str() {
                                    "created_desc" => HistorySortBy::CreatedAtDesc,
                                    "created_asc" => HistorySortBy::CreatedAtAsc,
                                    "title_asc" => HistorySortBy::TitleAsc,
                                    "title_desc" => HistorySortBy::TitleDesc,
                                    "words_desc" => HistorySortBy::WordCountDesc,
                                    "words_asc" => HistorySortBy::WordCountAsc,
                                    _ => HistorySortBy::CreatedAtDesc,
                                };
                                handle_sort_change(sort);
                            }
                        >
                            <option value="created_desc">"时间 ↓"</option>
                            <option value="created_asc">"时间 ↑"</option>
                            <option value="title_asc">"标题 ↑"</option>
                            <option value="title_desc">"标题 ↓"</option>
                            <option value="words_desc">"字数 ↓"</option>
                            <option value="words_asc">"字数 ↑"</option>
                        </select>
                    </div>
                </div>
            </div>
            
            // 历史记录列表
            <div class="rounded-lg shadow-lg" style=move || theme_context.get().theme.card_style()>
                <Show
                    when=move || !history.is_loading.get()
                    fallback=move || {
                        let theme = theme_context.get().theme;
                        view! {
                            <div class="p-8 text-center" style=theme.subtext_style()>
                                <div class="animate-spin w-8 h-8 border-2 border-current border-t-transparent rounded-full mx-auto mb-4"></div>
                                "加载中..."
                            </div>
                        }
                    }
                >
                    <Show
                        when=move || !history.entries.get().is_empty()
                        fallback=move || {
                            let theme = theme_context.get().theme;
                            view! {
                                <div class="p-8 text-center" style=theme.subtext_style()>
                                    <svg class="w-16 h-16 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=theme.muted_text_style()>
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                    </svg>
                                    <p class="text-lg font-medium">"暂无历史记录"</p>
                                    <p class="text-sm">"开始翻译一些内容吧！"</p>
                                </div>
                            }
                        }
                    >
                        <div class="divide-y themed-divide">
                            <For
                                each=move || history.entries.get()
                                key=|entry| entry.id.clone()
                                children=move |entry| {
                                    let entry_id = entry.id.clone();
                                    let entry_id_for_expand = entry_id.clone();
                                    let entry_id_for_delete = entry_id.clone();
                                    let entry_id_for_download_single = entry_id.clone();
                                    let entry_id_for_download_batch = entry_id.clone();
                                    let entry_id_for_download_batch_inline = entry_id.clone();
                                    
                                    // 克隆所有在view!中需要使用的字段
                                    let entry_title = entry.title.clone();
                                    let entry_url = entry.url.clone();
                                    let entry_source_lang = entry.source_lang.clone();
                                    let entry_target_lang = entry.target_lang.clone();
                                    let entry_word_count = entry.word_count;
                                    let entry_formatted_date = entry.get_formatted_date();
                                    let entry_original_content = entry.original_content.clone();
                                    let entry_translated_content = entry.translated_content.clone();
                                    let entry_type = entry.entry_type.clone();
                                    let entry_batch_data = entry.batch_data.clone();
                                    
                                    let is_expanded = create_memo(move |_| {
                                        selected_entry_id.get() == Some(entry_id.clone())
                                    });
                                    
                                    let toggle_expand_1 = {
                                        let entry_id_for_expand = entry_id_for_expand.clone();
                                        move |_| {
                                            if is_expanded.get() {
                                                set_selected_entry_id.set(None);
                                            } else {
                                                set_selected_entry_id.set(Some(entry_id_for_expand.clone()));
                                            }
                                        }
                                    };
                                    
                                    let toggle_expand_2 = {
                                        let entry_id_for_expand = entry_id_for_expand.clone();
                                        move |_| {
                                            if is_expanded.get() {
                                                set_selected_entry_id.set(None);
                                            } else {
                                                set_selected_entry_id.set(Some(entry_id_for_expand.clone()));
                                            }
                                        }
                                    };
                                    
                                    let delete_entry = move |_| {
                                        if web_sys::window()
                                            .and_then(|w| w.confirm_with_message("确定要删除这条记录吗？").ok())
                                            .unwrap_or(false)
                                        {
                                            handle_delete(entry_id_for_delete.clone());
                                        }
                                    };
                                    
                                    view! {
                                        <div class="p-4">
                                            <div class="flex items-center justify-between">
                                                <div class="flex-1 cursor-pointer" on:click=toggle_expand_1>
                                                    <h3 class="font-medium themed-text">{entry_title.clone()}</h3>
                                                    <div class="flex items-center space-x-4 text-sm themed-subtext mt-1">
                                                        <span>{entry_formatted_date.clone()}</span>
                                                        <span>{format!("{} -> {}", entry_source_lang, entry_target_lang)}</span>
                                                        <span>{format!("{} 字", entry_word_count)}</span>
                                                    </div>
                                                    <div class="text-sm themed-subtext0 mt-1 truncate">
                                                        {entry_url.clone()}
                                                    </div>
                                                </div>
                                                
                                                <div class="flex items-center space-x-2">
                                                    // 下载按钮
                                                    {
                                                        match entry_type {
                                                            HistoryEntryType::SinglePage => {
                                                                let entry_id = entry_id_for_download_single.clone();
                                                                view! {
                                                                    <button
                                                                        class="p-2 rounded-md transition-colors hover:opacity-80 themed-button-primary"
                                                                        on:click=move |_| download_single_page(entry_id.clone())
                                                                        title="下载翻译文档"
                                                                    >
                                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                                                        </svg>
                                                                    </button>
                                                                }.into_view()
                                                            }
                                                            HistoryEntryType::BatchTranslation => {
                                                                let entry_id = entry_id_for_download_batch.clone();
                                                                view! {
                                                                    <button
                                                                        class="p-2 rounded-md transition-colors hover:opacity-80 themed-button-primary"
                                                                        on:click=move |_| download_batch_translation(entry_id.clone(), None)
                                                                        title="下载批量翻译归档"
                                                                    >
                                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3M4 7h16l-1 10a2 2 0 01-2 2H7a2 2 0 01-2-2L4 7z"/>
                                                                        </svg>
                                                                    </button>
                                                                }.into_view()
                                                            }
                                                        }
                                                    }
                                                    
                                                    <button
                                                        class="p-2 rounded-md transition-colors hover:opacity-80 themed-button-secondary"
                                                        on:click=toggle_expand_2
                                                        title="展开/收起"
                                                    >
                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path 
                                                                stroke-linecap="round" 
                                                                stroke-linejoin="round" 
                                                                stroke-width="2" 
                                                                d=move || if is_expanded.get() { 
                                                                    "M5 15l7-7 7 7" 
                                                                } else { 
                                                                    "M19 9l-7 7-7-7" 
                                                                }
                                                            />
                                                        </svg>
                                                    </button>
                                                    <button
                                                        class="p-2 rounded-md transition-colors hover:opacity-80 themed-button-danger"
                                                        on:click=delete_entry
                                                        title="删除"
                                                    >
                                                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
                                                        </svg>
                                                    </button>
                                                </div>
                                            </div>
                                            
                                            <Show when=move || is_expanded.get()>
                                                <div class="mt-4 space-y-4 border-t pt-4 themed-border-t">
                                                    {
                                                        match entry_type.clone() {
                                                            HistoryEntryType::SinglePage => {
                                                                view! {
                                                                    <div>
                                                                        <h4 class="font-medium themed-text mb-2">"原文内容"</h4>
                                                                        <div class="p-3 rounded themed-content-bg max-h-40 overflow-y-auto">
                                                                            <pre class="whitespace-pre-wrap text-sm">{entry_original_content.clone()}</pre>
                                                                        </div>
                                                                    </div>
                                                                    <div>
                                                                        <h4 class="font-medium themed-text mb-2">"翻译内容"</h4>
                                                                        <div class="p-3 rounded themed-content-bg max-h-40 overflow-y-auto">
                                                                            <pre class="whitespace-pre-wrap text-sm">{entry_translated_content.clone()}</pre>
                                                                        </div>
                                                                    </div>
                                                                }.into_view()
                                                            }
                                                            HistoryEntryType::BatchTranslation => {
                                                                if let Some(batch_data) = &entry_batch_data {
                                                                    view! {
                                                                        <div>
                                                                            <h4 class="font-medium themed-text mb-2">"批量翻译详情"</h4>
                                                                            <div class="p-3 rounded themed-content-bg">
                                                                                <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-4">
                                                                                    <div class="text-center">
                                                                                        <div class="text-lg font-bold themed-text">{batch_data.total_documents}</div>
                                                                                        <div class="text-sm themed-subtext">"总文档数"</div>
                                                                                    </div>
                                                                                    <div class="text-center">
                                                                                        <div class="text-lg font-bold text-green-600">{batch_data.successful_documents}</div>
                                                                                        <div class="text-sm themed-subtext">"成功翻译"</div>
                                                                                    </div>
                                                                                    <div class="text-center">
                                                                                        <div class="text-lg font-bold text-red-600">{batch_data.failed_documents}</div>
                                                                                        <div class="text-sm themed-subtext">"翻译失败"</div>
                                                                                    </div>
                                                                                    <div class="text-center">
                                                                                        <div class="text-lg font-bold themed-text">{entry_word_count}</div>
                                                                                        <div class="text-sm themed-subtext">"总字数"</div>
                                                                                    </div>
                                                                                </div>
                                                                                
                                                                                <div class="mb-3">
                                                                                    <div class="flex justify-between items-center mb-2">
                                                                                        <span class="text-sm font-medium themed-text">"索引URL:"</span>
                                                                                        <button
                                                                                            class="px-3 py-1 text-xs rounded themed-button-primary"
                                                                                            on:click={
                                                                                                let entry_id = entry_id_for_download_batch_inline.clone();
                                                                                                move |_| {
                                                                                                    download_batch_translation(entry_id.clone(), None);
                                                                                                }
                                                                                            }
                                                                                        >
                                                                                            "下载全部"
                                                                                        </button>
                                                                                    </div>
                                                                                    <div class="text-sm themed-subtext truncate">{batch_data.index_url.clone()}</div>
                                                                                </div>
                                                                                
                                                                                // 显示文档列表（只显示前10个，其余折叠）
                                                                                <div>
                                                                                    <h5 class="text-sm font-medium themed-text mb-2">"文档列表:"</h5>
                                                                                    <div class="max-h-60 overflow-y-auto space-y-2">
                                                                                        {batch_data.document_list.iter().take(10).enumerate().map(|(_idx, doc)| {
                                                                                            let status_color = if doc.translated { "text-green-600" } else { "text-red-600" };
                                                                                            view! {
                                                                                                <div class="flex items-center justify-between text-xs p-2 rounded themed-bg-surface1">
                                                                                                    <div class="flex-1 min-w-0">
                                                                                                        <div class=format!("font-medium {}", status_color)>
                                                                                                            {format!("{:03}. {}", doc.order + 1, doc.title)}
                                                                                                        </div>
                                                                                                        <div class="themed-subtext0 truncate">{doc.url.clone()}</div>
                                                                                                    </div>
                                                                                                    <div class="ml-2">
                                                                                                        {if doc.translated {
                                                                                                            view! { <span class="text-green-600">"✓"</span> }.into_view()
                                                                                                        } else {
                                                                                                            view! { <span class="text-red-600">"✗"</span> }.into_view()
                                                                                                        }}
                                                                                                    </div>
                                                                                                </div>
                                                                                            }
                                                                                        }).collect::<Vec<_>>()}
                                                                                        
                                                                                        {if batch_data.document_list.len() > 10 {
                                                                                            view! {
                                                                                                <div class="text-center text-sm themed-subtext py-2">
                                                                                                    {format!("还有 {} 个文档未显示...", batch_data.document_list.len() - 10)}
                                                                                                </div>
                                                                                            }.into_view()
                                                                                        } else {
                                                                                            view! {}.into_view()
                                                                                        }}
                                                                                    </div>
                                                                                </div>
                                                                            </div>
                                                                        </div>
                                                                    }.into_view()
                                                                } else {
                                                                    view! {
                                                                        <div class="p-3 rounded themed-content-bg">
                                                                            <div class="text-sm themed-subtext">"批量翻译数据缺失"</div>
                                                                        </div>
                                                                    }.into_view()
                                                                }
                                                            }
                                                        }
                                                    }
                                                </div>
                                            </Show>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </Show>
                </Show>
            </div>
        </div>
    }
}

/// 触发文件下载
fn trigger_download(data: &[u8], filename: &str) -> Result<(), String> {
    let window = web_sys::window().ok_or("无法获取window对象")?;
    let document = window.document().ok_or("无法获取document对象")?;

    // 创建Blob
    let array = js_sys::Uint8Array::new_with_length(data.len() as u32);
    array.copy_from(data);
    
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array);
    
    let blob = web_sys::Blob::new_with_u8_array_sequence(&blob_parts)
        .map_err(|_| "无法创建Blob对象")?;

    // 创建下载链接
    let url = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| "无法创建对象URL")?;

    let anchor = document.create_element("a")
        .map_err(|_| "无法创建a元素")?
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .map_err(|_| "无法转换为HtmlAnchorElement")?;

    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.style().set_property("display", "none").map_err(|_| "无法设置样式")?;

    let body = document.body().ok_or("无法获取body元素")?;
    body.append_child(&anchor).map_err(|_| "无法添加下载链接")?;

    // 触发点击下载
    anchor.click();

    // 清理
    body.remove_child(&anchor).map_err(|_| "无法移除下载链接")?;
    web_sys::Url::revoke_object_url(&url).map_err(|_| "无法释放对象URL")?;

    Ok(())
}