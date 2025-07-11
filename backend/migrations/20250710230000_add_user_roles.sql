-- 为用户系统添加角色和权限管理
-- 创建时间: 2025-07-10 23:30:00

-- 添加用户角色枚举类型
CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');

-- 为users表添加角色相关字段
ALTER TABLE users 
ADD COLUMN role user_role DEFAULT 'user' NOT NULL,
ADD COLUMN is_admin BOOLEAN DEFAULT FALSE NOT NULL,
ADD COLUMN permissions TEXT[] DEFAULT '{}' NOT NULL;

-- 创建角色权限预设表
CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role user_role NOT NULL UNIQUE,
    permissions TEXT[] NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 插入默认角色权限配置
INSERT INTO role_permissions (role, permissions, description) VALUES 
(
    'admin', 
    ARRAY[
        'system:read', 'system:write', 'system:config', 'system:stats',
        'users:read', 'users:write', 'users:delete', 'users:manage',
        'projects:read', 'projects:write', 'projects:delete', 'projects:manage',
        'translations:read', 'translations:write', 'translations:delete', 'translations:manage',
        'api_keys:read', 'api_keys:write', 'api_keys:delete', 'api_keys:manage',
        'cache:clear', 'metrics:read', 'logs:read'
    ],
    '系统管理员，拥有所有权限'
),
(
    'moderator',
    ARRAY[
        'users:read', 'users:write',
        'projects:read', 'projects:write', 'projects:delete',
        'translations:read', 'translations:write', 'translations:delete',
        'api_keys:read', 'api_keys:write'
    ],
    '内容管理员，拥有用户和内容管理权限'
),
(
    'user',
    ARRAY[
        'projects:read', 'projects:write',
        'translations:read', 'translations:write',
        'api_keys:read', 'api_keys:write'
    ],
    '普通用户，拥有基本功能权限'
);

-- 创建索引提升查询性能
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_is_admin ON users(is_admin);
CREATE INDEX idx_users_permissions ON users USING GIN(permissions);

-- 更新触发器以自动维护 updated_at
CREATE OR REPLACE FUNCTION update_role_permissions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_role_permissions_updated_at
    BEFORE UPDATE ON role_permissions
    FOR EACH ROW
    EXECUTE FUNCTION update_role_permissions_updated_at();

-- 创建函数用于检查用户权限
CREATE OR REPLACE FUNCTION user_has_permission(user_uuid UUID, permission_name TEXT)
RETURNS BOOLEAN AS $$
DECLARE
    user_permissions TEXT[];
    role_permissions TEXT[];
    user_is_admin BOOLEAN;
BEGIN
    -- 获取用户的基本信息
    SELECT u.permissions, rp.permissions, u.is_admin
    INTO user_permissions, role_permissions, user_is_admin
    FROM users u
    LEFT JOIN role_permissions rp ON u.role = rp.role
    WHERE u.id = user_uuid;
    
    -- 如果用户不存在，返回false
    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;
    
    -- 管理员拥有所有权限
    IF user_is_admin THEN
        RETURN TRUE;
    END IF;
    
    -- 检查用户特定权限
    IF permission_name = ANY(user_permissions) THEN
        RETURN TRUE;
    END IF;
    
    -- 检查角色权限
    IF permission_name = ANY(role_permissions) THEN
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- 创建视图以便查看用户的完整权限信息
CREATE VIEW user_permissions_view AS
SELECT 
    u.id,
    u.username,
    u.email,
    u.role,
    u.is_admin,
    u.permissions as user_specific_permissions,
    rp.permissions as role_permissions,
    CASE 
        WHEN u.is_admin THEN ARRAY['*'] -- 管理员拥有所有权限
        ELSE array_cat(u.permissions, rp.permissions)
    END as effective_permissions,
    u.created_at,
    u.last_login_at
FROM users u
LEFT JOIN role_permissions rp ON u.role = rp.role;

-- 添加注释
COMMENT ON TABLE role_permissions IS '角色权限配置表';
COMMENT ON COLUMN users.role IS '用户角色: admin(管理员), moderator(管理者), user(普通用户)';
COMMENT ON COLUMN users.is_admin IS '是否为超级管理员，超级管理员拥有所有权限';
COMMENT ON COLUMN users.permissions IS '用户特定权限数组，补充角色权限';
COMMENT ON FUNCTION user_has_permission IS '检查用户是否拥有指定权限';
COMMENT ON VIEW user_permissions_view IS '用户权限信息视图，包含有效权限列表';