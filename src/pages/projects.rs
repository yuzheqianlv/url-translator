//! 项目管理页面
//! 
//! 提供翻译项目的创建、编辑、删除和管理功能

use leptos::*;
use crate::hooks::use_auth::{use_auth, is_authenticated};
use crate::services::api_client::{CreateProjectRequest, UpdateProjectRequest, Project};
use crate::components::AuthModal;

#[component]
pub fn ProjectsPage() -> impl IntoView {
    let auth = use_auth();
    
    // 项目列表状态
    let (projects, set_projects) = create_signal(Vec::<Project>::new());
    let (loading, set_loading) = create_signal(false);
    let (error_message, set_error_message) = create_signal(Option::<String>::None);
    
    // 新建项目模态框状态
    let (show_create_modal, set_show_create_modal) = create_signal(false);
    let (show_edit_modal, set_show_edit_modal) = create_signal(false);
    let (editing_project, set_editing_project) = create_signal(Option::<Project>::None);
    
    // 表单状态
    let (project_name, set_project_name) = create_signal(String::new());
    let (project_description, set_project_description) = create_signal(String::new());
    let (source_language, set_source_language) = create_signal("auto".to_string());
    let (target_language, set_target_language) = create_signal("zh".to_string());
    
    // 认证模态框状态
    let (show_auth_modal, set_show_auth_modal) = create_signal(false);

    // 重置表单
    let reset_form = move || {
        set_project_name.set(String::new());
        set_project_description.set(String::new());
        set_source_language.set("auto".to_string());
        set_target_language.set("zh".to_string());
        set_error_message.set(None);
    };

    // 加载项目列表
    let load_projects = move || {
        if !is_authenticated(&auth.auth_status.get()) {
            return;
        }
        
        set_loading.set(true);
        set_error_message.set(None);
        
        spawn_local(async move {
            let api_client = auth.api_client.get();
            match api_client.get_projects(None, None).await {
                Ok(project_list) => {
                    set_projects.set(project_list.projects);
                }
                Err(e) => {
                    set_error_message.set(Some(format!("加载项目失败: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // 创建项目
    let create_project = move || {
        let name = project_name.get().trim().to_string();
        let description = project_description.get().trim().to_string();
        
        if name.is_empty() {
            set_error_message.set(Some("项目名称不能为空".to_string()));
            return;
        }
        
        set_loading.set(true);
        set_error_message.set(None);
        
        let request = CreateProjectRequest {
            name,
            description: if description.is_empty() { None } else { Some(description) },
            source_language: source_language.get(),
            target_language: target_language.get(),
        };
        
        spawn_local(async move {
            let api_client = auth.api_client.get();
            match api_client.create_project(request).await {
                Ok(_) => {
                    set_show_create_modal.set(false);
                    reset_form();
                    load_projects();
                }
                Err(e) => {
                    set_error_message.set(Some(format!("创建项目失败: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // 更新项目
    let update_project = move || {
        let Some(project) = editing_project.get() else {
            return;
        };
        
        let name = project_name.get().trim().to_string();
        let description = project_description.get().trim().to_string();
        
        if name.is_empty() {
            set_error_message.set(Some("项目名称不能为空".to_string()));
            return;
        }
        
        set_loading.set(true);
        set_error_message.set(None);
        
        let request = UpdateProjectRequest {
            name,
            description: if description.is_empty() { None } else { Some(description) },
            source_language: source_language.get(),
            target_language: target_language.get(),
        };
        
        let project_id = project.id;
        spawn_local(async move {
            let api_client = auth.api_client.get();
            match api_client.update_project(project_id, request).await {
                Ok(_) => {
                    set_show_edit_modal.set(false);
                    set_editing_project.set(None);
                    reset_form();
                    load_projects();
                }
                Err(e) => {
                    set_error_message.set(Some(format!("更新项目失败: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // 删除项目
    let delete_project = move |project_id: i64| {
        set_loading.set(true);
        set_error_message.set(None);
        
        spawn_local(async move {
            let api_client = auth.api_client.get();
            match api_client.delete_project(project_id).await {
                Ok(_) => {
                    load_projects();
                }
                Err(e) => {
                    set_error_message.set(Some(format!("删除项目失败: {}", e)));
                }
            }
            set_loading.set(false);
        });
    };

    // 打开编辑模态框
    let open_edit_modal = move |project: Project| {
        set_project_name.set(project.name.clone());
        set_project_description.set(project.description.clone().unwrap_or_default());
        set_source_language.set(project.source_language.clone());
        set_target_language.set(project.target_language.clone());
        set_editing_project.set(Some(project));
        set_show_edit_modal.set(true);
    };

    // 初始加载
    create_effect(move |_| {
        if is_authenticated(&auth.auth_status.get()) {
            load_projects();
        }
    });

    view! {
        <div class="max-w-6xl mx-auto">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-3xl font-bold text-gray-900">"项目管理"</h1>
                
                {move || {
                    if is_authenticated(&auth.auth_status.get()) {
                        view! {
                            <button
                                class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors"
                                on:click=move |_| {
                                    reset_form();
                                    set_show_create_modal.set(true);
                                }
                            >
                                "新建项目"
                            </button>
                        }
                    } else {
                        view! {
                            <button
                                class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors"
                                on:click=move |_| set_show_auth_modal.set(true)
                            >
                                "登录以管理项目"
                            </button>
                        }
                    }
                }}
            </div>

            // 错误消息
            {move || {
                if let Some(error) = error_message.get() {
                    view! {
                        <div class="bg-red-50 border border-red-200 rounded-md p-4 mb-4">
                            <div class="text-red-800">{error}</div>
                        </div>
                    }
                } else {
                    view! { <div></div> }
                }
            }}

            // 主内容区域
            {move || {
                if !is_authenticated(&auth.auth_status.get()) {
                    view! {
                        <div class="text-center py-12">
                            <div class="text-gray-500 mb-4">"请登录以查看和管理您的翻译项目"</div>
                            <button
                                class="bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 transition-colors"
                                on:click=move |_| set_show_auth_modal.set(true)
                            >
                                "立即登录"
                            </button>
                        </div>
                    }
                } else if loading.get() {
                    view! {
                        <div class="text-center py-12">
                            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600 mx-auto mb-4"></div>
                            <div class="text-gray-600">"加载中..."</div>
                        </div>
                    }
                } else if projects.get().is_empty() {
                    view! {
                        <div class="text-center py-12">
                            <div class="text-gray-500 mb-4">"您还没有创建任何项目"</div>
                            <button
                                class="bg-blue-600 text-white px-6 py-3 rounded-md hover:bg-blue-700 transition-colors"
                                on:click=move |_| {
                                    reset_form();
                                    set_show_create_modal.set(true);
                                }
                            >
                                "创建第一个项目"
                            </button>
                        </div>
                    }
                } else {
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                            {move || projects.get().into_iter().map(|project| {
                                let project_clone = project.clone();
                                let edit_project = project.clone();
                                view! {
                                    <div class="bg-white rounded-lg shadow-md p-6 border hover:shadow-lg transition-shadow">
                                        <div class="flex justify-between items-start mb-4">
                                            <h3 class="text-lg font-semibold text-gray-900 truncate">
                                                {project.name}
                                            </h3>
                                            <div class="flex space-x-2">
                                                <button
                                                    class="text-gray-400 hover:text-blue-600 transition-colors"
                                                    on:click={
                                                        let edit_project = edit_project.clone();
                                                        move |_| open_edit_modal(edit_project.clone())
                                                    }
                                                    title="编辑项目"
                                                >
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"></path>
                                                    </svg>
                                                </button>
                                                <button
                                                    class="text-gray-400 hover:text-red-600 transition-colors"
                                                    on:click={
                                                        let project_id = project.id;
                                                        move |_| {
                                                            if web_sys::window()
                                                                .unwrap()
                                                                .confirm_with_message("确定要删除这个项目吗？此操作无法撤销。")
                                                                .unwrap()
                                                            {
                                                                delete_project(project_id);
                                                            }
                                                        }
                                                    }
                                                    title="删除项目"
                                                >
                                                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                                                    </svg>
                                                </button>
                                            </div>
                                        </div>

                                        <div class="space-y-2 text-sm text-gray-600">
                                            {project.description.map(|desc| view! {
                                                <p class="line-clamp-2">{desc}</p>
                                            })}
                                            <div class="flex justify-between">
                                                <span>
                                                    {format!("{} → {}", 
                                                        if project.source_language == "auto" { "自动检测".to_string() } else { project.source_language },
                                                        match project.target_language.as_str() {
                                                            "zh" => "中文",
                                                            "en" => "英文",
                                                            "ja" => "日文",
                                                            "ko" => "韩文",
                                                            "fr" => "法文",
                                                            "de" => "德文",
                                                            "es" => "西班牙文",
                                                            "it" => "意大利文",
                                                            "ru" => "俄文",
                                                            _ => &project.target_language,
                                                        }
                                                    )}
                                                </span>
                                            </div>
                                            <div class="text-xs text-gray-400">
                                                {"创建于 "}{project.created_at.format("%Y-%m-%d %H:%M").to_string()}
                                            </div>
                                        </div>

                                        <div class="mt-4 pt-4 border-t">
                                            <button
                                                class="w-full bg-blue-50 text-blue-600 py-2 px-4 rounded-md hover:bg-blue-100 transition-colors"
                                                on:click={
                                                    let _project_id = project_clone.id;
                                                    move |_| {
                                                        // 这里可以导航到项目详情页面或开始翻译
                                                        // TODO: 实现项目详情页面导航
                                                    }
                                                }
                                            >
                                                "打开项目"
                                            </button>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }
            }}
        </div>

        // 创建项目模态框
        <ProjectModal
            show=show_create_modal
            on_close=set_show_create_modal
            title="创建新项目"
            project_name=project_name
            set_project_name=set_project_name
            project_description=project_description
            set_project_description=set_project_description
            source_language=source_language
            set_source_language=set_source_language
            target_language=target_language
            set_target_language=set_target_language
            on_submit=create_project
            is_loading=loading
        />

        // 编辑项目模态框
        <ProjectModal
            show=show_edit_modal
            on_close=set_show_edit_modal
            title="编辑项目"
            project_name=project_name
            set_project_name=set_project_name
            project_description=project_description
            set_project_description=set_project_description
            source_language=source_language
            set_source_language=set_source_language
            target_language=target_language
            set_target_language=set_target_language
            on_submit=update_project
            is_loading=loading
        />

        // 认证模态框
        <AuthModal 
            show=show_auth_modal
            on_close=set_show_auth_modal
        />
    }
}

#[component]
fn ProjectModal(
    #[prop(into)] show: Signal<bool>,
    on_close: WriteSignal<bool>,
    title: &'static str,
    project_name: ReadSignal<String>,
    set_project_name: WriteSignal<String>,
    project_description: ReadSignal<String>,
    set_project_description: WriteSignal<String>,
    source_language: ReadSignal<String>,
    set_source_language: WriteSignal<String>,
    target_language: ReadSignal<String>,
    set_target_language: WriteSignal<String>,
    on_submit: impl Fn() + 'static,
    #[prop(into)] is_loading: Signal<bool>,
) -> impl IntoView {
    let submit = move || on_submit();
    
    view! {
        <div
            class:hidden={move || !show.get()}
            class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
            on:click=move |_| on_close.set(false)
        >
            <div
                class="bg-white rounded-lg shadow-xl max-w-md w-full m-4 p-6"
                on:click=|e| e.stop_propagation()
            >
                <div class="flex justify-between items-center mb-6">
                    <h2 class="text-2xl font-bold text-gray-900">{title}</h2>
                    <button
                        class="text-gray-400 hover:text-gray-600 text-2xl"
                        on:click=move |_| on_close.set(false)
                    >
                        "×"
                    </button>
                </div>

                <form class="space-y-4" on:submit=move |e| {
                    e.prevent_default();
                    submit();
                }>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "项目名称"
                        </label>
                        <input
                            type="text"
                            placeholder="请输入项目名称"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value={move || project_name.get()}
                            on:input=move |e| set_project_name.set(event_target_value(&e))
                        />
                    </div>

                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-2">
                            "项目描述（可选）"
                        </label>
                        <textarea
                            placeholder="请输入项目描述"
                            rows="3"
                            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                            prop:value={move || project_description.get()}
                            on:input=move |e| set_project_description.set(event_target_value(&e))
                        ></textarea>
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "源语言"
                            </label>
                            <select
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                prop:value={move || source_language.get()}
                                on:change=move |e| set_source_language.set(event_target_value(&e))
                            >
                                <option value="auto">"自动检测"</option>
                                <option value="en">"英文"</option>
                                <option value="zh">"中文"</option>
                                <option value="ja">"日文"</option>
                                <option value="ko">"韩文"</option>
                                <option value="fr">"法文"</option>
                                <option value="de">"德文"</option>
                                <option value="es">"西班牙文"</option>
                                <option value="it">"意大利文"</option>
                                <option value="ru">"俄文"</option>
                            </select>
                        </div>

                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-2">
                                "目标语言"
                            </label>
                            <select
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                                prop:value={move || target_language.get()}
                                on:change=move |e| set_target_language.set(event_target_value(&e))
                            >
                                <option value="zh">"中文"</option>
                                <option value="en">"英文"</option>
                                <option value="ja">"日文"</option>
                                <option value="ko">"韩文"</option>
                                <option value="fr">"法文"</option>
                                <option value="de">"德文"</option>
                                <option value="es">"西班牙文"</option>
                                <option value="it">"意大利文"</option>
                                <option value="ru">"俄文"</option>
                            </select>
                        </div>
                    </div>

                    <button
                        type="submit"
                        disabled={move || is_loading.get()}
                        class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                        {move || if is_loading.get() {
                            "处理中..."
                        } else {
                            title.split_whitespace().last().unwrap_or("确定")
                        }}
                    </button>
                </form>
            </div>
        </div>
    }
}