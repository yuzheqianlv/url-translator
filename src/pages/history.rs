use leptos::*;
use crate::hooks::use_history::use_history;
use crate::types::history::{HistoryFilter, HistorySortBy};
use crate::services::history_service::ExportFormat;

#[component]
pub fn HistoryPage() -> impl IntoView {
    let history = use_history();
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
    
    view! {
        <div class="max-w-6xl mx-auto space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold themed-text">
                    "翻译历史记录"
                </h1>
                
                <div class="flex space-x-2">
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90 themed-button-secondary"
                        on:click=move |_| handle_export(ExportFormat::Json)
                    >
                        "导出 JSON"
                    </button>
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90 themed-button-secondary"
                        on:click=move |_| handle_export(ExportFormat::Csv)
                    >
                        "导出 CSV"
                    </button>
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90 themed-button-danger"
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
                            view! {
                                <div class="rounded-lg p-4 themed-bg-surface0">
                                    <div class="text-2xl font-bold themed-text">{stats.total_entries}</div>
                                    <div class="text-sm themed-subtext">"总翻译数"</div>
                                </div>
                                <div class="rounded-lg p-4 themed-bg-surface0">
                                    <div class="text-2xl font-bold themed-text">{stats.total_words}</div>
                                    <div class="text-sm themed-subtext">"总字数"</div>
                                </div>
                                <div class="rounded-lg p-4 themed-bg-surface0">
                                    <div class="text-lg font-bold themed-text">
                                        {stats.most_used_language_pair.unwrap_or_else(|| "无".to_string())}
                                    </div>
                                    <div class="text-sm themed-subtext">"常用语言对"</div>
                                </div>
                                <div class="rounded-lg p-4 themed-bg-surface0">
                                    <div class="text-lg font-bold themed-text">
                                        {stats.most_translated_domain.unwrap_or_else(|| "无".to_string())}
                                    </div>
                                    <div class="text-sm themed-subtext">"常翻译域名"</div>
                                </div>
                            }.into_view()
                        } else {
                            view! { <div></div> }.into_view()
                        }
                    }}
                </div>
            </Show>
            
            // 搜索和过滤
            <div class="rounded-lg shadow-lg p-6 themed-bg-surface0">
                <div class="flex flex-col md:flex-row gap-4">
                    <div class="flex-1">
                        <input
                            type="text"
                            class="w-full px-4 py-2 border rounded-md focus:ring-2 focus:border-transparent themed-input"
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
                            class="px-4 py-2 rounded-md transition-colors hover:opacity-90 themed-button-primary"
                            on:click=move |_| handle_search(())
                        >
                            "搜索"
                        </button>
                        <select 
                            class="px-4 py-2 border rounded-md themed-input"
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
            <div class="rounded-lg shadow-lg themed-bg-surface0">
                <Show
                    when=move || !history.is_loading.get()
                    fallback=|| view! {
                        <div class="p-8 text-center themed-subtext">
                            <div class="animate-spin w-8 h-8 border-2 border-current border-t-transparent rounded-full mx-auto mb-4"></div>
                            "加载中..."
                        </div>
                    }
                >
                    <Show
                        when=move || !history.entries.get().is_empty()
                        fallback=|| view! {
                            <div class="p-8 text-center themed-subtext">
                                <svg class="w-16 h-16 mx-auto mb-4 themed-subtext0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                                </svg>
                                <p class="text-lg font-medium">"暂无历史记录"</p>
                                <p class="text-sm">"开始翻译一些内容吧！"</p>
                            </div>
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
                                                    <h3 class="font-medium themed-text">{entry.title.clone()}</h3>
                                                    <div class="flex items-center space-x-4 text-sm themed-subtext mt-1">
                                                        <span>{entry.get_formatted_date()}</span>
                                                        <span>{format!("{} -> {}", entry.source_lang, entry.target_lang)}</span>
                                                        <span>{format!("{} 字", entry.word_count)}</span>
                                                    </div>
                                                    <div class="text-sm themed-subtext0 mt-1 truncate">
                                                        {entry.url.clone()}
                                                    </div>
                                                </div>
                                                
                                                <div class="flex items-center space-x-2">
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
                                                    <div>
                                                        <h4 class="font-medium themed-text mb-2">"原文内容"</h4>
                                                        <div class="p-3 rounded themed-content-bg max-h-40 overflow-y-auto">
                                                            <pre class="whitespace-pre-wrap text-sm">{entry.original_content.clone()}</pre>
                                                        </div>
                                                    </div>
                                                    <div>
                                                        <h4 class="font-medium themed-text mb-2">"翻译内容"</h4>
                                                        <div class="p-3 rounded themed-content-bg max-h-40 overflow-y-auto">
                                                            <pre class="whitespace-pre-wrap text-sm">{entry.translated_content.clone()}</pre>
                                                        </div>
                                                    </div>
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