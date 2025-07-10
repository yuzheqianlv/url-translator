#!/bin/bash
# 开发环境启动脚本

set -e

echo "🛠️ 启动开发环境..."

# 检查依赖
echo "📋 检查开发依赖..."
if ! command -v trunk &> /dev/null; then
    echo "❌ Trunk 未安装，请运行: cargo install trunk"
    exit 1
fi

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "📦 安装 WASM 目标..."
    rustup target add wasm32-unknown-unknown
fi

# 设置开发环境变量
echo "⚙️ 设置开发环境变量..."
export DEBUG_MODE=true
export RUST_LOG=debug
export WASM_LOG=debug
export ENABLE_DEVTOOLS=true
export HOT_RELOAD=true
export FRONTEND_PORT=3001

# 加载开发环境配置
if [ -f ".env" ]; then
    echo "📄 加载开发环境配置..."
    export $(cat .env | grep -v '^#' | xargs)
elif [ -f ".env.example" ]; then
    echo "⚠️ 未找到 .env 文件，请复制 .env.example 到 .env 并修改配置"
    echo "💡 运行: cp .env.example .env"
fi

# 检查后端服务
echo "🔍 检查后端服务状态..."
BACKEND_URL="${FRONTEND_API_BASE_URL:-http://localhost:3002}"
if curl -s "${BACKEND_URL}/health" > /dev/null 2>&1; then
    echo "✅ 后端服务运行正常"
else
    echo "⚠️ 后端服务未运行或无法访问: $BACKEND_URL"
    echo "💡 请确保后端服务已启动，或查看后端启动指南"
fi

# 清理之前的构建（可选）
if [ "$CLEAN_BUILD" = "true" ]; then
    echo "🧹 清理之前的构建..."
    trunk clean
    rm -rf dist/
fi

# 运行快速检查
echo "🔍 运行快速代码检查..."
if ! cargo check --target wasm32-unknown-unknown; then
    echo "❌ 代码检查失败，请修复错误后重试"
    exit 1
fi

# 显示启动信息
echo ""
echo "🎯 开发环境配置:"
echo "- 前端端口: ${FRONTEND_PORT}"
echo "- 后端API: ${BACKEND_URL}"
echo "- 调试模式: ${DEBUG_MODE:-true}"
echo "- 热重载: ${HOT_RELOAD:-true}"
echo ""

# 启动开发服务器
echo "🚀 启动前端开发服务器..."
echo "📱 浏览器将自动打开 http://localhost:${FRONTEND_PORT}"
echo "⏹️ 按 Ctrl+C 停止服务器"
echo ""

# 使用 trunk serve 启动开发服务器
trunk serve \
    --port ${FRONTEND_PORT} \
    --open \
    --watch src \
    --watch index.html \
    --watch style.css