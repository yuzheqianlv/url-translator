use crate::error::{use_error_handler, AppError};
use crate::services::config_service::ConfigService;
use crate::types::api_types::AppConfig;
use leptos::*;

pub struct UseConfigReturn {
    pub config: ReadSignal<AppConfig>,
    pub save_config: Box<dyn Fn(AppConfig)>,
    pub reset_config: Box<dyn Fn()>,
    pub is_loading: ReadSignal<bool>,
}

pub fn use_config() -> UseConfigReturn {
    let error_handler = use_error_handler();
    let config_service = ConfigService::new();

    let initial_config = config_service.get_config().unwrap_or_default();
    let (config, set_config) = create_signal(initial_config);
    let (is_loading, set_is_loading) = create_signal(false);

    let save_config = {
        let config_service = config_service.clone();
        let set_config = set_config.clone();
        let set_is_loading = set_is_loading.clone();

        Box::new(move |new_config: AppConfig| {
            set_is_loading.set(true);

            match config_service.save_config(&new_config) {
                Ok(_) => {
                    set_config.set(new_config);
                    web_sys::console::log_1(&"配置保存成功".into());
                }
                Err(e) => {
                    error_handler.handle_error(AppError::config(format!("保存配置失败: {}", e)));
                }
            }

            set_is_loading.set(false);
        })
    };

    let reset_config = {
        let config_service = config_service.clone();
        let set_config = set_config.clone();
        let set_is_loading = set_is_loading.clone();

        Box::new(move || {
            set_is_loading.set(true);

            let default_config = AppConfig::default();
            match config_service.save_config(&default_config) {
                Ok(_) => {
                    set_config.set(default_config);
                    web_sys::console::log_1(&"配置已重置为默认值".into());
                }
                Err(e) => {
                    error_handler.handle_error(AppError::config(format!("重置配置失败: {}", e)));
                }
            }

            set_is_loading.set(false);
        })
    };

    UseConfigReturn {
        config,
        save_config,
        reset_config,
        is_loading,
    }
}
