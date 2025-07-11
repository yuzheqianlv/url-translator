#!/bin/bash
# 生产环境构建脚本

set -e

echo "🚀 开始生产环境构建..."

# 检查依赖
echo "📋 检查构建依赖..."
if ! command -v trunk &> /dev/null; then
    echo "❌ Trunk 未安装，请运行: cargo install trunk"
    exit 1
fi

if ! rustup target list --installed | grep -q wasm32-unknown-unknown; then
    echo "📦 安装 WASM 目标..."
    rustup target add wasm32-unknown-unknown
fi

# 设置环境变量
echo "⚙️ 设置生产环境变量..."
export PRODUCTION_MODE=true
export DEBUG_MODE=false
export RUST_LOG=warn
export ENABLE_DEVTOOLS=false
export ENABLE_COMPRESSION=true
export ENABLE_MINIFICATION=true

# 加载生产环境配置
if [ -f ".env.production" ]; then
    echo "📄 加载生产环境配置..."
    export $(cat .env.production | grep -v '^#' | xargs)
fi

# 清理之前的构建
echo "🧹 清理之前的构建..."
trunk clean
rm -rf dist/

# 运行代码检查
echo "🔍 运行代码检查..."
cargo clippy --target wasm32-unknown-unknown -- -D warnings

# 运行测试
echo "🧪 运行测试..."
cargo test --lib

# 生产构建
echo "🏗️ 开始生产构建..."
trunk build --release

# 优化构建产物
echo "⚡ 优化构建产物..."

# 压缩 WASM 文件
if command -v wasm-opt &> /dev/null; then
    echo "🗜️ 压缩 WASM 文件..."
    find dist -name "*.wasm" -exec wasm-opt -Oz {} -o {} \;
else
    echo "⚠️ wasm-opt 未找到，跳过 WASM 优化"
fi

# 生成文件清单
echo "📝 生成文件清单..."
find dist -type f -exec ls -lh {} \; > dist/file-manifest.txt

# 计算总大小
echo "📊 构建统计:"
echo "- 总文件数: $(find dist -type f | wc -l)"
echo "- 总大小: $(du -sh dist | cut -f1)"
echo "- WASM 文件: $(find dist -name "*.wasm" -exec du -sh {} \; | cut -f1)"
echo "- JS 文件: $(find dist -name "*.js" -exec du -sh {} \; | cut -f1)"

# 验证构建
echo "✅ 验证构建产物..."
if [ ! -f "dist/index.html" ]; then
    echo "❌ 构建失败: index.html 未找到"
    exit 1
fi

if [ ! -f "dist/style.css" ]; then
    echo "⚠️ 警告: style.css 未找到"
fi

# 生成部署信息
echo "📋 生成部署信息..."
cat > dist/build-info.json << EOF
{
    "buildTime": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
    "version": "$(grep '^version' Cargo.toml | cut -d'"' -f2)",
    "gitHash": "$(git rev-parse --short HEAD 2>/dev/null || echo 'unknown')",
    "buildMode": "production",
    "rustVersion": "$(rustc --version)",
    "trunkVersion": "$(trunk --version)"
}
EOF

echo "🎉 生产构建完成!"
echo "📁 构建产物位于 dist/ 目录"
echo "🌐 可以将 dist/ 目录部署到任何静态文件服务器"

# 可选：自动部署
if [ "$AUTO_DEPLOY" = "true" ]; then
    echo "🚀 开始自动部署..."
    if [ -f "./scripts/deploy.sh" ]; then
        ./scripts/deploy.sh
    else
        echo "⚠️ 部署脚本未找到，跳过自动部署"
    fi
fi