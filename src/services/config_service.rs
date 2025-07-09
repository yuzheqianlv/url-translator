use crate::types::api_types::AppConfig;
use gloo_storage::{LocalStorage, Storage};

#[derive(Debug, Clone)]
pub struct ConfigService;

impl ConfigService {
    pub fn new() -> Self {
        Self
    }

    pub fn get_config(&self) -> Result<AppConfig, Box<dyn std::error::Error>> {
        match LocalStorage::get("app_config") {
            Ok(config) => Ok(config),
            Err(_) => {
                let default_config = AppConfig::default();
                self.save_config(&default_config)?;
                Ok(default_config)
            }
        }
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
        LocalStorage::set("app_config", config).map_err(|e| format!("保存配置失败: {:?}", e).into())
    }
}
