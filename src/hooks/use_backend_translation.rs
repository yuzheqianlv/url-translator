//! 后端翻译Hook
//! 
//! 这个Hook提供了与后端API集成的翻译功能，包括：
//! - 使用后端翻译服务
//! - 翻译历史管理
//! - 用户认证集成

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::services::api_client::{TranslateUrlRequest, TranslationResponse};
use crate::hooks::use_auth::{use_auth, AuthStatus};
use crate::error::{use_error_handler, AppError};

/// 后端翻译状态
#[derive(Clone, Debug)]
pub enum BackendTranslationStatus {
    Idle,
    Submitting,
    Processing,
    Completed,
    Failed(String),
}

/// 后端翻译Hook返回值
pub struct UseBackendTranslationReturn {
    /// 当前翻译状态
    pub status: ReadSignal<BackendTranslationStatus>,
    /// 翻译结果
    pub translation_result: ReadSignal<Option<TranslationResponse>>,
    /// 是否正在加载
    pub is_loading: Memo<bool>,
    /// 进度消息
    pub progress_message: ReadSignal<String>,
    /// 开始翻译函数
    pub translate: WriteSignal<Option<TranslateUrlRequest>>,
}

pub fn use_backend_translation() -> UseBackendTranslationReturn {
    let auth = use_auth();
    let error_handler = use_error_handler();

    // 创建信号
    let (status, set_status) = create_signal(BackendTranslationStatus::Idle);
    let (translation_result, set_translation_result) = create_signal(None::<TranslationResponse>);
    let (progress_message, set_progress_message) = create_signal(String::new());
    let (translate_trigger, set_translate_trigger) = create_signal(None::<TranslateUrlRequest>);

    // 从状态派生is_loading
    let is_loading = create_memo(move |_| {
        matches!(
            status.get(),
            BackendTranslationStatus::Submitting | BackendTranslationStatus::Processing
        )
    });

    // 翻译Effect
    create_effect({
        let auth = auth.clone();
        let set_status = set_status;
        let set_translation_result = set_translation_result;
        let set_progress_message = set_progress_message;
        let error_handler = error_handler.clone();

        move |_| {
            if let Some(translate_request) = translate_trigger.get() {
                // 检查认证状态
                match auth.auth_status.get() {
                    AuthStatus::Authenticated(_) => {
                        set_status.set(BackendTranslationStatus::Submitting);
                        set_progress_message.set("正在提交翻译请求...".to_string());
                        set_translation_result.set(None);

                        let client = auth.api_client.get();
                        let set_status = set_status;
                        let set_translation_result = set_translation_result;
                        let set_progress_message = set_progress_message;
                        let error_handler = error_handler.clone();

                        spawn_local(async move {
                            web_sys::console::log_1(&format!("开始后端翻译: {}", translate_request.url).into());

                            match client.translate_url(translate_request).await {
                                Ok(translation) => {
                                    web_sys::console::log_1(&"后端翻译成功".into());
                                    set_translation_result.set(Some(translation));
                                    set_status.set(BackendTranslationStatus::Completed);
                                    set_progress_message.set("翻译完成".to_string());
                                }
                                Err(e) => {
                                    web_sys::console::log_1(&format!("后端翻译失败: {}", e).into());
                                    let error_msg = format!("翻译失败: {}", e);
                                    set_status.set(BackendTranslationStatus::Failed(error_msg.clone()));
                                    set_progress_message.set(String::new());
                                    error_handler.handle_error(AppError::api_simple(error_msg));
                                }
                            }
                        });
                    }
                    AuthStatus::Unauthenticated => {
                        error_handler.handle_error(AppError::auth("请先登录后再使用翻译功能"));
                    }
                    AuthStatus::Authenticating => {
                        error_handler.handle_error(AppError::auth("正在登录中，请稍后重试"));
                    }
                    _ => {
                        error_handler.handle_error(AppError::auth("认证状态异常，请重新登录"));
                    }
                }
            }
        }
    });

    UseBackendTranslationReturn {
        status,
        translation_result,
        is_loading,
        progress_message,
        translate: set_translate_trigger,
    }
}

/// 翻译历史Hook
pub struct UseTranslationHistoryReturn {
    /// 翻译历史列表
    pub history: ReadSignal<Vec<TranslationResponse>>,
    /// 是否正在加载
    pub is_loading: ReadSignal<bool>,
    /// 总数
    pub total: ReadSignal<i64>,
    /// 当前页
    pub current_page: ReadSignal<u32>,
    /// 每页数量
    pub per_page: ReadSignal<u32>,
    /// 加载历史函数
    pub load_history: WriteSignal<Option<(u32, u32)>>, // (page, per_page)
    /// 删除翻译函数
    pub delete_translation: WriteSignal<Option<String>>, // translation_id
}

pub fn use_translation_history() -> UseTranslationHistoryReturn {
    let auth = use_auth();
    let error_handler = use_error_handler();

    // 创建信号
    let (history, set_history) = create_signal(Vec::<TranslationResponse>::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (total, set_total) = create_signal(0i64);
    let (current_page, set_current_page) = create_signal(1u32);
    let (per_page, set_per_page) = create_signal(20u32);
    let (load_trigger, set_load_trigger) = create_signal(None::<(u32, u32)>);
    let (delete_trigger, set_delete_trigger) = create_signal(None::<String>);

    // 自动加载历史记录（当用户登录时）
    create_effect({
        let auth = auth.clone();
        let set_load_trigger = set_load_trigger;

        move |_| {
            if let AuthStatus::Authenticated(_) = auth.auth_status.get() {
                // 用户登录后自动加载第一页历史记录
                set_load_trigger.set(Some((1, 20)));
            }
        }
    });

    // 加载历史记录Effect
    create_effect({
        let auth = auth.clone();
        let set_history = set_history;
        let set_is_loading = set_is_loading;
        let set_total = set_total;
        let set_current_page = set_current_page;
        let set_per_page = set_per_page;
        let error_handler = error_handler.clone();

        move |_| {
            if let Some((page, per_page_param)) = load_trigger.get() {
                if let AuthStatus::Authenticated(_) = auth.auth_status.get() {
                    set_is_loading.set(true);

                    let client = auth.api_client.get();
                    let set_history = set_history;
                    let set_is_loading = set_is_loading;
                    let set_total = set_total;
                    let set_current_page = set_current_page;
                    let set_per_page = set_per_page;
                    let error_handler = error_handler.clone();

                    spawn_local(async move {
                        web_sys::console::log_1(&format!("加载翻译历史: page={}, per_page={}", page, per_page_param).into());

                        match client.get_translation_history(Some(page), Some(per_page_param)).await {
                            Ok(history_response) => {
                                web_sys::console::log_1(&format!("历史记录加载成功，共{}条", history_response.translations.len()).into());
                                set_history.set(history_response.translations);
                                set_total.set(history_response.total);
                                set_current_page.set(history_response.page);
                                set_per_page.set(history_response.per_page);
                            }
                            Err(e) => {
                                web_sys::console::log_1(&format!("历史记录加载失败: {}", e).into());
                                error_handler.handle_error(AppError::api_simple(format!("加载历史记录失败: {}", e)));
                            }
                        }

                        set_is_loading.set(false);
                    });
                }
            }
        }
    });

    // 删除翻译Effect
    create_effect({
        let auth = auth.clone();
        let set_load_trigger = set_load_trigger;
        let current_page = current_page;
        let per_page = per_page;
        let error_handler = error_handler.clone();

        move |_| {
            if let Some(translation_id) = delete_trigger.get() {
                if let AuthStatus::Authenticated(_) = auth.auth_status.get() {
                    let client = auth.api_client.get();
                    let set_load_trigger = set_load_trigger;
                    let current_page = current_page.get();
                    let per_page = per_page.get();
                    let error_handler = error_handler.clone();

                    spawn_local(async move {
                        web_sys::console::log_1(&format!("删除翻译: {}", translation_id).into());

                        match client.delete_translation(&translation_id).await {
                            Ok(()) => {
                                web_sys::console::log_1(&"翻译删除成功".into());
                                // 重新加载当前页
                                set_load_trigger.set(Some((current_page, per_page)));
                            }
                            Err(e) => {
                                web_sys::console::log_1(&format!("翻译删除失败: {}", e).into());
                                error_handler.handle_error(AppError::api_simple(format!("删除失败: {}", e)));
                            }
                        }
                    });
                }
            }
        }
    });

    UseTranslationHistoryReturn {
        history,
        is_loading,
        total,
        current_page,
        per_page,
        load_history: set_load_trigger,
        delete_translation: set_delete_trigger,
    }
}

/// 搜索翻译Hook
pub struct UseTranslationSearchReturn {
    /// 搜索结果
    pub search_results: ReadSignal<Vec<crate::services::api_client::SearchResult>>,
    /// 是否正在搜索
    pub is_searching: ReadSignal<bool>,
    /// 搜索总数
    pub total: ReadSignal<i64>,
    /// 搜索时间
    pub search_time_ms: ReadSignal<u64>,
    /// 搜索建议
    pub suggestions: ReadSignal<Vec<String>>,
    /// 执行搜索函数
    pub search: WriteSignal<Option<crate::services::api_client::SearchRequest>>,
}

pub fn use_translation_search() -> UseTranslationSearchReturn {
    let auth = use_auth();
    let error_handler = use_error_handler();

    // 创建信号
    let (search_results, set_search_results) = create_signal(Vec::new());
    let (is_searching, set_is_searching) = create_signal(false);
    let (total, set_total) = create_signal(0i64);
    let (search_time_ms, set_search_time_ms) = create_signal(0u64);
    let (suggestions, set_suggestions) = create_signal(Vec::<String>::new());
    let (search_trigger, set_search_trigger) = create_signal(None::<crate::services::api_client::SearchRequest>);

    // 搜索Effect
    create_effect({
        let auth = auth.clone();
        let set_search_results = set_search_results;
        let set_is_searching = set_is_searching;
        let set_total = set_total;
        let set_search_time_ms = set_search_time_ms;
        let set_suggestions = set_suggestions;
        let error_handler = error_handler.clone();

        move |_| {
            if let Some(search_request) = search_trigger.get() {
                if let AuthStatus::Authenticated(_) = auth.auth_status.get() {
                    set_is_searching.set(true);

                    let client = auth.api_client.get();
                    let set_search_results = set_search_results;
                    let set_is_searching = set_is_searching;
                    let set_total = set_total;
                    let set_search_time_ms = set_search_time_ms;
                    let set_suggestions = set_suggestions;
                    let error_handler = error_handler.clone();

                    spawn_local(async move {
                        web_sys::console::log_1(&format!("开始搜索: {}", search_request.query).into());

                        match client.search_translations(search_request).await {
                            Ok(search_response) => {
                                web_sys::console::log_1(&format!("搜索成功，找到{}条结果", search_response.results.len()).into());
                                set_search_results.set(search_response.results);
                                set_total.set(search_response.total);
                                set_search_time_ms.set(search_response.search_time_ms);
                                set_suggestions.set(search_response.suggestions);
                            }
                            Err(e) => {
                                web_sys::console::log_1(&format!("搜索失败: {}", e).into());
                                error_handler.handle_error(AppError::api_simple(format!("搜索失败: {}", e)));
                                // 清空结果
                                set_search_results.set(Vec::new());
                                set_total.set(0);
                                set_search_time_ms.set(0);
                                set_suggestions.set(Vec::new());
                            }
                        }

                        set_is_searching.set(false);
                    });
                } else {
                    error_handler.handle_error(AppError::auth("请先登录后再使用搜索功能"));
                }
            }
        }
    });

    UseTranslationSearchReturn {
        search_results,
        is_searching,
        total,
        search_time_ms,
        suggestions,
        search: set_search_trigger,
    }
}