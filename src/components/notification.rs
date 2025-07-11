//! 通知组件
//! 
//! 提供成功、错误、警告等通知消息的显示

use leptos::*;
use crate::theme::catppuccin::{ThemeVariant, CatppuccinTheme};
use std::time::Duration;
use uuid::Uuid;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Clone, Debug)]
pub struct Notification {
    pub id: String,
    pub message: String,
    pub notification_type: NotificationType,
    pub duration: Option<Duration>,
}

impl Notification {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.into(),
            notification_type: NotificationType::Success,
            duration: Some(Duration::from_secs(3)),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.into(),
            notification_type: NotificationType::Error,
            duration: Some(Duration::from_secs(5)),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.into(),
            notification_type: NotificationType::Warning,
            duration: Some(Duration::from_secs(4)),
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            message: message.into(),
            notification_type: NotificationType::Info,
            duration: Some(Duration::from_secs(3)),
        }
    }
}

/// 通知管理器
#[derive(Clone)]
pub struct NotificationManager {
    notifications: RwSignal<Vec<Notification>>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            notifications: create_rw_signal(Vec::new()),
        }
    }

    /// 添加通知
    pub fn add(&self, notification: Notification) {
        let duration = notification.duration;
        let notification_id = notification.id.clone();
        
        self.notifications.update(|notifications| {
            notifications.push(notification);
        });

        // 自动移除通知
        if let Some(duration) = duration {
            let manager = self.clone();
            set_timeout(
                move || {
                    manager.remove(&notification_id);
                },
                duration,
            );
        }
    }

    /// 移除通知
    pub fn remove(&self, id: &str) {
        self.notifications.update(|notifications| {
            notifications.retain(|n| n.id != id);
        });
    }

    /// 清空所有通知
    pub fn clear(&self) {
        self.notifications.set(Vec::new());
    }

    /// 获取通知列表
    pub fn get_notifications(&self) -> ReadSignal<Vec<Notification>> {
        self.notifications.read_only()
    }

    /// 快捷方法
    pub fn success(&self, message: impl Into<String>) {
        self.add(Notification::success(message));
    }

    pub fn error(&self, message: impl Into<String>) {
        self.add(Notification::error(message));
    }

    pub fn warning(&self, message: impl Into<String>) {
        self.add(Notification::warning(message));
    }

    pub fn info(&self, message: impl Into<String>) {
        self.add(Notification::info(message));
    }
}

// 全局通知管理器
thread_local! {
    static NOTIFICATION_MANAGER: NotificationManager = NotificationManager::new();
}

/// 获取全局通知管理器
pub fn use_notifications() -> NotificationManager {
    NOTIFICATION_MANAGER.with(|manager| manager.clone())
}

/// 通知容器组件
#[component]
pub fn NotificationContainer() -> impl IntoView {
    let manager = use_notifications();
    let notifications = manager.get_notifications();
    let theme = CatppuccinTheme::get_theme(&ThemeVariant::default());

    view! {
        <div class="fixed top-4 right-4 z-50 space-y-2">
            <For
                each=move || notifications.get()
                key=|notification| notification.id.clone()
                children=move |notification| {
                    let (bg_color, border_color, text_color, icon) = match notification.notification_type {
                        NotificationType::Success => (theme.green, theme.green, theme.base, "✓"),
                        NotificationType::Error => (theme.red, theme.red, theme.base, "✕"),
                        NotificationType::Warning => (theme.yellow, theme.yellow, theme.base, "⚠"),
                        NotificationType::Info => (theme.blue, theme.blue, theme.base, "ℹ"),
                    };

                    let notification_id = notification.id.clone();
                    let manager_clone = manager.clone();

                    view! {
                        <div
                            class="max-w-sm w-full bg-white shadow-lg rounded-lg pointer-events-auto flex ring-1 ring-black ring-opacity-5 animate-in slide-in-from-right duration-300"
                            style=format!(
                                "background-color: {}; border-left: 4px solid {}; color: {};",
                                theme.surface0, border_color, theme.text
                            )
                        >
                            <div class="flex-1 w-0 p-4">
                                <div class="flex items-start">
                                    <div class="flex-shrink-0">
                                        <div
                                            class="h-5 w-5 rounded-full flex items-center justify-center text-xs font-bold"
                                            style=format!(
                                                "background-color: {}; color: {};",
                                                bg_color, text_color
                                            )
                                        >
                                            {icon}
                                        </div>
                                    </div>
                                    <div class="ml-3 flex-1">
                                        <p class="text-sm font-medium">
                                            {notification.message}
                                        </p>
                                    </div>
                                </div>
                            </div>
                            <div class="flex border-l border-gray-200">
                                <button
                                    class="w-full border border-transparent rounded-none rounded-r-lg p-4 flex items-center justify-center text-sm font-medium hover:opacity-75 focus:outline-none"
                                    style=format!("color: {};", theme.subtext1)
                                    on:click=move |_| {
                                        manager_clone.remove(&notification_id);
                                    }
                                >
                                    "×"
                                </button>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

/// 创建带有过期时间的 setTimeout
fn set_timeout<F>(f: F, duration: Duration)
where
    F: Fn() + 'static,
{
    let closure = wasm_bindgen::closure::Closure::wrap(Box::new(f) as Box<dyn Fn()>);
    web_sys::window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            duration.as_millis() as i32,
        )
        .unwrap();
    closure.forget();
}