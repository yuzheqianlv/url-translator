use crate::error::{use_error_handler, AppError};
use crate::services::history_service::{ExportFormat, HistoryService, HistoryStatistics};
use crate::types::history::{HistoryEntry, HistoryFilter, HistorySortBy};
use leptos::*;
use wasm_bindgen::JsCast;

pub struct UseHistoryReturn {
    pub entries: ReadSignal<Vec<HistoryEntry>>,
    pub statistics: ReadSignal<Option<HistoryStatistics>>,
    pub is_loading: ReadSignal<bool>,
    pub current_filter: ReadSignal<HistoryFilter>,
    pub current_sort: ReadSignal<HistorySortBy>,
    pub add_entry: WriteSignal<Option<HistoryEntry>>,
    pub set_filter: WriteSignal<HistoryFilter>,
    pub set_sort: WriteSignal<HistorySortBy>,
    pub delete_entry: WriteSignal<Option<String>>,
    pub clear_history: WriteSignal<bool>,
    pub export_history: WriteSignal<Option<ExportFormat>>,
}

pub fn use_history() -> UseHistoryReturn {
    let error_handler = use_error_handler();
    let history_service = HistoryService::new();

    let (entries, set_entries) = create_signal(Vec::<HistoryEntry>::new());
    let (statistics, set_statistics) = create_signal(None::<HistoryStatistics>);
    let (is_loading, set_is_loading) = create_signal(false);
    let (current_filter, set_current_filter) = create_signal(HistoryFilter::default());
    let (current_sort, set_current_sort) = create_signal(HistorySortBy::default());

    // 触发信号
    let (add_trigger, set_add_trigger) = create_signal(None::<HistoryEntry>);
    let (delete_trigger, set_delete_trigger) = create_signal(None::<String>);
    let (clear_trigger, set_clear_trigger) = create_signal(false);
    let (export_trigger, set_export_trigger) = create_signal(None::<ExportFormat>);

    // 加载历史记录
    let load_entries = {
        let history_service = history_service.clone();
        let set_entries = set_entries.clone();
        let set_statistics = set_statistics.clone();
        let set_is_loading = set_is_loading.clone();

        move |filter: &HistoryFilter, sort: &HistorySortBy| {
            set_is_loading.set(true);

            match history_service.search_entries(filter, sort) {
                Ok(filtered_entries) => {
                    set_entries.set(filtered_entries);

                    // 更新统计信息
                    match history_service.get_statistics() {
                        Ok(stats) => set_statistics.set(Some(stats)),
                        Err(e) => {
                            error_handler
                                .handle_error(AppError::config(format!("获取统计信息失败: {}", e)));
                        }
                    }
                }
                Err(e) => {
                    error_handler
                        .handle_error(AppError::config(format!("加载历史记录失败: {}", e)));
                }
            }

            set_is_loading.set(false);
        }
    };

    // 初始加载
    let load_entries_clone = load_entries.clone();
    create_effect(move |_| {
        load_entries_clone(&HistoryFilter::default(), &HistorySortBy::default());
    });

    // 过滤器或排序变化时重新加载
    create_effect({
        let load_entries = load_entries.clone();
        move |_| {
            let filter = current_filter.get();
            let sort = current_sort.get();
            load_entries(&filter, &sort);
        }
    });

    // 处理添加条目
    create_effect({
        let history_service = history_service.clone();
        let load_entries = load_entries.clone();
        move |_| {
            if let Some(entry) = add_trigger.get() {
                match history_service.add_entry(entry) {
                    Ok(()) => {
                        let filter = current_filter.get();
                        let sort = current_sort.get();
                        load_entries(&filter, &sort);
                        web_sys::console::log_1(&"历史记录添加成功".into());
                    }
                    Err(e) => {
                        error_handler
                            .handle_error(AppError::config(format!("添加历史记录失败: {}", e)));
                    }
                }
            }
        }
    });

    // 处理删除条目
    create_effect({
        let history_service = history_service.clone();
        let load_entries = load_entries.clone();
        move |_| {
            if let Some(id) = delete_trigger.get() {
                match history_service.delete_entry(&id) {
                    Ok(()) => {
                        let filter = current_filter.get();
                        let sort = current_sort.get();
                        load_entries(&filter, &sort);
                        web_sys::console::log_1(&"历史记录删除成功".into());
                    }
                    Err(e) => {
                        error_handler
                            .handle_error(AppError::config(format!("删除历史记录失败: {}", e)));
                    }
                }
            }
        }
    });

    // 处理清空历史
    create_effect({
        let history_service = history_service.clone();
        let load_entries = load_entries.clone();
        move |_| {
            if clear_trigger.get() {
                match history_service.clear_history() {
                    Ok(()) => {
                        let filter = current_filter.get();
                        let sort = current_sort.get();
                        load_entries(&filter, &sort);
                        web_sys::console::log_1(&"历史记录清空成功".into());
                    }
                    Err(e) => {
                        error_handler
                            .handle_error(AppError::config(format!("清空历史记录失败: {}", e)));
                    }
                }
                set_clear_trigger.set(false);
            }
        }
    });

    // 处理导出历史
    create_effect({
        let history_service = history_service.clone();
        move |_| {
            if let Some(format) = export_trigger.get() {
                match history_service.export_history(format) {
                    Ok(content) => {
                        // 创建下载
                        let filename = match export_trigger.get().unwrap() {
                            ExportFormat::Json => "translation_history.json",
                            ExportFormat::Csv => "translation_history.csv",
                            ExportFormat::Markdown => "translation_history.md",
                        };

                        if let Err(e) = create_and_download_file(&content, filename, "text/plain") {
                            error_handler
                                .handle_error(AppError::config(format!("下载文件失败: {:?}", e)));
                        } else {
                            web_sys::console::log_1(&"历史记录导出成功".into());
                        }
                    }
                    Err(e) => {
                        error_handler
                            .handle_error(AppError::config(format!("导出历史记录失败: {}", e)));
                    }
                }
            }
        }
    });

    UseHistoryReturn {
        entries,
        statistics,
        is_loading,
        current_filter,
        current_sort,
        add_entry: set_add_trigger,
        set_filter: set_current_filter,
        set_sort: set_current_sort,
        delete_entry: set_delete_trigger,
        clear_history: set_clear_trigger,
        export_history: set_export_trigger,
    }
}

fn create_and_download_file(
    content: &str,
    filename: &str,
    _mime_type: &str,
) -> Result<(), wasm_bindgen::JsValue> {
    let window = web_sys::window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;

    let blob_parts = js_sys::Array::new();
    blob_parts.push(&wasm_bindgen::JsValue::from_str(content));

    let blob = web_sys::Blob::new_with_str_sequence(&blob_parts)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob)?;

    let anchor = document.create_element("a")?;
    anchor.set_attribute("href", &url)?;
    anchor.set_attribute("download", filename)?;
    anchor.set_attribute("style", "display: none")?;

    document.body().unwrap().append_child(&anchor)?;

    let html_anchor = anchor.dyn_ref::<web_sys::HtmlAnchorElement>().unwrap();
    html_anchor.click();

    document.body().unwrap().remove_child(&anchor)?;
    web_sys::Url::revoke_object_url(&url)?;

    Ok(())
}
