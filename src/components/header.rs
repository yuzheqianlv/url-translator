use leptos::*;
use leptos_router::*;

use crate::hooks::use_auth::{use_auth, AuthStatus, is_authenticated};
use crate::components::AuthModal;

#[component]
pub fn Header() -> impl IntoView {
    let auth = use_auth();
    
    // 控制认证模态框显示
    let (show_auth_modal, set_show_auth_modal) = create_signal(false);
    
    // 用户菜单显示状态
    let (show_user_menu, set_show_user_menu) = create_signal(false);

    // 退出登录处理
    let handle_logout = move || {
        auth.logout.set(true);
        set_show_user_menu.set(false);
    };

    view! {
        <header class="bg-white shadow-sm border-b">
            <div class="container mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    <div class="flex items-center space-x-4">
                        <A href="/" class="text-xl font-bold text-gray-800 hover:text-blue-600">
                            "URL翻译工具"
                        </A>
                    </div>

                    <nav class="flex items-center space-x-6">
                        <A
                            href="/"
                            class="text-gray-600 hover:text-blue-600 transition-colors"
                            active_class="text-blue-600 font-medium"
                        >
                            "首页"
                        </A>
                        
                        // 已认证用户可见的导航
                        {move || {
                            if is_authenticated(&auth.auth_status.get()) {
                                view! {
                                    <>
                                        <A
                                            href="/history"
                                            class="text-gray-600 hover:text-blue-600 transition-colors"
                                            active_class="text-blue-600 font-medium"
                                        >
                                            "历史记录"
                                        </A>
                                        <A
                                            href="/projects"
                                            class="text-gray-600 hover:text-blue-600 transition-colors"
                                            active_class="text-blue-600 font-medium"
                                        >
                                            "项目管理"
                                        </A>
                                    </>
                                }.into_view()
                            } else {
                                view! { <></> }.into_view()
                            }
                        }}
                        
                        <A
                            href="/settings"
                            class="text-gray-600 hover:text-blue-600 transition-colors"
                            active_class="text-blue-600 font-medium"
                        >
                            "设置"
                        </A>

                        // 认证状态相关的按钮
                        {move || {
                            match auth.auth_status.get() {
                                AuthStatus::Authenticated(ref user) => {
                                    let user_name = user.username.clone();
                                    view! {
                                        <div class="relative">
                                            <button
                                                class="flex items-center space-x-2 text-gray-700 hover:text-blue-600 transition-colors"
                                                on:click=move |_| set_show_user_menu.update(|show| *show = !*show)
                                            >
                                                <div class="w-8 h-8 bg-blue-600 text-white rounded-full flex items-center justify-center text-sm font-medium">
                                                    {user_name.chars().next().unwrap_or('U').to_uppercase().to_string()}
                                                </div>
                                                <span class="text-sm">{user_name}</span>
                                                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"></path>
                                                </svg>
                                            </button>

                                            // 用户菜单
                                            <div 
                                                class:hidden={move || !show_user_menu.get()}
                                                class="absolute right-0 mt-2 w-48 bg-white rounded-md shadow-lg py-1 z-50"
                                            >
                                                <A
                                                    href="/profile"
                                                    class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                                                    on:click=move |_| set_show_user_menu.set(false)
                                                >
                                                    "个人资料"
                                                </A>
                                                <button
                                                    class="block w-full text-left px-4 py-2 text-sm text-gray-700 hover:bg-gray-100"
                                                    on:click=move |_| handle_logout()
                                                >
                                                    "退出登录"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }
                                AuthStatus::Authenticating => {
                                    view! {
                                        <div class="flex items-center space-x-2">
                                            <div class="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
                                            <span class="text-sm text-gray-600">"登录中..."</span>
                                        </div>
                                    }
                                }
                                _ => {
                                    view! {
                                        <div>
                                            <button
                                                class="bg-blue-600 text-white px-4 py-2 rounded-md hover:bg-blue-700 transition-colors"
                                                on:click=move |_| set_show_auth_modal.set(true)
                                            >
                                                "登录"
                                            </button>
                                        </div>
                                    }
                                }
                            }
                        }}
                    </nav>
                </div>
            </div>
        </header>

        // 认证模态框
        <AuthModal 
            show=show_auth_modal
            on_close=set_show_auth_modal
        />
    }
}
