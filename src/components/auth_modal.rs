//! 认证模态框组件
//! 
//! 提供用户登录和注册功能的模态框界面

use leptos::*;
use leptos_router::*;
use crate::hooks::use_auth::{use_auth, AuthStatus};
use crate::services::api_client::{LoginRequest, RegisterRequest};
use crate::components::use_notifications;
use crate::theme::catppuccin::{ThemeVariant, CatppuccinTheme};
use gloo_timers::future::TimeoutFuture;

#[derive(Clone, Copy, PartialEq)]
pub enum AuthMode {
    Login,
    Register,
}

#[component]
pub fn AuthModal(
    /// 是否显示模态框
    #[prop(into)] show: Signal<bool>,
    /// 关闭模态框的回调
    on_close: WriteSignal<bool>,
) -> impl IntoView {
    let auth = use_auth();
    let notifications = use_notifications();
    let navigate = use_navigate();
    let theme = CatppuccinTheme::get_theme(&ThemeVariant::default());
    
    // 认证模式状态
    let (auth_mode, set_auth_mode) = create_signal(AuthMode::Login);
    
    // 表单状态
    let (email, set_email) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (username, set_username) = create_signal(String::new());
    let (confirm_password, set_confirm_password) = create_signal(String::new());
    
    // 表单验证状态
    let (form_errors, set_form_errors) = create_signal(Vec::<String>::new());

    // 重置表单
    let reset_form = move || {
        set_email.set(String::new());
        set_password.set(String::new());
        set_username.set(String::new());
        set_confirm_password.set(String::new());
        set_form_errors.set(Vec::new());
    };

    // 切换认证模式
    let toggle_mode = move || {
        match auth_mode.get() {
            AuthMode::Login => set_auth_mode.set(AuthMode::Register),
            AuthMode::Register => set_auth_mode.set(AuthMode::Login),
        }
        reset_form();
    };

    // 验证表单
    let validate_form = move |mode: AuthMode| -> Vec<String> {
        let mut errors = Vec::new();
        
        let email_val = email.get();
        let password_val = password.get();
        
        // 邮箱验证
        if email_val.is_empty() {
            errors.push("请输入邮箱地址".to_string());
        } else if !email_val.contains('@') {
            errors.push("请输入有效的邮箱地址".to_string());
        }
        
        // 密码验证
        if password_val.is_empty() {
            errors.push("请输入密码".to_string());
        } else if password_val.len() < 8 {
            errors.push("密码长度至少为8位".to_string());
        }
        
        // 注册模式的额外验证
        if mode == AuthMode::Register {
            let username_val = username.get();
            let confirm_password_val = confirm_password.get();
            
            if username_val.is_empty() {
                errors.push("请输入用户名".to_string());
            } else if username_val.len() < 3 {
                errors.push("用户名长度至少为3位".to_string());
            }
            
            if confirm_password_val != password_val {
                errors.push("两次输入的密码不一致".to_string());
            }
        }
        
        errors
    };

    // 提交表单
    let submit_form = move || {
        let mode = auth_mode.get();
        let errors = validate_form(mode);
        
        if !errors.is_empty() {
            set_form_errors.set(errors);
            return;
        }
        
        set_form_errors.set(Vec::new());
        
        match mode {
            AuthMode::Login => {
                let login_request = LoginRequest {
                    email: email.get(),
                    password: password.get(),
                };
                auth.login.set(Some(login_request));
            }
            AuthMode::Register => {
                let register_request = RegisterRequest {
                    username: username.get(),
                    email: email.get(),
                    password: password.get(),
                };
                auth.register.set(Some(register_request));
            }
        }
    };

    // 监听认证状态变化，成功后关闭模态框并显示通知
    create_effect({
        move |_| {
            match auth.auth_status.get() {
                AuthStatus::Authenticated(_) => {
                    let mode = auth_mode.get();
                    let message = match mode {
                        AuthMode::Login => "登录成功！",
                        AuthMode::Register => "注册成功，已自动登录！",
                    };
                    
                    // 显示成功通知
                    notifications.success(message);
                    
                    // 清理表单并关闭模态框
                    reset_form();
                    on_close.set(false);
                    
                    // 延迟跳转到项目页面
                    let navigate_clone = navigate.clone();
                    spawn_local(async move {
                        TimeoutFuture::new(1000).await;
                        navigate_clone("/projects", Default::default());
                    });
                }
                AuthStatus::Failed(ref error) => {
                    // 显示错误通知
                    notifications.error(format!("认证失败: {}", error));
                }
                _ => {}
            }
        }
    });

    // 键盘事件处理
    let handle_keydown = move |event: web_sys::KeyboardEvent| {
        if event.key() == "Enter" {
            event.prevent_default();
            submit_form();
        } else if event.key() == "Escape" {
            event.prevent_default();
            on_close.set(false);
        }
    };

    view! {
        <div
            class:hidden={move || !show.get()}
            class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
            on:click=move |_| on_close.set(false)
        >
            <div
                class="rounded-lg shadow-xl max-w-md w-full m-4 p-6"
                style=format!("{}; color: {};", theme.card_style(), theme.text)
                on:click=|e| e.stop_propagation()
                on:keydown=handle_keydown
            >
                // 标题
                <div class="flex justify-between items-center mb-6">
                    <h2 
                        class="text-2xl font-bold"
                        style=format!("color: {};", theme.text)
                    >
                        {move || match auth_mode.get() {
                            AuthMode::Login => "登录",
                            AuthMode::Register => "注册",
                        }}
                    </h2>
                    <button
                        class="text-2xl hover:opacity-75 transition-opacity"
                        style=format!("color: {};", theme.subtext1)
                        on:click=move |_| on_close.set(false)
                    >
                        "×"
                    </button>
                </div>

                // 错误显示
                <div class:hidden={move || form_errors.get().is_empty()}>
                    <div 
                        class="rounded-md p-3 mb-4"
                        style=format!(
                            "background-color: {}; border: 1px solid {}; color: {};",
                            theme.surface1, theme.error_color(), theme.error_color()
                        )
                    >
                        <div class="text-sm">
                            {move || form_errors.get().into_iter().map(|error| view! {
                                <div class="mb-1">{error}</div>
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                </div>

                // 认证状态错误显示已通过通知系统处理，不再需要额外显示

                // 表单
                <form class="space-y-4" on:submit=move |e| {
                    e.prevent_default();
                    submit_form();
                }>
                    // 用户名字段（仅注册时显示）
                    <div class:hidden={move || auth_mode.get() != AuthMode::Register}>
                        <label 
                            class="block text-sm font-medium mb-2"
                            style=format!("color: {};", theme.text)
                        >
                            "用户名"
                        </label>
                        <input
                            type="text"
                            placeholder="请输入用户名"
                            class="w-full px-3 py-2 rounded-md focus:outline-none focus:ring-2 transition-all"
                            style=format!(
                                "{}; focus:ring-color: {};",
                                theme.input_style(),
                                theme.blue
                            )
                            prop:value={move || username.get()}
                            on:input=move |e| set_username.set(event_target_value(&e))
                        />
                    </div>

                    // 邮箱字段
                    <div>
                        <label 
                            class="block text-sm font-medium mb-2"
                            style=format!("color: {};", theme.text)
                        >
                            "邮箱"
                        </label>
                        <input
                            type="email"
                            placeholder="请输入邮箱地址"
                            class="w-full px-3 py-2 rounded-md focus:outline-none focus:ring-2 transition-all"
                            style=format!(
                                "{}; focus:ring-color: {};",
                                theme.input_style(),
                                theme.blue
                            )
                            prop:value={move || email.get()}
                            on:input=move |e| set_email.set(event_target_value(&e))
                        />
                    </div>

                    // 密码字段
                    <div>
                        <label 
                            class="block text-sm font-medium mb-2"
                            style=format!("color: {};", theme.text)
                        >
                            "密码"
                        </label>
                        <input
                            type="password"
                            placeholder="请输入密码"
                            class="w-full px-3 py-2 rounded-md focus:outline-none focus:ring-2 transition-all"
                            style=format!(
                                "{}; focus:ring-color: {};",
                                theme.input_style(),
                                theme.blue
                            )
                            prop:value={move || password.get()}
                            on:input=move |e| set_password.set(event_target_value(&e))
                        />
                    </div>

                    // 确认密码字段（仅注册时显示）
                    <div class:hidden={move || auth_mode.get() != AuthMode::Register}>
                        <label 
                            class="block text-sm font-medium mb-2"
                            style=format!("color: {};", theme.text)
                        >
                            "确认密码"
                        </label>
                        <input
                            type="password"
                            placeholder="请再次输入密码"
                            class="w-full px-3 py-2 rounded-md focus:outline-none focus:ring-2 transition-all"
                            style=format!(
                                "{}; focus:ring-color: {};",
                                theme.input_style(),
                                theme.blue
                            )
                            prop:value={move || confirm_password.get()}
                            on:input=move |e| set_confirm_password.set(event_target_value(&e))
                        />
                    </div>

                    // 提交按钮
                    <button
                        type="submit"
                        disabled={move || auth.is_loading.get()}
                        class="w-full py-2 px-4 rounded-md focus:outline-none focus:ring-2 disabled:opacity-50 disabled:cursor-not-allowed hover:opacity-90 transition-all"
                        style=format!(
                            "{}; focus:ring-color: {};",
                            theme.button_primary_style(),
                            theme.blue
                        )
                    >
                        {move || if auth.is_loading.get() {
                            match auth_mode.get() {
                                AuthMode::Login => "登录中...",
                                AuthMode::Register => "注册中...",
                            }
                        } else {
                            match auth_mode.get() {
                                AuthMode::Login => "登录",
                                AuthMode::Register => "注册",
                            }
                        }}
                    </button>
                </form>

                // 切换模式
                <div class="mt-6 text-center">
                    <button
                        class="text-sm hover:opacity-75 transition-opacity"
                        style=format!("color: {};", theme.blue)
                        on:click=move |_| toggle_mode()
                    >
                        {move || match auth_mode.get() {
                            AuthMode::Login => "没有账号？点击注册",
                            AuthMode::Register => "已有账号？点击登录",
                        }}
                    </button>
                </div>
            </div>
        </div>
    }
}