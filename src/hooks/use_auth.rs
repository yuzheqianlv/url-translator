//! 用户认证状态管理Hook
//! 
//! 这个Hook提供了完整的用户认证功能，包括：
//! - 用户登录/注册
//! - 认证状态管理
//! - Token存储和自动刷新
//! - 退出登录

use leptos::*;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;

use crate::services::api_client::{
    ApiClient, ApiConfig, LoginRequest, RegisterRequest, 
    UserProfile
};
use crate::error::{use_error_handler, AppError};

/// 认证状态
#[derive(Clone, Debug)]
pub enum AuthStatus {
    /// 未初始化（正在检查本地存储）
    Uninitialized,
    /// 未认证
    Unauthenticated,
    /// 认证中（登录/注册进行中）
    Authenticating,
    /// 已认证
    Authenticated(UserProfile),
    /// 认证失败
    Failed(String),
}

/// 认证状态管理返回值
#[derive(Clone)]
pub struct UseAuthReturn {
    /// 当前认证状态
    pub auth_status: ReadSignal<AuthStatus>,
    /// 当前用户信息（如果已认证）
    pub user: Memo<Option<UserProfile>>,
    /// 是否正在加载
    pub is_loading: ReadSignal<bool>,
    /// 登录函数
    pub login: WriteSignal<Option<LoginRequest>>,
    /// 注册函数
    pub register: WriteSignal<Option<RegisterRequest>>,
    /// 退出登录函数
    pub logout: WriteSignal<bool>,
    /// API客户端（已包含认证token）
    pub api_client: ReadSignal<ApiClient>,
}

pub fn use_auth() -> UseAuthReturn {
    let error_handler = use_error_handler();
    
    // 创建信号
    let (auth_status, set_auth_status) = create_signal(AuthStatus::Uninitialized);
    let (is_loading, set_is_loading) = create_signal(false);
    let (login_trigger, set_login_trigger) = create_signal(None::<LoginRequest>);
    let (register_trigger, set_register_trigger) = create_signal(None::<RegisterRequest>);
    let (logout_trigger, set_logout_trigger) = create_signal(false);
    
    // 创建API客户端
    let (api_client, set_api_client) = create_signal(ApiClient::new(ApiConfig::default()));
    
    // 从auth_status派生user信号
    let user = create_memo(move |_| {
        match auth_status.get() {
            AuthStatus::Authenticated(user) => Some(user),
            _ => None,
        }
    });

    // 初始化：检查本地存储的token
    create_effect({
        let set_auth_status = set_auth_status;
        let set_api_client = set_api_client;
        
        move |_| {
            spawn_local(async move {
                if let Ok(stored_token) = LocalStorage::get::<String>("auth_token") {
                    if let Ok(stored_user) = LocalStorage::get::<UserProfile>("user_profile") {
                        // 创建带认证的API客户端
                        let mut client = ApiClient::new(ApiConfig::default());
                        client.set_auth_token(Some(stored_token));
                        set_api_client.set(client);
                        
                        // 设置认证状态
                        set_auth_status.set(AuthStatus::Authenticated(stored_user));
                        
                        web_sys::console::log_1(&"从本地存储恢复认证状态".into());
                    } else {
                        // Token存在但用户信息不存在，清除token
                        let _ = LocalStorage::delete("auth_token");
                        set_auth_status.set(AuthStatus::Unauthenticated);
                    }
                } else {
                    set_auth_status.set(AuthStatus::Unauthenticated);
                }
            });
        }
    });

    // 登录Effect
    create_effect({
        let set_auth_status = set_auth_status;
        let set_is_loading = set_is_loading;
        let set_api_client = set_api_client;
        let error_handler = error_handler.clone();
        
        move |_| {
            if let Some(login_request) = login_trigger.get() {
                set_is_loading.set(true);
                set_auth_status.set(AuthStatus::Authenticating);
                
                let set_auth_status = set_auth_status;
                let set_is_loading = set_is_loading;
                let set_api_client = set_api_client;
                let error_handler = error_handler.clone();
                
                spawn_local(async move {
                    web_sys::console::log_1(&format!("开始登录: {}", login_request.email).into());
                    
                    let client = ApiClient::new(ApiConfig::default());
                    
                    match client.login(login_request).await {
                        Ok(login_response) => {
                            web_sys::console::log_1(&"登录成功".into());
                            
                            // 存储token和用户信息
                            if let Err(e) = LocalStorage::set("auth_token", &login_response.access_token) {
                                web_sys::console::log_1(&format!("存储token失败: {:?}", e).into());
                            }
                            if let Err(e) = LocalStorage::set("user_profile", &login_response.user) {
                                web_sys::console::log_1(&format!("存储用户信息失败: {:?}", e).into());
                            }
                            
                            // 创建带认证的API客户端
                            let mut authenticated_client = ApiClient::new(ApiConfig::default());
                            authenticated_client.set_auth_token(Some(login_response.access_token));
                            set_api_client.set(authenticated_client);
                            
                            // 更新认证状态
                            set_auth_status.set(AuthStatus::Authenticated(login_response.user));
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("登录失败: {}", e).into());
                            set_auth_status.set(AuthStatus::Failed(e.clone()));
                            error_handler.handle_error(AppError::auth(e));
                        }
                    }
                    
                    set_is_loading.set(false);
                });
            }
        }
    });

    // 注册Effect
    create_effect({
        let set_auth_status = set_auth_status;
        let set_is_loading = set_is_loading;
        let error_handler = error_handler.clone();
        
        move |_| {
            if let Some(register_request) = register_trigger.get() {
                set_is_loading.set(true);
                set_auth_status.set(AuthStatus::Authenticating);
                
                let set_auth_status = set_auth_status;
                let set_is_loading = set_is_loading;
                let error_handler = error_handler.clone();
                
                spawn_local(async move {
                    web_sys::console::log_1(&format!("开始注册: {}", register_request.email).into());
                    
                    let client = ApiClient::new(ApiConfig::default());
                    
                    match client.register(register_request).await {
                        Ok(user) => {
                            web_sys::console::log_1(&"注册成功，请登录".into());
                            // 注册成功后不自动登录，需要用户手动登录
                            set_auth_status.set(AuthStatus::Unauthenticated);
                            
                            // 可以显示成功消息
                            web_sys::console::log_1(&format!("注册成功，用户: {}", user.username).into());
                        }
                        Err(e) => {
                            web_sys::console::log_1(&format!("注册失败: {}", e).into());
                            set_auth_status.set(AuthStatus::Failed(e.clone()));
                            error_handler.handle_error(AppError::auth(e));
                        }
                    }
                    
                    set_is_loading.set(false);
                });
            }
        }
    });

    // 退出登录Effect
    create_effect({
        let set_auth_status = set_auth_status;
        let set_api_client = set_api_client;
        
        move |_| {
            if logout_trigger.get() {
                web_sys::console::log_1(&"开始退出登录".into());
                
                // 清除本地存储
                let _ = LocalStorage::delete("auth_token");
                let _ = LocalStorage::delete("user_profile");
                
                // 创建未认证的API客户端
                let client = ApiClient::new(ApiConfig::default());
                set_api_client.set(client);
                
                // 更新认证状态
                set_auth_status.set(AuthStatus::Unauthenticated);
                
                web_sys::console::log_1(&"退出登录完成".into());
                
                // 重置退出触发器
                set_logout_trigger.set(false);
            }
        }
    });

    UseAuthReturn {
        auth_status,
        user,
        is_loading,
        login: set_login_trigger,
        register: set_register_trigger,
        logout: set_logout_trigger,
        api_client,
    }
}

/// 检查是否已认证的便捷函数
pub fn is_authenticated(auth_status: &AuthStatus) -> bool {
    matches!(auth_status, AuthStatus::Authenticated(_))
}

/// 获取当前用户的便捷函数
pub fn get_current_user(auth_status: &AuthStatus) -> Option<&UserProfile> {
    match auth_status {
        AuthStatus::Authenticated(user) => Some(user),
        _ => None,
    }
}

/// 获取用户配置Hook（需要认证）
pub fn use_user_config() -> (
    ReadSignal<Option<crate::services::api_client::UserConfigResponse>>,
    WriteSignal<Option<crate::services::api_client::UpdateUserConfigRequest>>,
    ReadSignal<bool>
) {
    let auth = use_auth();
    let error_handler = use_error_handler();
    
    let (config, set_config) = create_signal(None);
    let (update_trigger, set_update_trigger) = create_signal(None);
    let (is_loading, set_is_loading) = create_signal(false);
    
    // 加载用户配置
    create_effect({
        let auth = auth.clone();
        let set_config = set_config;
        let set_is_loading = set_is_loading;
        let error_handler = error_handler.clone();
        
        move |_| {
            if let Some(_) = auth.user.get() {
                set_is_loading.set(true);
                
                let client = auth.api_client.get();
                let set_config = set_config;
                let set_is_loading = set_is_loading;
                let error_handler = error_handler.clone();
                
                spawn_local(async move {
                    match client.get_user_config().await {
                        Ok(config) => {
                            set_config.set(Some(config));
                        }
                        Err(e) => {
                            error_handler.handle_error(AppError::api("auth", e));
                        }
                    }
                    set_is_loading.set(false);
                });
            }
        }
    });
    
    // 更新用户配置
    create_effect({
        let auth = auth.clone();
        let set_config = set_config;
        let set_is_loading = set_is_loading;
        let error_handler = error_handler.clone();
        
        move |_| {
            if let Some(update_request) = update_trigger.get() {
                set_is_loading.set(true);
                
                let client = auth.api_client.get();
                let set_config = set_config;
                let set_is_loading = set_is_loading;
                let error_handler = error_handler.clone();
                
                spawn_local(async move {
                    match client.update_user_config(update_request).await {
                        Ok(()) => {
                            // 重新加载配置
                            match client.get_user_config().await {
                                Ok(config) => {
                                    set_config.set(Some(config));
                                }
                                Err(e) => {
                                    error_handler.handle_error(AppError::api("auth", e));
                                }
                            }
                        }
                        Err(e) => {
                            error_handler.handle_error(AppError::api("auth", e));
                        }
                    }
                    set_is_loading.set(false);
                });
            }
        }
    });
    
    (config, set_update_trigger, is_loading)
}