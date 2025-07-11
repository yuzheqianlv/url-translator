//! User-related database models

use super::*;
use std::collections::HashSet;

/// 用户角色枚举
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "moderator")]
    Moderator,
    #[serde(rename = "user")]
    User,
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Moderator => write!(f, "moderator"),
            UserRole::User => write!(f, "user"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub role: UserRole,
    pub is_admin: bool,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserConfig {
    pub id: Uuid,
    pub user_id: Uuid,
    pub deeplx_api_url: Option<String>,
    pub jina_api_url: String,
    pub default_source_lang: String,
    pub default_target_lang: String,
    pub max_requests_per_second: i32,
    pub max_text_length: i32,
    pub max_paragraphs_per_request: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 128))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub role: UserRole,
    pub is_admin: bool,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateUserConfigRequest {
    pub deeplx_api_url: Option<String>,
    
    #[validate(url)]
    pub jina_api_url: Option<String>,
    
    #[validate(length(min = 2, max = 10))]
    pub default_source_lang: Option<String>,
    
    #[validate(length(min = 2, max = 10))]
    pub default_target_lang: Option<String>,
    
    #[validate(range(min = 1, max = 100))]
    pub max_requests_per_second: Option<i32>,
    
    #[validate(range(min = 100, max = 100000))]
    pub max_text_length: Option<i32>,
    
    #[validate(range(min = 1, max = 50))]
    pub max_paragraphs_per_request: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStats {
    pub total_translations: i64,
    pub total_projects: i64,
    pub total_characters_translated: i64,
    pub average_translation_time_ms: Option<f64>,
    pub most_used_source_language: Option<String>,
    pub most_used_target_language: Option<String>,
    pub translations_this_month: i64,
    pub translations_this_week: i64,
}

/// 角色权限配置
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermissions {
    pub id: Uuid,
    pub role: UserRole,
    pub permissions: Vec<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 创建管理员用户请求
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateAdminRequest {
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    
    pub role: Option<UserRole>,
    pub permissions: Option<Vec<String>>,
}

/// 用户权限检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheck {
    pub user_id: Uuid,
    pub permission: String,
    pub granted: bool,
    pub reason: String,
}

impl User {
    /// 检查用户是否拥有指定权限
    pub fn has_permission(&self, permission: &str) -> bool {
        // 超级管理员拥有所有权限
        if self.is_admin {
            return true;
        }
        
        // 检查用户特定权限
        if self.permissions.contains(&permission.to_string()) {
            return true;
        }
        
        // 检查通配符权限
        for user_perm in &self.permissions {
            if user_perm.ends_with(":*") {
                let prefix = user_perm.trim_end_matches('*');
                if permission.starts_with(prefix) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// 检查用户是否为管理员
    pub fn is_administrator(&self) -> bool {
        self.is_admin || self.role == UserRole::Admin
    }
    
    /// 检查用户是否为管理者（包括管理员和管理者）
    pub fn is_manager(&self) -> bool {
        self.is_administrator() || self.role == UserRole::Moderator
    }
    
    /// 获取用户的所有有效权限
    pub fn get_effective_permissions(&self) -> HashSet<String> {
        let mut effective_permissions = HashSet::new();
        
        // 如果是超级管理员，返回所有权限的标识
        if self.is_admin {
            effective_permissions.insert("*".to_string());
            return effective_permissions;
        }
        
        // 添加用户特定权限
        for permission in &self.permissions {
            effective_permissions.insert(permission.clone());
        }
        
        // 根据角色添加默认权限
        let role_permissions = self.get_default_role_permissions();
        for permission in role_permissions {
            effective_permissions.insert(permission);
        }
        
        effective_permissions
    }
    
    /// 获取角色的默认权限
    pub fn get_default_role_permissions(&self) -> Vec<String> {
        match self.role {
            UserRole::Admin => vec![
                "system:read".to_string(), "system:write".to_string(), "system:config".to_string(),
                "users:read".to_string(), "users:write".to_string(), "users:delete".to_string(),
                "projects:read".to_string(), "projects:write".to_string(), "projects:delete".to_string(),
                "translations:read".to_string(), "translations:write".to_string(), "translations:delete".to_string(),
                "api_keys:read".to_string(), "api_keys:write".to_string(), "api_keys:delete".to_string(),
                "cache:clear".to_string(), "metrics:read".to_string(), "logs:read".to_string(),
            ],
            UserRole::Moderator => vec![
                "users:read".to_string(), "users:write".to_string(),
                "projects:read".to_string(), "projects:write".to_string(), "projects:delete".to_string(),
                "translations:read".to_string(), "translations:write".to_string(), "translations:delete".to_string(),
                "api_keys:read".to_string(), "api_keys:write".to_string(),
            ],
            UserRole::User => vec![
                "projects:read".to_string(), "projects:write".to_string(),
                "translations:read".to_string(), "translations:write".to_string(),
                "api_keys:read".to_string(), "api_keys:write".to_string(),
            ],
        }
    }
}

impl UserRole {
    /// 获取所有可用角色
    pub fn all() -> Vec<UserRole> {
        vec![UserRole::Admin, UserRole::Moderator, UserRole::User]
    }
    
    /// 检查角色是否有权限管理其他角色
    pub fn can_manage_role(&self, target_role: &UserRole) -> bool {
        match self {
            UserRole::Admin => true, // 管理员可以管理所有角色
            UserRole::Moderator => matches!(target_role, UserRole::User), // 管理者只能管理普通用户
            UserRole::User => false, // 普通用户不能管理其他用户
        }
    }
}

impl From<User> for UserProfile {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            is_active: user.is_active,
            role: user.role,
            is_admin: user.is_admin,
            permissions: user.permissions,
            created_at: user.created_at,
            last_login_at: user.last_login_at,
        }
    }
}

impl UserConfig {
    pub fn default_for_user(user_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            deeplx_api_url: None,
            jina_api_url: "https://r.jina.ai".to_string(),
            default_source_lang: "auto".to_string(),
            default_target_lang: "zh".to_string(),
            max_requests_per_second: 10,
            max_text_length: 5000,
            max_paragraphs_per_request: 10,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}