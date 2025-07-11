//! Admin management handlers
//! 
//! 管理员专用的API处理程序，包括用户管理、系统配置等

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;
use bcrypt::{hash, DEFAULT_COST};

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthenticatedUser,
    models::user::{CreateAdminRequest, User, UserRole, UserProfile, PermissionCheck},
    services::Services,
};

/// 创建管理员用户
pub async fn create_admin_user(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Json(request): Json<CreateAdminRequest>,
) -> AppResult<Json<UserProfile>> {
    // 检查当前用户是否有权限创建管理员
    if !user.is_admin() {
        return Err(AppError::Forbidden("Only super admins can create admin users".to_string()));
    }

    // 验证请求数据
    request.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // 检查用户是否已存在
    let existing_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 OR username = $2"
    )
    .bind(&request.email)
    .bind(&request.username)
    .fetch_optional(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("User with this email or username already exists".to_string()));
    }

    // 哈希密码
    let password_hash = hash(&request.password, DEFAULT_COST)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {}", e)))?;

    // 确定角色和权限
    let role = request.role.unwrap_or(UserRole::Admin);
    let is_admin = role == UserRole::Admin;
    let permissions = request.permissions.unwrap_or_else(|| {
        // 如果没有指定权限，根据角色给予默认权限
        match role {
            UserRole::Admin => vec![
                "system:*".to_string(),
                "users:*".to_string(),
                "projects:*".to_string(),
                "translations:*".to_string(),
                "api_keys:*".to_string(),
            ],
            UserRole::Moderator => vec![
                "users:read".to_string(),
                "users:write".to_string(),
                "projects:*".to_string(),
                "translations:*".to_string(),
            ],
            UserRole::User => vec![
                "projects:read".to_string(),
                "projects:write".to_string(),
                "translations:read".to_string(),
                "translations:write".to_string(),
            ],
        }
    });

    // 创建用户
    let user_id = Uuid::new_v4();
    let created_user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, username, email, password_hash, is_active, role, is_admin, permissions, created_at, updated_at)
        VALUES ($1, $2, $3, $4, true, $5, $6, $7, NOW(), NOW())
        RETURNING *
        "#
    )
    .bind(user_id)
    .bind(&request.username)
    .bind(&request.email)
    .bind(&password_hash)
    .bind(&role)
    .bind(is_admin)
    .bind(&permissions)
    .fetch_one(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?;

    tracing::info!(
        user_id = %user.user_id,
        new_admin_id = %created_user.id,
        role = %role,
        "Admin user created by {}", user.user.username
    );

    Ok(Json(created_user.into()))
}

/// 获取所有用户列表（管理员功能）
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub role: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub users: Vec<UserProfile>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

pub async fn list_users(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<ListUsersResponse>> {
    // 检查权限
    if !user.has_permission("users:read") {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    // 构建查询条件
    let mut where_conditions = Vec::new();
    let mut bind_values: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(role_filter) = &query.role {
        param_count += 1;
        where_conditions.push(format!("role = ${}", param_count));
        bind_values.push(Box::new(role_filter.clone()));
    }

    if let Some(search) = &query.search {
        param_count += 1;
        where_conditions.push(format!("(username ILIKE ${} OR email ILIKE ${})", param_count, param_count));
        bind_values.push(Box::new(format!("%{}%", search)));
    }

    let where_clause = if where_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    // 查询用户总数
    let total_query = format!("SELECT COUNT(*) FROM users {}", where_clause);
    let total: i64 = sqlx::query_scalar(&total_query)
        .fetch_one(services.db.pool())
        .await
        .map_err(|e| AppError::Database(e))?;

    // 查询用户列表
    param_count += 1;
    let limit_param = param_count;
    param_count += 1;
    let offset_param = param_count;

    let users_query = format!(
        "SELECT * FROM users {} ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause, limit_param, offset_param
    );

    let users: Vec<User> = sqlx::query_as(&users_query)
        .bind(per_page as i64)
        .bind(offset as i64)
        .fetch_all(services.db.pool())
        .await
        .map_err(|e| AppError::Database(e))?;

    let user_profiles: Vec<UserProfile> = users.into_iter().map(Into::into).collect();
    let total_pages = ((total as f64) / (per_page as f64)).ceil() as u32;

    Ok(Json(ListUsersResponse {
        users: user_profiles,
        total,
        page,
        per_page,
        total_pages,
    }))
}

/// 更新用户角色和权限
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRoleRequest {
    pub role: Option<UserRole>,
    pub is_admin: Option<bool>,
    pub permissions: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

pub async fn update_user_role(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(target_user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> AppResult<Json<UserProfile>> {
    // 检查权限
    if !user.has_permission("users:write") {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    // 验证请求数据
    request.validate()
        .map_err(|e| AppError::BadRequest(format!("Validation error: {}", e)))?;

    // 防止用户修改自己的权限
    if user.user_id == target_user_id {
        return Err(AppError::BadRequest("Cannot modify your own permissions".to_string()));
    }

    // 获取目标用户
    let _target_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(target_user_id)
    .fetch_optional(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // 检查角色管理权限
    if let Some(new_role) = &request.role {
        if !user.role().can_manage_role(new_role) && !user.is_admin() {
            return Err(AppError::Forbidden("Insufficient privileges to assign this role".to_string()));
        }
    }

    // 构建更新查询
    let mut set_clauses = Vec::new();
    let mut bind_values: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send + Sync>> = Vec::new();
    let mut param_count = 0;

    if let Some(role) = &request.role {
        param_count += 1;
        set_clauses.push(format!("role = ${}", param_count));
        bind_values.push(Box::new(role.clone()));
    }

    if let Some(is_admin) = request.is_admin {
        param_count += 1;
        set_clauses.push(format!("is_admin = ${}", param_count));
        bind_values.push(Box::new(is_admin));
    }

    if let Some(permissions) = &request.permissions {
        param_count += 1;
        set_clauses.push(format!("permissions = ${}", param_count));
        bind_values.push(Box::new(permissions.clone()));
    }

    if let Some(is_active) = request.is_active {
        param_count += 1;
        set_clauses.push(format!("is_active = ${}", param_count));
        bind_values.push(Box::new(is_active));
    }

    if set_clauses.is_empty() {
        return Err(AppError::BadRequest("No fields to update".to_string()));
    }

    param_count += 1;
    set_clauses.push(format!("updated_at = ${}", param_count));
    bind_values.push(Box::new(chrono::Utc::now()));

    param_count += 1;
    let user_id_param = param_count;

    let update_query = format!(
        "UPDATE users SET {} WHERE id = ${} RETURNING *",
        set_clauses.join(", "),
        user_id_param
    );

    let updated_user: User = sqlx::query_as(&update_query)
        .bind(target_user_id)
        .fetch_one(services.db.pool())
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::info!(
        admin_id = %user.user_id,
        target_user_id = %target_user_id,
        "User role/permissions updated"
    );

    Ok(Json(updated_user.into()))
}

/// 检查用户权限
pub async fn check_user_permission(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path((target_user_id, permission)): Path<(Uuid, String)>,
) -> AppResult<Json<PermissionCheck>> {
    // 检查权限
    if !user.has_permission("users:read") && user.user_id != target_user_id {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    // 获取目标用户
    let target_user = if user.user_id == target_user_id {
        user.user.clone()
    } else {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(target_user_id)
            .fetch_optional(services.db.pool())
            .await
            .map_err(|e| AppError::Database(e))?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?
    };

    let has_permission = target_user.has_permission(&permission);
    let reason = if target_user.is_admin {
        "Super admin has all permissions".to_string()
    } else if has_permission {
        "Permission granted by role or user-specific permissions".to_string()
    } else {
        "Permission not granted".to_string()
    };

    Ok(Json(PermissionCheck {
        user_id: target_user_id,
        permission,
        granted: has_permission,
        reason,
    }))
}

/// 删除用户（仅超级管理员）
pub async fn delete_user(
    State(services): State<Services>,
    user: AuthenticatedUser,
    Path(target_user_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    // 只有超级管理员可以删除用户
    if !user.is_admin() {
        return Err(AppError::Forbidden("Only super admins can delete users".to_string()));
    }

    // 防止用户删除自己
    if user.user_id == target_user_id {
        return Err(AppError::BadRequest("Cannot delete yourself".to_string()));
    }

    // 检查用户是否存在
    let target_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
    .bind(target_user_id)
    .fetch_optional(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    // 删除用户
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(target_user_id)
        .execute(services.db.pool())
        .await
        .map_err(|e| AppError::Database(e))?;

    tracing::warn!(
        admin_id = %user.user_id,
        deleted_user_id = %target_user_id,
        deleted_username = %target_user.username,
        "User deleted by admin"
    );

    Ok(StatusCode::NO_CONTENT)
}