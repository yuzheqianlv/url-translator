//! Database migration handlers
//! 
//! 用于执行数据库迁移的临时API处理程序

use axum::{
    extract::State,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    services::Services,
};

#[derive(Debug, Serialize)]
pub struct MigrationResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

/// 执行角色和权限迁移
pub async fn migrate_user_roles(
    State(services): State<Services>,
) -> AppResult<Json<MigrationResult>> {
    tracing::info!("开始执行用户角色和权限迁移");

    let mut transaction = services.db.pool().begin().await
        .map_err(|e| AppError::Database(e))?;

    let mut migration_steps = Vec::new();

    // 步骤1: 创建用户角色枚举类型
    match sqlx::query("CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user')")
        .execute(&mut *transaction)
        .await
    {
        Ok(_) => {
            migration_steps.push("✅ 创建用户角色枚举类型".to_string());
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("already exists") {
                migration_steps.push("⚠️  用户角色枚举类型已存在，跳过".to_string());
            } else {
                return Err(AppError::Database(e));
            }
        }
    }

    // 步骤2: 检查并添加role列
    let role_column_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'users' AND column_name = 'role'
        )"
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| AppError::Database(e))?;

    if !role_column_exists {
        sqlx::query("ALTER TABLE users ADD COLUMN role user_role DEFAULT 'user' NOT NULL")
            .execute(&mut *transaction)
            .await
            .map_err(|e| AppError::Database(e))?;
        migration_steps.push("✅ 添加role列".to_string());
    } else {
        migration_steps.push("⚠️  role列已存在，跳过".to_string());
    }

    // 步骤3: 检查并添加is_admin列
    let is_admin_column_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'users' AND column_name = 'is_admin'
        )"
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| AppError::Database(e))?;

    if !is_admin_column_exists {
        sqlx::query("ALTER TABLE users ADD COLUMN is_admin BOOLEAN DEFAULT FALSE NOT NULL")
            .execute(&mut *transaction)
            .await
            .map_err(|e| AppError::Database(e))?;
        migration_steps.push("✅ 添加is_admin列".to_string());
    } else {
        migration_steps.push("⚠️  is_admin列已存在，跳过".to_string());
    }

    // 步骤4: 检查并添加permissions列
    let permissions_column_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.columns 
            WHERE table_name = 'users' AND column_name = 'permissions'
        )"
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(|e| AppError::Database(e))?;

    if !permissions_column_exists {
        sqlx::query("ALTER TABLE users ADD COLUMN permissions TEXT[] DEFAULT '{}' NOT NULL")
            .execute(&mut *transaction)
            .await
            .map_err(|e| AppError::Database(e))?;
        migration_steps.push("✅ 添加permissions列".to_string());
    } else {
        migration_steps.push("⚠️  permissions列已存在，跳过".to_string());
    }

    // 步骤5: 创建角色权限表
    match sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS role_permissions (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            role user_role NOT NULL UNIQUE,
            permissions TEXT[] NOT NULL,
            description TEXT,
            created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
        )
        "#
    )
    .execute(&mut *transaction)
    .await
    {
        Ok(_) => {
            migration_steps.push("✅ 创建角色权限表".to_string());
        }
        Err(e) => {
            return Err(AppError::Database(e));
        }
    }

    // 步骤6: 插入默认角色权限
    let role_permissions_data = vec![
        (
            "admin",
            vec![
                "system:read", "system:write", "system:config", "system:stats",
                "users:read", "users:write", "users:delete", "users:manage",
                "projects:read", "projects:write", "projects:delete", "projects:manage",
                "translations:read", "translations:write", "translations:delete", "translations:manage",
                "api_keys:read", "api_keys:write", "api_keys:delete", "api_keys:manage",
                "cache:clear", "metrics:read", "logs:read"
            ],
            "系统管理员，拥有所有权限"
        ),
        (
            "moderator",
            vec![
                "users:read", "users:write",
                "projects:read", "projects:write", "projects:delete",
                "translations:read", "translations:write", "translations:delete",
                "api_keys:read", "api_keys:write"
            ],
            "内容管理员，拥有用户和内容管理权限"
        ),
        (
            "user",
            vec![
                "projects:read", "projects:write",
                "translations:read", "translations:write",
                "api_keys:read", "api_keys:write"
            ],
            "普通用户，拥有基本功能权限"
        ),
    ];

    for (role, permissions, description) in role_permissions_data {
        match sqlx::query(
            "INSERT INTO role_permissions (role, permissions, description) VALUES ($1, $2, $3) ON CONFLICT (role) DO NOTHING"
        )
        .bind(role)
        .bind(&permissions)
        .bind(description)
        .execute(&mut *transaction)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    migration_steps.push(format!("✅ 插入{}角色权限", role));
                } else {
                    migration_steps.push(format!("⚠️  {}角色权限已存在，跳过", role));
                }
            }
            Err(e) => {
                return Err(AppError::Database(e));
            }
        }
    }

    // 提交事务
    transaction.commit().await
        .map_err(|e| AppError::Database(e))?;

    let details = migration_steps.join("\n");
    tracing::info!("用户角色和权限迁移完成: {}", details);

    Ok(Json(MigrationResult {
        success: true,
        message: "用户角色和权限迁移完成".to_string(),
        details: Some(details),
    }))
}

/// 升级指定用户为管理员
#[derive(Debug, Deserialize)]
pub struct UpgradeAdminRequest {
    pub user_id: Uuid,
}

pub async fn upgrade_user_to_admin(
    State(services): State<Services>,
    Json(request): Json<UpgradeAdminRequest>,
) -> AppResult<Json<MigrationResult>> {
    tracing::info!(user_id = %request.user_id, "开始升级用户为管理员");

    // 检查用户是否存在
    let user_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)"
    )
    .bind(request.user_id)
    .fetch_one(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?;

    if !user_exists {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    // 升级用户为管理员
    let admin_permissions = vec![
        "system:*".to_string(),
        "users:*".to_string(),
        "projects:*".to_string(),
        "translations:*".to_string(),
        "api_keys:*".to_string(),
    ];

    let updated_rows = sqlx::query(
        r#"
        UPDATE users SET 
            role = 'admin',
            is_admin = TRUE,
            permissions = $2,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        "#
    )
    .bind(request.user_id)
    .bind(&admin_permissions)
    .execute(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?
    .rows_affected();

    if updated_rows == 0 {
        return Err(AppError::Internal("Failed to update user permissions".to_string()));
    }

    tracing::info!(
        user_id = %request.user_id,
        permissions = ?admin_permissions,
        "用户成功升级为管理员"
    );

    Ok(Json(MigrationResult {
        success: true,
        message: format!("用户 {} 已成功升级为管理员", request.user_id),
        details: Some(format!("权限: {:?}", admin_permissions)),
    }))
}

/// 检查数据库迁移状态
pub async fn check_migration_status(
    State(services): State<Services>,
) -> AppResult<Json<MigrationResult>> {
    let mut status_checks = Vec::new();

    // 检查user_role枚举类型
    let role_enum_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'user_role')"
    )
    .fetch_one(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?;

    status_checks.push(format!("user_role枚举: {}", if role_enum_exists { "✅" } else { "❌" }));

    // 检查新增的列
    let columns_to_check = vec!["role", "is_admin", "permissions"];
    for column in columns_to_check {
        let column_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.columns 
                WHERE table_name = 'users' AND column_name = $1
            )"
        )
        .bind(column)
        .fetch_one(services.db.pool())
        .await
        .map_err(|e| AppError::Internal(format!("Failed to check {} column: {}", column, e)))?;

        status_checks.push(format!("{}列: {}", column, if column_exists { "✅" } else { "❌" }));
    }

    // 检查role_permissions表
    let role_permissions_table_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (
            SELECT 1 FROM information_schema.tables 
            WHERE table_name = 'role_permissions'
        )"
    )
    .fetch_one(services.db.pool())
    .await
    .map_err(|e| AppError::Database(e))?;

    status_checks.push(format!("role_permissions表: {}", if role_permissions_table_exists { "✅" } else { "❌" }));

    // 检查管理员用户数量
    if role_enum_exists {
        let admin_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE is_admin = TRUE"
        )
        .fetch_one(services.db.pool())
        .await
        .unwrap_or(0);

        status_checks.push(format!("管理员用户数量: {}", admin_count));
    }

    let all_migrated = role_enum_exists && role_permissions_table_exists;

    Ok(Json(MigrationResult {
        success: all_migrated,
        message: if all_migrated { "数据库迁移已完成" } else { "数据库迁移未完成" }.to_string(),
        details: Some(status_checks.join("\n")),
    }))
}