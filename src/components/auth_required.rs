//! 需要认证的组件包装器
//! 
//! 当用户未登录时显示登录提示，已登录时显示子组件

use crate::hooks::use_auth::{use_auth, AuthStatus};
use crate::components::AuthModal;
use crate::theme::use_theme_context;
use leptos::*;

#[component]
pub fn AuthRequired(
    /// 子组件内容
    children: ChildrenFn,
    /// 自定义提示消息
    #[prop(optional)]
    message: Option<String>,
) -> impl IntoView {
    let auth = use_auth();
    let theme_context = use_theme_context();
    
    // 控制认证模态框显示状态
    let (show_auth_modal, set_show_auth_modal) = create_signal(false);
    
    let default_message = "请先登录后再使用此功能".to_string();
    let prompt_message = message.unwrap_or(default_message);

    view! {
        <Show
            when=move || matches!(auth.auth_status.get(), AuthStatus::Authenticated(_))
            fallback=move || {
                view! {
                    <div class="rounded-lg shadow-lg p-6 text-center" style=move || theme_context.get().theme.card_style()>
                        <div class="mb-4">
                            <svg class="w-16 h-16 mx-auto mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24" style=move || format!("color: {};", theme_context.get().theme.warning_color())>
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z"></path>
                            </svg>
                        </div>
                        <h3 class="text-lg font-semibold mb-2" style=move || theme_context.get().theme.text_style()>
                            "需要登录"
                        </h3>
                        <p class="mb-4" style=move || theme_context.get().theme.subtext_style()>
                            {prompt_message.clone()}
                        </p>
                        <div class="space-x-4">
                            <button
                                class="px-6 py-2 rounded-md transition-colors hover:opacity-90"
                                style=move || theme_context.get().theme.button_primary_style()
                                on:click=move |_| {
                                    set_show_auth_modal.set(true);
                                }
                            >
                                "登录"
                            </button>
                        </div>
                    </div>
                    
                    // 认证模态框
                    <AuthModal 
                        show=Signal::from(show_auth_modal)
                        on_close=set_show_auth_modal
                    />
                }
            }
        >
            {children()}
        </Show>
    }
}