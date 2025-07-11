//! 异步翻译任务Hook
//! 
//! 提供异步翻译任务的状态管理和API交互

use leptos::*;
use wasm_bindgen_futures::spawn_local;
use crate::services::api_client::{ApiClient, TranslationTask, TaskStatus, TranslateUrlRequest};
use crate::hooks::use_auth::use_auth;

/// 异步翻译任务状态
#[derive(Debug, Clone)]
pub struct AsyncTranslationState {
    pub is_loading: ReadSignal<bool>,
    pub current_task: ReadSignal<Option<TranslationTask>>,
    pub error_message: ReadSignal<Option<String>>,
    pub submit_task: WriteSignal<Option<TranslateUrlRequest>>,
    pub task_id: ReadSignal<Option<String>>,
    pub cancel_task: WriteSignal<Option<String>>,
}

/// 异步翻译任务Hook
#[component]
pub fn AsyncTranslationProvider(children: Children) -> impl IntoView {
    let auth = use_auth();
    
    // 创建状态信号
    let (is_loading, set_is_loading) = create_signal(false);
    let (current_task, set_current_task) = create_signal(None::<TranslationTask>);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (submit_task, set_submit_task) = create_signal(None::<TranslateUrlRequest>);
    let (task_id, set_task_id) = create_signal(None::<String>);
    let (cancel_task, set_cancel_task) = create_signal(None::<String>);
    
    // 使用认证钩子提供的API客户端
    let api_client = auth.api_client;
    
    // 提交翻译任务
    create_effect(move |_| {
        if let Some(request) = submit_task.get() {
            set_submit_task.set(None);
            set_is_loading.set(true);
            set_error_message.set(None);
            
            let client = api_client.get();
            
            spawn_local(async move {
                match client.submit_translation_task(request).await {
                    Ok(response) => {
                        set_task_id.set(Some(response.task_id.clone()));
                        // 开始轮询任务状态
                        start_task_polling(response.task_id, client, set_current_task, set_is_loading, set_error_message);
                    }
                    Err(error) => {
                        set_is_loading.set(false);
                        set_error_message.set(Some(error));
                    }
                }
            });
        }
    });

    // 取消翻译任务
    create_effect(move |_| {
        if let Some(task_id_to_cancel) = cancel_task.get() {
            set_cancel_task.set(None);
            
            let client = api_client.get();
            
            spawn_local(async move {
                match client.cancel_translation_task(&task_id_to_cancel).await {
                    Ok(()) => {
                        // 取消成功，清除当前任务状态
                        set_current_task.set(None);
                        set_is_loading.set(false);
                        set_error_message.set(None);
                        web_sys::console::log_1(&format!("任务 {} 已取消", task_id_to_cancel).into());
                    }
                    Err(error) => {
                        set_error_message.set(Some(format!("取消任务失败: {}", error)));
                    }
                }
            });
        }
    });
    
    let state = AsyncTranslationState {
        is_loading,
        current_task,
        error_message,
        submit_task: set_submit_task,
        task_id,
        cancel_task: set_cancel_task,
    };
    
    provide_context(state);
    
    children()
}


/// 启动任务状态轮询
fn start_task_polling(
    task_id: String,
    client: ApiClient,
    set_current_task: WriteSignal<Option<TranslationTask>>,
    set_is_loading: WriteSignal<bool>,
    set_error_message: WriteSignal<Option<String>>,
) {
    spawn_local(async move {
        loop {
            match client.get_task_status(&task_id).await {
                Ok(task) => {
                    set_current_task.set(Some(task.clone()));
                    
                    match task.status {
                        TaskStatus::Completed => {
                            set_is_loading.set(false);
                            web_sys::console::log_1(&format!("翻译任务完成: {}", task_id).into());
                            break;
                        }
                        TaskStatus::Failed => {
                            set_is_loading.set(false);
                            set_error_message.set(Some(
                                task.error_message.unwrap_or_else(|| "翻译任务失败".to_string())
                            ));
                            break;
                        }
                        TaskStatus::Pending | TaskStatus::Processing | TaskStatus::Retrying => {
                            // 继续轮询，延迟2秒
                            gloo_timers::future::TimeoutFuture::new(2000).await;
                        }
                    }
                }
                Err(error) => {
                    set_is_loading.set(false);
                    set_error_message.set(Some(format!("获取任务状态失败: {}", error)));
                    break;
                }
            }
        }
    });
}

/// 使用异步翻译Hook
pub fn use_async_translation() -> AsyncTranslationState {
    expect_context::<AsyncTranslationState>()
}