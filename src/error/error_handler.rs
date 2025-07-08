use leptos::*;
use crate::error::{AppError, ErrorContext, ErrorSeverity};
use std::collections::VecDeque;

const MAX_ERROR_HISTORY: usize = 50;

#[derive(Clone, Debug)]
pub struct ErrorState {
    pub current_error: Option<ErrorContext>,
    pub error_history: VecDeque<ErrorContext>,
    pub is_visible: bool,
}

impl Default for ErrorState {
    fn default() -> Self {
        Self {
            current_error: None,
            error_history: VecDeque::new(),
            is_visible: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ErrorHandler {
    error_state: RwSignal<ErrorState>,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self {
            error_state: create_rw_signal(ErrorState::default()),
        }
    }
    
    /// 处理错误并显示给用户
    pub fn handle_error(&self, error: AppError) {
        let context = ErrorContext::new(error);
        self.set_error(context);
    }
    
    /// 设置错误上下文
    pub fn set_error(&self, context: ErrorContext) {
        self.error_state.update(|state| {
            // 添加到历史记录
            state.error_history.push_back(context.clone());
            if state.error_history.len() > MAX_ERROR_HISTORY {
                state.error_history.pop_front();
            }
            
            // 设置当前错误
            state.current_error = Some(context);
            state.is_visible = true;
        });
        
        // 根据严重程度自动设置显示时间
        let auto_hide_delay = match self.get_current_severity() {
            Some(ErrorSeverity::Low) => Some(3000),
            Some(ErrorSeverity::Medium) => Some(5000),
            _ => None, // High 和 Critical 需要手动关闭
        };
        
        if let Some(delay) = auto_hide_delay {
            let error_handler = self.clone();
            spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(delay).await;
                error_handler.hide_error();
            });
        }
    }
    
    /// 清除当前错误
    pub fn clear_error(&self) {
        self.error_state.update(|state| {
            state.current_error = None;
            state.is_visible = false;
        });
    }
    
    /// 隐藏错误显示（但保留在上下文中）
    pub fn hide_error(&self) {
        self.error_state.update(|state| {
            state.is_visible = false;
        });
    }
    
    /// 显示错误
    pub fn show_error(&self) {
        self.error_state.update(|state| {
            if state.current_error.is_some() {
                state.is_visible = true;
            }
        });
    }
    
    /// 获取当前错误
    pub fn get_current_error(&self) -> Option<ErrorContext> {
        self.error_state.get().current_error
    }
    
    /// 获取当前错误的严重程度
    pub fn get_current_severity(&self) -> Option<ErrorSeverity> {
        self.get_current_error().map(|ctx| ctx.severity)
    }
    
    /// 检查是否有错误显示
    pub fn is_error_visible(&self) -> bool {
        self.error_state.get().is_visible
    }
    
    /// 获取错误历史
    pub fn get_error_history(&self) -> Vec<ErrorContext> {
        self.error_state.get().error_history.iter().cloned().collect()
    }
    
    /// 清空错误历史
    pub fn clear_history(&self) {
        self.error_state.update(|state| {
            state.error_history.clear();
        });
    }
    
    /// 重试当前错误对应的操作
    pub fn retry_current(&self) -> Option<ErrorContext> {
        self.error_state.get().current_error.map(|ctx| ctx.increment_retry())
    }
    
    /// 检查当前错误是否可以重试
    pub fn can_retry_current(&self, max_retries: u32) -> bool {
        self.get_current_error()
            .map(|ctx| ctx.can_retry(max_retries))
            .unwrap_or(false)
    }
    
    /// 创建错误信号，用于组件中监听错误状态
    pub fn create_error_signal(&self) -> (Memo<Option<ErrorContext>>, Memo<bool>) {
        let error_state = self.error_state;
        let current_error = create_memo(move |_| error_state.get().current_error);
        let is_visible = create_memo(move |_| error_state.get().is_visible);
        
        (current_error, is_visible)
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局错误处理器提供上下文
#[component]
pub fn ErrorProvider(children: Children) -> impl IntoView {
    let error_handler = ErrorHandler::new();
    provide_context(error_handler);
    children()
}

/// 获取错误处理器的钩子
pub fn use_error_handler() -> ErrorHandler {
    use_context::<ErrorHandler>()
        .expect("ErrorHandler context not found. Make sure to wrap your app with ErrorProvider.")
}

/// 错误显示组件
#[component]
pub fn ErrorDisplay() -> impl IntoView {
    let error_handler = use_error_handler();
    let error_state = error_handler.error_state;
    
    view! {
        <Show when=move || error_state.get().is_visible>
            <ErrorCard error_handler=error_handler />
        </Show>
    }
}

#[component]
fn ErrorCard(error_handler: ErrorHandler) -> impl IntoView {
    view! {
        <div class="fixed top-4 right-4 max-w-md p-4 rounded-lg shadow-lg border z-50 themed-alert-error">
            {move || {
                if let Some(ctx) = error_handler.get_current_error() {
                    let severity_class = match ctx.severity {
                        ErrorSeverity::Low => "themed-alert-info",
                        ErrorSeverity::Medium => "themed-alert-info",
                        ErrorSeverity::High => "themed-alert-error",
                        ErrorSeverity::Critical => "themed-alert-error",
                    };
                    
                    let icon = match ctx.severity {
                        ErrorSeverity::Low | ErrorSeverity::Medium => "ℹ️",
                        ErrorSeverity::High | ErrorSeverity::Critical => "⚠️",
                    };
                    
                    view! {
                        <div class=format!("flex items-start space-x-3 {}", severity_class)>
                            <span class="text-lg">{icon}</span>
                            <div class="flex-1">
                                <div class="font-medium mb-1">
                                    {ctx.user_message.clone()}
                                </div>
                                
                                {if !ctx.suggested_actions.is_empty() {
                                    view! {
                                        <div class="text-sm mt-2">
                                            <div class="font-medium mb-1">"建议操作:"</div>
                                            <ul class="list-disc list-inside space-y-1">
                                                {ctx.suggested_actions.iter().map(|action| {
                                                    view! { <li>{action.clone()}</li> }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        </div>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                                
                                {if error_handler.can_retry_current(3) {
                                    view! {
                                        <button 
                                            class="mt-2 px-3 py-1 text-sm themed-button-secondary rounded"
                                            on:click=move |_| error_handler.clear_error()
                                        >
                                            "重试"
                                        </button>
                                    }.into_view()
                                } else {
                                    view! {}.into_view()
                                }}
                            </div>
                            
                            <button 
                                class="text-lg leading-none"
                                on:click=move |_| error_handler.clear_error()
                            >
                                "×"
                            </button>
                        </div>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}
        </div>
    }
}

/// 简化的错误处理宏
#[macro_export]
macro_rules! handle_error {
    ($error_handler:expr, $result:expr) => {
        match $result {
            Ok(value) => Some(value),
            Err(err) => {
                $error_handler.handle_error(err.into());
                None
            }
        }
    };
}

/// 异步错误处理宏
#[macro_export]
macro_rules! handle_async_error {
    ($error_handler:expr, $future:expr) => {
        match $future.await {
            Ok(value) => Some(value),
            Err(err) => {
                $error_handler.handle_error(err.into());
                None
            }
        }
    };
}