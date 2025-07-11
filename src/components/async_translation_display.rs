//! 异步翻译显示组件
//! 
//! 显示异步翻译任务的状态和结果

use leptos::*;
use crate::hooks::use_async_translation;
use crate::services::api_client::TaskStatus;
use crate::theme::use_theme_context;

#[component]
pub fn AsyncTranslationDisplay() -> impl IntoView {
    let async_translation = use_async_translation();
    let theme_context = use_theme_context();

    view! {
        <Show
            when=move || async_translation.current_task.get().is_some() || async_translation.is_loading.get()
            fallback=|| view! { <div></div> }
        >
            <div class="rounded-lg shadow-lg p-6 mt-4" style=move || theme_context.get().theme.card_style()>
                <h3 class="text-lg font-semibold mb-4" style=move || theme_context.get().theme.text_style()>
                    "翻译任务状态"
                </h3>

                <Show
                    when=move || async_translation.is_loading.get()
                    fallback=|| view! { <div></div> }
                >
                    <TaskStatusDisplay />
                </Show>

                <Show
                    when=move || async_translation.error_message.get().is_some()
                    fallback=|| view! { <div></div> }
                >
                    <div class="mt-4 p-4 rounded-lg" style=move || format!("background-color: {}; border: 1px solid {};", theme_context.get().theme.surface1, theme_context.get().theme.red)>
                        <div class="flex items-center">
                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.red)>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                            </svg>
                            <span class="text-sm font-medium" style=move || format!("color: {};", theme_context.get().theme.red)>
                                "错误"
                            </span>
                        </div>
                        <p class="mt-2 text-sm" style=move || theme_context.get().theme.text_style()>
                            {move || async_translation.error_message.get().unwrap_or_default()}
                        </p>
                    </div>
                </Show>

                <Show
                    when=move || {
                        if let Some(task) = async_translation.current_task.get() {
                            task.status == TaskStatus::Completed
                        } else {
                            false
                        }
                    }
                    fallback=|| view! { <div></div> }
                >
                    <CompletedTaskDisplay />
                </Show>
            </div>
        </Show>
    }
}

#[component]
fn TaskStatusDisplay() -> impl IntoView {
    let async_translation = use_async_translation();
    let theme_context = use_theme_context();

    view! {
        <div class="space-y-4">
            {move || {
                if let Some(task) = async_translation.current_task.get() {
                    let status_text = get_status_text(&task.status);
                    let progress = task.progress;
                    let progress_message = task.progress_message;
                    let retry_count = task.retry_count;
                    let max_retries = task.max_retries;
                    let task_id = task.id.clone();
                    let task_status = task.status.clone();
                    
                    view! {
                        <div class="space-y-3">
                            <div class="flex items-center justify-between">
                                <div class="flex items-center space-x-3">
                                    <span class="text-sm font-medium" style=move || theme_context.get().theme.text_style()>
                                        {status_text}
                                    </span>
                                    <span class="text-sm" style=move || theme_context.get().theme.subtext_style()>
                                        {format!("{:.0}%", progress * 100.0)}
                                    </span>
                                </div>
                                
                                // 取消按钮（只在任务可取消时显示）
                                {move || {
                                    match task_status {
                                        TaskStatus::Pending | TaskStatus::Processing | TaskStatus::Retrying => {
                                            let task_id_for_cancel = task_id.clone();
                                            view! {
                                                <button
                                                    class="px-3 py-1 text-xs rounded transition-colors hover:opacity-90"
                                                    style=move || format!(
                                                        "background-color: {}; color: {}; border: 1px solid {};",
                                                        theme_context.get().theme.red,
                                                        theme_context.get().theme.base,
                                                        theme_context.get().theme.red
                                                    )
                                                    on:click=move |_| {
                                                        async_translation.cancel_task.set(Some(task_id_for_cancel.clone()));
                                                    }
                                                >
                                                    "取消任务"
                                                </button>
                                            }.into_view()
                                        }
                                        _ => view! { <div></div> }.into_view()
                                    }
                                }}
                            </div>
                            
                            <div class="w-full bg-gray-200 rounded-full h-2" style=move || format!("background-color: {};", theme_context.get().theme.surface2)>
                                <div 
                                    class="h-2 rounded-full transition-all duration-300" 
                                    style=format!(
                                        "width: {}%; background-color: {};", 
                                        progress * 100.0,
                                        theme_context.get().theme.blue
                                    )
                                ></div>
                            </div>
                            
                            <p class="text-sm" style=move || theme_context.get().theme.subtext_style()>
                                {progress_message}
                            </p>
                            
                            {move || {
                                if retry_count > 0 {
                                    view! {
                                        <p class="text-xs" style=move || format!("color: {};", theme_context.get().theme.warning_color())>
                                            {format!("重试次数: {}/{}", retry_count, max_retries)}
                                        </p>
                                    }.into_view()
                                } else {
                                    view! { <div></div> }.into_view()
                                }
                            }}
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="flex items-center space-x-3">
                            <div class="animate-spin rounded-full h-6 w-6 border-b-2" style=move || format!("border-color: {};", theme_context.get().theme.blue)></div>
                            <span class="text-sm" style=move || theme_context.get().theme.subtext_style()>
                                "正在提交翻译任务..."
                            </span>
                        </div>
                    }.into_view()
                }
            }}
        </div>
    }
}

#[component]
fn CompletedTaskDisplay() -> impl IntoView {
    let async_translation = use_async_translation();
    let theme_context = use_theme_context();

    view! {
        <div class="mt-4 p-4 rounded-lg" style=move || format!("background-color: {}; border: 1px solid {};", theme_context.get().theme.surface1, theme_context.get().theme.green)>
            <div class="flex items-center mb-3">
                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.green)>
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                </svg>
                <span class="text-sm font-medium" style=move || format!("color: {};", theme_context.get().theme.green)>
                    "翻译完成"
                </span>
            </div>
            
            {move || {
                if let Some(task) = async_translation.current_task.get() {
                    view! {
                        <div class="space-y-2">
                            <p class="text-sm" style=move || theme_context.get().theme.text_style()>
                                "URL: " {task.url}
                            </p>
                            <p class="text-sm" style=move || theme_context.get().theme.subtext_style()>
                                {format!("翻译方向: {} → {}", task.source_lang, task.target_lang)}
                            </p>
                            <button 
                                class="mt-3 px-4 py-2 rounded-md text-sm transition-colors hover:opacity-90"
                                style=move || theme_context.get().theme.button_primary_style()
                                on:click=move |_| {
                                    // TODO: 跳转到翻译结果页面或显示结果
                                    web_sys::console::log_1(&"查看翻译结果".into());
                                }
                            >
                                "查看翻译结果"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! { <div></div> }.into_view()
                }
            }}
        </div>
    }
}

fn get_status_text(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Pending => "等待处理",
        TaskStatus::Processing => "正在翻译",
        TaskStatus::Completed => "翻译完成",
        TaskStatus::Failed => "翻译失败",
        TaskStatus::Retrying => "正在重试",
    }
}