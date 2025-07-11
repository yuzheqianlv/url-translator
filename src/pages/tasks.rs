//! 翻译任务管理页面
//! 
//! 显示用户的所有翻译任务历史和状态

use leptos::*;
use crate::components::AuthRequired;
use crate::hooks::{use_auth, AuthStatus};
use crate::services::api_client::{ApiClient, TranslationTask, TaskStatus};
use crate::theme::use_theme_context;
use crate::config::EnvConfig;
use wasm_bindgen_futures::spawn_local;
use chrono::{DateTime, Utc};

#[component]
pub fn TasksPage() -> impl IntoView {
    let auth = use_auth();
    let theme_context = use_theme_context();
    let (tasks, set_tasks) = create_signal(Vec::<TranslationTask>::new());
    let (is_loading, set_is_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(None::<String>);
    let (selected_task_id, set_selected_task_id) = create_signal(None::<String>);
    let (search_term, set_search_term) = create_signal(String::new());
    let (status_filter, set_status_filter) = create_signal(None::<TaskStatus>);

    // 创建API客户端
    let api_client = create_memo(move |_| {
        let config = EnvConfig::global();
        let mut client = ApiClient::new(config.into());
        
        if let AuthStatus::Authenticated(user) = auth.auth_status.get() {
            client.set_auth_token(Some(user.access_token.clone()));
        }
        
        client
    });

    // 加载任务列表
    let load_tasks = move || {
        set_is_loading.set(true);
        set_error_message.set(None);
        
        let client = api_client.get();
        
        spawn_local(async move {
            match client.get_user_tasks().await {
                Ok(task_list) => {
                    set_tasks.set(task_list);
                    set_is_loading.set(false);
                }
                Err(error) => {
                    set_error_message.set(Some(error));
                    set_is_loading.set(false);
                }
            }
        });
    };

    // 页面加载时获取任务列表
    create_effect(move |_| {
        if matches!(auth.auth_status.get(), AuthStatus::Authenticated(_)) {
            load_tasks();
        }
    });

    // 过滤任务
    let filtered_tasks = create_memo(move |_| {
        let mut filtered = tasks.get();
        
        // 状态过滤
        if let Some(status) = status_filter.get() {
            filtered.retain(|task| task.status == status);
        }
        
        // 搜索过滤
        let search = search_term.get().to_lowercase();
        if !search.is_empty() {
            filtered.retain(|task| {
                task.url.to_lowercase().contains(&search) ||
                task.id.to_lowercase().contains(&search)
            });
        }
        
        // 按创建时间降序排序
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        filtered
    });

    // 统计信息
    let task_stats = create_memo(move |_| {
        let tasks = tasks.get();
        let total = tasks.len();
        let completed = tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = tasks.iter().filter(|t| t.status == TaskStatus::Failed).count();
        let processing = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Processing | TaskStatus::Retrying)).count();
        
        (total, completed, failed, processing)
    });

    // 取消任务函数
    let cancel_task = move |task_id: String| {
        let client = api_client.get();
        
        spawn_local(async move {
            match client.cancel_translation_task(&task_id).await {
                Ok(()) => {
                    // 取消成功，重新加载任务列表
                    load_tasks();
                    web_sys::console::log_1(&format!("任务 {} 已取消", task_id).into());
                }
                Err(error) => {
                    set_error_message.set(Some(format!("取消任务失败: {}", error)));
                }
            }
        });
    };

    view! {
        <div class="max-w-6xl mx-auto space-y-6">
            <div class="flex justify-between items-center">
                <h1 class="text-3xl font-bold" style=move || theme_context.get().theme.text_style()>
                    "翻译任务管理"
                </h1>

                <div class="flex space-x-2">
                    <button
                        class="px-4 py-2 rounded-md transition-colors hover:opacity-90"
                        style=move || theme_context.get().theme.button_primary_style()
                        on:click=move |_| load_tasks()
                        disabled=is_loading
                    >
                        {move || if is_loading.get() { "加载中..." } else { "刷新" }}
                    </button>
                </div>
            </div>

            <AuthRequired message="请先登录后查看您的翻译任务".to_string()>
                // 统计信息卡片
                <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
                    {move || {
                        let (total, completed, failed, processing) = task_stats.get();
                        let theme = theme_context.get().theme;
                        view! {
                            <div class="rounded-lg p-4" style=theme.card_style()>
                                <div class="text-2xl font-bold" style=theme.text_style()>{total}</div>
                                <div class="text-sm" style=theme.subtext_style()>"总任务数"</div>
                            </div>
                            <div class="rounded-lg p-4" style=theme.card_style()>
                                <div class="text-2xl font-bold text-green-600">{completed}</div>
                                <div class="text-sm" style=theme.subtext_style()>"已完成"</div>
                            </div>
                            <div class="rounded-lg p-4" style=theme.card_style()>
                                <div class="text-2xl font-bold text-red-600">{failed}</div>
                                <div class="text-sm" style=theme.subtext_style()>"失败"</div>
                            </div>
                            <div class="rounded-lg p-4" style=theme.card_style()>
                                <div class="text-2xl font-bold text-blue-600">{processing}</div>
                                <div class="text-sm" style=theme.subtext_style()>"处理中"</div>
                            </div>
                        }
                    }}
                </div>

                // 搜索和过滤
                <div class="rounded-lg shadow-lg p-6" style=move || theme_context.get().theme.card_style()>
                    <div class="flex flex-col md:flex-row gap-4">
                        <div class="flex-1">
                            <input
                                type="text"
                                class="w-full px-4 py-2 rounded-md focus:ring-2 focus:border-transparent"
                                style=move || theme_context.get().theme.input_style()
                                placeholder="搜索任务ID或URL..."
                                prop:value=search_term
                                on:input=move |ev| {
                                    set_search_term.set(event_target_value(&ev));
                                }
                            />
                        </div>
                        <div class="flex space-x-2">
                            <select
                                class="px-4 py-2 rounded-md"
                                style=move || theme_context.get().theme.input_style()
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    let status = match value.as_str() {
                                        "pending" => Some(TaskStatus::Pending),
                                        "processing" => Some(TaskStatus::Processing),
                                        "completed" => Some(TaskStatus::Completed),
                                        "failed" => Some(TaskStatus::Failed),
                                        "retrying" => Some(TaskStatus::Retrying),
                                        _ => None,
                                    };
                                    set_status_filter.set(status);
                                }
                            >
                                <option value="">"全部状态"</option>
                                <option value="pending">"等待处理"</option>
                                <option value="processing">"正在翻译"</option>
                                <option value="completed">"翻译完成"</option>
                                <option value="failed">"翻译失败"</option>
                                <option value="retrying">"正在重试"</option>
                            </select>
                        </div>
                    </div>
                </div>

                // 任务列表
                <div class="rounded-lg shadow-lg" style=move || theme_context.get().theme.card_style()>
                    <Show
                        when=move || error_message.get().is_some()
                        fallback=|| view! { <div></div> }
                    >
                        <div class="p-4 border-b" style=move || format!("background-color: {}; border-color: {}; color: {};", theme_context.get().theme.surface1, theme_context.get().theme.red, theme_context.get().theme.red)>
                            <p class="text-sm">
                                {move || error_message.get().unwrap_or_default()}
                            </p>
                        </div>
                    </Show>

                    <Show
                        when=move || !is_loading.get()
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
                            when=move || !filtered_tasks.get().is_empty()
                            fallback=move || {
                                let theme = theme_context.get().theme;
                                view! {
                                    <div class="p-8 text-center" style=theme.subtext_style()>
                                        <svg class="w-16 h-16 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=theme.muted_text_style()>
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v10a2 2 0 002 2h8a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"></path>
                                        </svg>
                                        <p class="text-lg font-medium">"暂无翻译任务"</p>
                                        <p class="text-sm">"开始您的第一个翻译任务吧！"</p>
                                    </div>
                                }
                            }
                        >
                            <TasksList 
                                tasks=filtered_tasks 
                                selected_task_id=selected_task_id
                                set_selected_task_id=set_selected_task_id
                                cancel_task=cancel_task
                            />
                        </Show>
                    </Show>
                </div>
            </AuthRequired>
        </div>
    }
}

#[component]
fn TasksList(
    tasks: ReadSignal<Vec<TranslationTask>>,
    selected_task_id: ReadSignal<Option<String>>,
    set_selected_task_id: WriteSignal<Option<String>>,
    cancel_task: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let theme_context = use_theme_context();

    view! {
        <div class="divide-y" style=move || format!("border-color: {};", theme_context.get().theme.surface2)>
            <For
                each=move || tasks.get()
                key=|task| task.id.clone()
                children=move |task| {
                    let task_id = task.id.clone();
                    let task_id_for_expand = task_id.clone();
                    
                    let is_expanded = create_memo(move |_| {
                        selected_task_id.get() == Some(task_id.clone())
                    });
                    
                    let toggle_expand = move |_| {
                        if is_expanded.get() {
                            set_selected_task_id.set(None);
                        } else {
                            set_selected_task_id.set(Some(task_id_for_expand.clone()));
                        }
                    };
                    
                    view! {
                        <TaskItem 
                            task=task 
                            is_expanded=is_expanded
                            on_toggle_expand=toggle_expand
                            cancel_task=cancel_task.clone()
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn TaskItem(
    task: TranslationTask,
    is_expanded: Memo<bool>,
    on_toggle_expand: impl Fn(leptos::ev::MouseEvent) + 'static,
    cancel_task: impl Fn(String) + 'static + Clone,
) -> impl IntoView {
    let theme_context = use_theme_context();

    let status_color = match task.status {
        TaskStatus::Completed => theme_context.get().theme.green,
        TaskStatus::Failed => theme_context.get().theme.red,
        TaskStatus::Processing | TaskStatus::Retrying => theme_context.get().theme.blue,
        TaskStatus::Pending => theme_context.get().theme.yellow,
    };

    let status_text = match task.status {
        TaskStatus::Pending => "等待处理",
        TaskStatus::Processing => "正在翻译",
        TaskStatus::Completed => "翻译完成",
        TaskStatus::Failed => "翻译失败",
        TaskStatus::Retrying => "正在重试",
    };

    view! {
        <div class="p-4 transition-all hover:shadow-md" style=move || format!("background-color: {};", theme_context.get().theme.base)>
            <div class="flex items-center justify-between cursor-pointer" on:click=on_toggle_expand>
                <div class="flex-1">
                    <div class="flex items-center space-x-3 mb-2">
                        <div class="flex items-center space-x-2">
                            <div class="w-3 h-3 rounded-full" style=format!("background-color: {};", status_color)></div>
                            <span class="text-sm font-medium" style=move || theme_context.get().theme.text_style()>
                                {status_text}
                            </span>
                        </div>
                        <span class="text-xs px-2 py-1 rounded-full" style=format!("background-color: {}; color: {};", theme_context.get().theme.surface1, theme_context.get().theme.subtext1)>
                            {format!("{} → {}", task.source_lang, task.target_lang)}
                        </span>
                        <span class="text-xs" style=move || theme_context.get().theme.subtext_style()>
                            {task.created_at.clone()}
                        </span>
                    </div>
                    
                    <div class="text-sm" style=move || theme_context.get().theme.subtext_style()>
                        <span class="font-medium">ID:</span> {format!("{}...", &task.id[..8.min(task.id.len())])}
                    </div>
                    <div class="text-sm truncate mt-1" style=move || theme_context.get().theme.subtext_style()>
                        {task.url.clone()}
                    </div>
                </div>
                
                <div class="flex items-center space-x-2 ml-4">
                    <button
                        class="p-2 rounded-md transition-colors hover:opacity-80"
                        style=move || theme_context.get().theme.button_secondary_style()
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
                </div>
            </div>

            <Show
                when=move || task.status == TaskStatus::Processing || task.status == TaskStatus::Retrying
                fallback=|| view! { <div></div> }
            >
                <div class="mt-3 mb-3">
                    <div class="flex items-center justify-between mb-1">
                        <span class="text-xs" style=move || theme_context.get().theme.subtext_style()>
                            "进度"
                        </span>
                        <span class="text-xs" style=move || theme_context.get().theme.subtext_style()>
                            {format!("{:.0}%", task.progress * 100.0)}
                        </span>
                    </div>
                    <div class="w-full bg-gray-200 rounded-full h-2" style=move || format!("background-color: {};", theme_context.get().theme.surface2)>
                        <div 
                            class="h-2 rounded-full transition-all duration-300" 
                            style=format!(
                                "width: {}%; background-color: {};", 
                                task.progress * 100.0,
                                theme_context.get().theme.blue
                            )
                        ></div>
                    </div>
                    <p class="text-xs mt-1" style=move || theme_context.get().theme.subtext_style()>
                        {task.progress_message.clone()}
                    </p>
                </div>
            </Show>

            <Show
                when=move || is_expanded.get()
                fallback=|| view! { <div></div> }
            >
                <div class="border-t pt-3 mt-3 space-y-2" style=move || format!("border-color: {};", theme_context.get().theme.surface2)>
                    <div class="grid grid-cols-2 gap-4 text-xs">
                        <div>
                            <span class="font-medium" style=move || theme_context.get().theme.text_style()>"任务ID:"</span>
                            <p class="break-all" style=move || theme_context.get().theme.subtext_style()>{task.id.clone()}</p>
                        </div>
                        <div>
                            <span class="font-medium" style=move || theme_context.get().theme.text_style()>"创建时间:"</span>
                            <p style=move || theme_context.get().theme.subtext_style()>{task.created_at.clone()}</p>
                        </div>
                        {move || {
                            if let Some(started_at) = &task.started_at {
                                view! {
                                    <div>
                                        <span class="font-medium" style=move || theme_context.get().theme.text_style()>"开始时间:"</span>
                                        <p style=move || theme_context.get().theme.subtext_style()>{started_at.clone()}</p>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }}
                        {move || {
                            if let Some(completed_at) = &task.completed_at {
                                view! {
                                    <div>
                                        <span class="font-medium" style=move || theme_context.get().theme.text_style()>"完成时间:"</span>
                                        <p style=move || theme_context.get().theme.subtext_style()>{completed_at.clone()}</p>
                                    </div>
                                }.into_view()
                            } else {
                                view! { <div></div> }.into_view()
                            }
                        }}
                    </div>
                    
                    <Show
                        when=move || task.retry_count > 0
                        fallback=|| view! { <div></div> }
                    >
                        <div class="text-xs">
                            <span class="font-medium" style=move || theme_context.get().theme.text_style()>"重试信息:"</span>
                            <p style=move || format!("color: {};", theme_context.get().theme.warning_color())>
                                {format!("已重试 {} 次，最大重试次数: {}", task.retry_count, task.max_retries)}
                            </p>
                        </div>
                    </Show>
                    
                    <Show
                        when=move || task.error_message.is_some()
                        fallback=|| view! { <div></div> }
                    >
                        <div class="text-xs">
                            <span class="font-medium" style=move || format!("color: {};", theme_context.get().theme.red)>"错误信息:"</span>
                            <p style=move || format!("color: {};", theme_context.get().theme.red)>
                                {task.error_message.clone().unwrap_or_default()}
                            </p>
                        </div>
                    </Show>

                    // 取消按钮（只在可取消状态下显示）
                    <Show
                        when=move || matches!(task.status, TaskStatus::Pending | TaskStatus::Processing | TaskStatus::Retrying)
                        fallback=|| view! { <div></div> }
                    >
                        <div class="flex space-x-2 pt-2">
                            <button
                                class="px-3 py-1 text-xs rounded transition-colors hover:opacity-90"
                                style=move || format!(
                                    "background-color: {}; color: {}; border: 1px solid {};",
                                    theme_context.get().theme.red,
                                    theme_context.get().theme.base,
                                    theme_context.get().theme.red
                                )
                                on:click={
                                    let task_id = task.id.clone();
                                    let cancel_fn = cancel_task.clone();
                                    move |_| {
                                        if web_sys::window()
                                            .and_then(|w| w.confirm_with_message("确定要取消这个翻译任务吗？").ok())
                                            .unwrap_or(false)
                                        {
                                            cancel_fn(task_id.clone());
                                        }
                                    }
                                }
                            >
                                "取消任务"
                            </button>
                        </div>
                    </Show>

                    <Show
                        when=move || task.status == TaskStatus::Completed
                        fallback=|| view! { <div></div> }
                    >
                        <div class="flex space-x-2 pt-2">
                            <button
                                class="px-3 py-1 text-xs rounded transition-colors hover:opacity-90"
                                style=move || theme_context.get().theme.button_primary_style()
                                on:click=move |_| {
                                    // TODO: 跳转到翻译结果页面
                                    web_sys::console::log_1(&format!("查看翻译结果: {}", task.id).into());
                                }
                            >
                                "查看结果"
                            </button>
                            <button
                                class="px-3 py-1 text-xs rounded transition-colors hover:opacity-90"
                                style=move || theme_context.get().theme.button_secondary_style()
                                on:click=move |_| {
                                    // TODO: 下载翻译结果
                                    web_sys::console::log_1(&format!("下载翻译结果: {}", task.id).into());
                                }
                            >
                                "下载"
                            </button>
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}