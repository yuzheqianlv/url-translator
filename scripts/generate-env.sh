#!/bin/bash

# URL翻译工具 - 环境变量生成脚本
# 用于生成安全的随机密码和配置文件

set -e

ECHO_PREFIX="🔐 [ENV-GEN]"
ENV_FILE=".env"

echo "$ECHO_PREFIX 开始生成安全的环境配置文件..."

# 检查是否已存在 .env 文件
if [ -f "$ENV_FILE" ]; then
    echo "$ECHO_PREFIX 警告: $ENV_FILE 文件已存在！"
    read -p "$ECHO_PREFIX 是否覆盖现有文件? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "$ECHO_PREFIX 取消生成。"
        exit 0
    fi
fi

# 生成随机密码的函数
generate_password() {
    local length=${1:-32}
    if command -v openssl >/dev/null 2>&1; then
        openssl rand -base64 $length | tr -d "=+/" | cut -c1-$length
    elif command -v pwgen >/dev/null 2>&1; then
        pwgen -s $length 1
    else
        # 使用 /dev/urandom 作为后备
        cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w $length | head -n 1
    fi
}

# 生成 UUID
generate_uuid() {
    if command -v uuidgen >/dev/null 2>&1; then
        uuidgen
    elif command -v python3 >/dev/null 2>&1; then
        python3 -c "import uuid; print(str(uuid.uuid4()))"
    else
        # 简单的伪 UUID
        cat /dev/urandom | tr -dc 'a-f0-9' | fold -w 8 | head -n 1
    fi
}

echo "$ECHO_PREFIX 正在生成安全的随机密码..."

# 生成各种密码
POSTGRES_PASSWORD=$(generate_password 32)
MEILI_MASTER_KEY=$(generate_password 32)
REDIS_PASSWORD=$(generate_password 32)
JWT_SECRET=$(generate_password 64)

echo "$ECHO_PREFIX 正在创建 $ENV_FILE 文件..."

# 创建 .env 文件
cat > "$ENV_FILE" << EOF
# ============================================================================= 
# URL翻译工具 - 环境配置文件
# 自动生成于: $(date)
# 警告: 请勿将此文件提交到版本控制系统！
# =============================================================================

# 应用端口 (Docker内部始终为80，此处为外部映射端口)
APP_PORT=3001

# DeepLX API配置 (请替换为您的实际API地址)
DEEPLX_API_URL=http://localhost:1188/translate

# Jina AI Reader配置
JINA_API_URL=https://r.jina.ai

# 速率限制配置
MAX_REQUESTS_PER_SECOND=10
MAX_TEXT_LENGTH=5000
MAX_PARAGRAPHS_PER_REQUEST=10

# 默认语言设置
DEFAULT_SOURCE_LANG=auto
DEFAULT_TARGET_LANG=zh

# =============================================================================
# 外部服务配置 - External Services Configuration
# =============================================================================

# PostgreSQL 数据库配置
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
POSTGRES_DB=markdown_manager
POSTGRES_USER=admin

# MeiliSearch 搜索引擎配置
MEILI_MASTER_KEY=$MEILI_MASTER_KEY
MEILI_ENV=production

# Redis 缓存配置
REDIS_PASSWORD=$REDIS_PASSWORD

# JWT 密钥配置 (用于用户认证)
JWT_SECRET=$JWT_SECRET

# 后端API配置
BACKEND_API_URL=http://localhost:3002
API_RATE_LIMIT=100

# =============================================================================
# 数据库连接字符串 - Database Connection Strings
# =============================================================================

# 完整的PostgreSQL连接字符串
DATABASE_URL=postgres://admin:$POSTGRES_PASSWORD@localhost:5432/markdown_manager

# Redis连接字符串
REDIS_URL=redis://:$REDIS_PASSWORD@localhost:6379

# MeiliSearch连接配置
MEILISEARCH_URL=http://localhost:7700
MEILISEARCH_API_KEY=$MEILI_MASTER_KEY

# =============================================================================
# 安全注意事项 - Security Notes
# =============================================================================
# 1. 请勿将此文件提交到版本控制系统
# 2. 在生产环境中请使用更强的密码
# 3. 定期轮换密码和密钥
# 4. 确保数据库和服务的网络安全配置
EOF

echo "$ECHO_PREFIX ✅ 环境配置文件已生成: $ENV_FILE"
echo "$ECHO_PREFIX 🔐 密码信息:"
echo "$ECHO_PREFIX   PostgreSQL: $POSTGRES_PASSWORD"
echo "$ECHO_PREFIX   MeiliSearch: $MEILI_MASTER_KEY"
echo "$ECHO_PREFIX   Redis: $REDIS_PASSWORD"
echo "$ECHO_PREFIX   JWT Secret: ${JWT_SECRET:0:16}..."
echo ""
echo "$ECHO_PREFIX ⚠️  重要提示:"
echo "$ECHO_PREFIX   1. 请将 $ENV_FILE 文件保存在安全的地方"
echo "$ECHO_PREFIX   2. 请勿将此文件提交到Git仓库"
echo "$ECHO_PREFIX   3. 请修改 DEEPLX_API_URL 为您的实际API地址"
echo "$ECHO_PREFIX   4. 可以现在运行: docker-compose up -d"
echo ""
echo "$ECHO_PREFIX 🎉 环境配置完成！"