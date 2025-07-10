//! 功能开关管理
//! 
//! 集中管理应用的功能开关，支持运行时切换和环境变量控制

use crate::config::env::CompileTimeEnv;
use leptos::*;
use std::collections::HashMap;

/// 功能标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Feature {
    /// 项目管理功能
    ProjectManagement,
    /// 历史记录功能
    History,
    /// 搜索功能
    Search,
    /// 批量翻译功能
    BatchTranslation,
    /// 用户认证功能
    UserAuthentication,
    /// 主题切换功能
    ThemeSwitching,
    /// 文件上传功能
    FileUpload,
    /// 离线模式
    OfflineMode,
    /// PWA支持
    PwaSupport,
    /// 键盘快捷键
    KeyboardShortcuts,
    /// 性能监控
    PerformanceMonitoring,
    /// 错误追踪
    ErrorTracking,
    /// 自动保存
    AutoSave,
}

impl Feature {
    /// 获取功能的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            Feature::ProjectManagement => "项目管理",
            Feature::History => "历史记录",
            Feature::Search => "搜索功能",
            Feature::BatchTranslation => "批量翻译",
            Feature::UserAuthentication => "用户认证",
            Feature::ThemeSwitching => "主题切换",
            Feature::FileUpload => "文件上传",
            Feature::OfflineMode => "离线模式",
            Feature::PwaSupport => "PWA支持",
            Feature::KeyboardShortcuts => "键盘快捷键",
            Feature::PerformanceMonitoring => "性能监控",
            Feature::ErrorTracking => "错误追踪",
            Feature::AutoSave => "自动保存",
        }
    }

    /// 获取功能的描述
    pub fn description(&self) -> &'static str {
        match self {
            Feature::ProjectManagement => "管理翻译项目，组织和分类翻译工作",
            Feature::History => "查看和管理翻译历史记录",
            Feature::Search => "搜索翻译内容和历史记录",
            Feature::BatchTranslation => "批量翻译多个URL或文件",
            Feature::UserAuthentication => "用户登录和账户管理",
            Feature::ThemeSwitching => "切换应用主题和外观",
            Feature::FileUpload => "上传文件进行翻译",
            Feature::OfflineMode => "离线模式下的基本功能",
            Feature::PwaSupport => "渐进式Web应用支持",
            Feature::KeyboardShortcuts => "键盘快捷键操作",
            Feature::PerformanceMonitoring => "应用性能监控和分析",
            Feature::ErrorTracking => "错误收集和报告",
            Feature::AutoSave => "自动保存用户输入和设置",
        }
    }

    /// 获取功能的默认启用状态
    pub fn default_enabled(&self) -> bool {
        match self {
            Feature::ProjectManagement => CompileTimeEnv::is_project_management_enabled(),
            Feature::History => CompileTimeEnv::is_history_enabled(),
            Feature::Search => CompileTimeEnv::is_search_enabled(),
            Feature::BatchTranslation => CompileTimeEnv::is_batch_translation_enabled(),
            Feature::UserAuthentication => true,
            Feature::ThemeSwitching => true,
            Feature::FileUpload => true,
            Feature::OfflineMode => false,
            Feature::PwaSupport => false,
            Feature::KeyboardShortcuts => true,
            Feature::PerformanceMonitoring => CompileTimeEnv::is_debug_mode(),
            Feature::ErrorTracking => true,
            Feature::AutoSave => true,
        }
    }

    /// 检查功能是否需要用户认证
    pub fn requires_authentication(&self) -> bool {
        match self {
            Feature::ProjectManagement => true,
            Feature::History => true,
            Feature::Search => true,
            Feature::BatchTranslation => true,
            Feature::UserAuthentication => false,
            Feature::ThemeSwitching => false,
            Feature::FileUpload => false,
            Feature::OfflineMode => false,
            Feature::PwaSupport => false,
            Feature::KeyboardShortcuts => false,
            Feature::PerformanceMonitoring => false,
            Feature::ErrorTracking => false,
            Feature::AutoSave => false,
        }
    }

    /// 获取所有可用的功能
    pub fn all() -> Vec<Feature> {
        vec![
            Feature::ProjectManagement,
            Feature::History,
            Feature::Search,
            Feature::BatchTranslation,
            Feature::UserAuthentication,
            Feature::ThemeSwitching,
            Feature::FileUpload,
            Feature::OfflineMode,
            Feature::PwaSupport,
            Feature::KeyboardShortcuts,
            Feature::PerformanceMonitoring,
            Feature::ErrorTracking,
            Feature::AutoSave,
        ]
    }
}

/// 功能开关管理器
#[derive(Debug, Clone)]
pub struct FeatureFlags {
    flags: HashMap<Feature, bool>,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        let mut flags = HashMap::new();
        for feature in Feature::all() {
            flags.insert(feature.clone(), feature.default_enabled());
        }
        Self { flags }
    }
}

impl FeatureFlags {
    /// 创建新的功能开关管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 检查功能是否启用
    pub fn is_enabled(&self, feature: &Feature) -> bool {
        self.flags.get(feature).copied().unwrap_or(false)
    }

    /// 启用功能
    pub fn enable(&mut self, feature: Feature) {
        self.flags.insert(feature, true);
    }

    /// 禁用功能
    pub fn disable(&mut self, feature: Feature) {
        self.flags.insert(feature, false);
    }

    /// 切换功能状态
    pub fn toggle(&mut self, feature: Feature) {
        let current = self.is_enabled(&feature);
        self.flags.insert(feature, !current);
    }

    /// 批量设置功能状态
    pub fn set_multiple(&mut self, features: Vec<(Feature, bool)>) {
        for (feature, enabled) in features {
            self.flags.insert(feature, enabled);
        }
    }

    /// 获取所有启用的功能
    pub fn enabled_features(&self) -> Vec<Feature> {
        self.flags
            .iter()
            .filter_map(|(feature, &enabled)| {
                if enabled {
                    Some(feature.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 获取所有禁用的功能
    pub fn disabled_features(&self) -> Vec<Feature> {
        self.flags
            .iter()
            .filter_map(|(feature, &enabled)| {
                if !enabled {
                    Some(feature.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 重置为默认状态
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// 导出配置为JSON格式
    pub fn to_json(&self) -> String {
        let mut items = Vec::new();
        for (feature, enabled) in &self.flags {
            items.push(format!(
                "\"{}\":{}",
                format!("{:?}", feature).to_lowercase(),
                enabled
            ));
        }
        format!("{{{}}}", items.join(","))
    }

    /// 从JSON格式导入配置
    pub fn from_json(&mut self, json: &str) -> Result<(), String> {
        // 简单的JSON解析实现
        // 在实际项目中建议使用serde_json
        if !json.starts_with('{') || !json.ends_with('}') {
            return Err("无效的JSON格式".to_string());
        }

        let content = &json[1..json.len()-1];
        if content.is_empty() {
            return Ok(());
        }

        for pair in content.split(',') {
            let parts: Vec<&str> = pair.split(':').collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim().trim_matches('"');
            let value = parts[1].trim();

            if let Some(feature) = self.parse_feature_name(key) {
                if let Ok(enabled) = value.parse::<bool>() {
                    self.flags.insert(feature, enabled);
                }
            }
        }

        Ok(())
    }

    /// 解析功能名称
    fn parse_feature_name(&self, name: &str) -> Option<Feature> {
        match name.to_lowercase().as_str() {
            "projectmanagement" => Some(Feature::ProjectManagement),
            "history" => Some(Feature::History),
            "search" => Some(Feature::Search),
            "batchtranslation" => Some(Feature::BatchTranslation),
            "userauthentication" => Some(Feature::UserAuthentication),
            "themeswitching" => Some(Feature::ThemeSwitching),
            "fileupload" => Some(Feature::FileUpload),
            "offlinemode" => Some(Feature::OfflineMode),
            "pwasupport" => Some(Feature::PwaSupport),
            "keyboardshortcuts" => Some(Feature::KeyboardShortcuts),
            "performancemonitoring" => Some(Feature::PerformanceMonitoring),
            "errortracking" => Some(Feature::ErrorTracking),
            "autosave" => Some(Feature::AutoSave),
            _ => None,
        }
    }

    /// 生成功能摘要报告
    pub fn summary(&self) -> String {
        let enabled = self.enabled_features();
        let disabled = self.disabled_features();

        format!(
            "功能开关状态:\n\
            启用的功能 ({}):\n{}\n\n\
            禁用的功能 ({}):\n{}",
            enabled.len(),
            enabled
                .iter()
                .map(|f| format!("  - {}", f.display_name()))
                .collect::<Vec<_>>()
                .join("\n"),
            disabled.len(),
            disabled
                .iter()
                .map(|f| format!("  - {}", f.display_name()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// 全局功能开关管理器
static GLOBAL_FEATURES: std::sync::OnceLock<std::sync::RwLock<FeatureFlags>> = std::sync::OnceLock::new();

/// 获取全局功能开关管理器
pub fn global_features() -> &'static std::sync::RwLock<FeatureFlags> {
    GLOBAL_FEATURES.get_or_init(|| std::sync::RwLock::new(FeatureFlags::new()))
}

/// 检查功能是否启用（全局）
pub fn is_feature_enabled(feature: &Feature) -> bool {
    global_features()
        .read()
        .map(|flags| flags.is_enabled(feature))
        .unwrap_or(false)
}

/// Leptos Hook：使用功能开关
pub fn use_feature_flags() -> (ReadSignal<FeatureFlags>, WriteSignal<FeatureFlags>) {
    let (flags, set_flags) = create_signal(FeatureFlags::new());

    // 初始化时从全局状态加载
    create_effect(move |_| {
        if let Ok(global_flags) = global_features().read() {
            set_flags.set(global_flags.clone());
        }
    });

    (flags, set_flags)
}

/// Leptos Hook：检查特定功能是否启用
pub fn use_feature(feature: Feature) -> ReadSignal<bool> {
    let (flags, _) = use_feature_flags();
    let memo = create_memo(move |_| flags.get().is_enabled(&feature));
    create_signal(memo.get()).0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_properties() {
        let feature = Feature::ProjectManagement;
        assert!(!feature.display_name().is_empty());
        assert!(!feature.description().is_empty());
        assert!(feature.requires_authentication());
    }

    #[test]
    fn test_feature_flags() {
        let mut flags = FeatureFlags::new();
        
        assert!(flags.is_enabled(&Feature::UserAuthentication));
        
        flags.disable(Feature::UserAuthentication);
        assert!(!flags.is_enabled(&Feature::UserAuthentication));
        
        flags.toggle(Feature::UserAuthentication);
        assert!(flags.is_enabled(&Feature::UserAuthentication));
    }

    #[test]
    fn test_json_serialization() {
        let flags = FeatureFlags::new();
        let json = flags.to_json();
        assert!(json.contains("userauthentication"));
        
        let mut new_flags = FeatureFlags::new();
        assert!(new_flags.from_json(&json).is_ok());
    }

    #[test]
    fn test_feature_lists() {
        let flags = FeatureFlags::new();
        let enabled = flags.enabled_features();
        let disabled = flags.disabled_features();
        
        assert!(!enabled.is_empty());
        assert_eq!(enabled.len() + disabled.len(), Feature::all().len());
    }
}