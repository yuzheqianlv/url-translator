#!/bin/bash

# 管理员账号设置脚本
# 通过后端API创建管理员账号并设置数据库结构

set -e

echo "🚀 开始设置管理员账号..."

# 后端API地址
API_BASE="http://localhost:3002/api/v1"

# 管理员账号信息
ADMIN_USERNAME="admin"
ADMIN_EMAIL="admin@url-translator.local"
ADMIN_PASSWORD="admin123456"

echo "📡 检查后端服务状态..."
if ! curl -s "${API_BASE%/api/v1}/health" > /dev/null; then
    echo "❌ 后端服务未运行，请先启动后端服务"
    echo "   cd backend && cargo run"
    exit 1
fi

echo "✅ 后端服务运行正常"

echo "👤 尝试注册管理员用户..."

# 先尝试注册普通用户
REGISTER_RESPONSE=$(curl -s -w "HTTPSTATUS:%{http_code}" -X POST "${API_BASE}/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"${ADMIN_USERNAME}\",
    \"email\": \"${ADMIN_EMAIL}\",
    \"password\": \"${ADMIN_PASSWORD}\"
  }")

# 提取HTTP状态码
HTTP_STATUS=$(echo $REGISTER_RESPONSE | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
RESPONSE_BODY=$(echo $REGISTER_RESPONSE | sed -e 's/HTTPSTATUS\:.*//g')

if [ "$HTTP_STATUS" -eq 200 ] || [ "$HTTP_STATUS" -eq 201 ]; then
    echo "✅ 管理员用户注册成功"
    echo "📄 响应: $RESPONSE_BODY"
    
    # 提取用户ID
    USER_ID=$(echo $RESPONSE_BODY | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
    echo "🆔 用户ID: $USER_ID"
    
elif [ "$HTTP_STATUS" -eq 409 ]; then
    echo "⚠️  用户已存在，尝试登录获取用户ID..."
    
    # 尝试登录获取token
    LOGIN_RESPONSE=$(curl -s -w "HTTPSTATUS:%{http_code}" -X POST "${API_BASE}/auth/login" \
      -H "Content-Type: application/json" \
      -d "{
        \"email\": \"${ADMIN_EMAIL}\",
        \"password\": \"${ADMIN_PASSWORD}\"
      }")
    
    LOGIN_HTTP_STATUS=$(echo $LOGIN_RESPONSE | tr -d '\n' | sed -e 's/.*HTTPSTATUS://')
    LOGIN_BODY=$(echo $LOGIN_RESPONSE | sed -e 's/HTTPSTATUS\:.*//g')
    
    if [ "$LOGIN_HTTP_STATUS" -eq 200 ]; then
        echo "✅ 登录成功"
        USER_ID=$(echo $LOGIN_BODY | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
        echo "🆔 用户ID: $USER_ID"
    else
        echo "❌ 登录失败: $LOGIN_BODY"
        exit 1
    fi
else
    echo "❌ 注册失败 (HTTP $HTTP_STATUS): $RESPONSE_BODY"
    exit 1
fi

echo ""
echo "🎯 管理员账号信息:"
echo "   用户名: $ADMIN_USERNAME"
echo "   邮箱: $ADMIN_EMAIL"
echo "   密码: $ADMIN_PASSWORD"
echo "   用户ID: $USER_ID"
echo ""

echo "⚠️  请注意:"
echo "   1. 由于数据库迁移尚未完成，当前用户还不具有管理员权限"
echo "   2. 需要手动执行数据库迁移来添加角色和权限字段"
echo "   3. 完成迁移后，可以通过数据库直接更新用户权限"
echo ""

echo "📋 后续步骤:"
echo "   1. 停止后端服务"
echo "   2. 执行以下SQL命令来升级数据库:"
echo ""
echo "   -- 添加角色枚举类型"
echo "   CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');"
echo ""
echo "   -- 添加角色和权限字段"
echo "   ALTER TABLE users"
echo "   ADD COLUMN role user_role DEFAULT 'user' NOT NULL,"
echo "   ADD COLUMN is_admin BOOLEAN DEFAULT FALSE NOT NULL,"
echo "   ADD COLUMN permissions TEXT[] DEFAULT '{}' NOT NULL;"
echo ""
echo "   -- 升级管理员用户"
echo "   UPDATE users SET"
echo "     role = 'admin',"
echo "     is_admin = TRUE,"
echo "     permissions = ARRAY['system:*', 'users:*', 'projects:*', 'translations:*']"
echo "   WHERE id = '$USER_ID';"
echo ""

echo "✅ 管理员账号设置完成!"
echo "🔐 请安全保存管理员凭据，并完成数据库迁移。"