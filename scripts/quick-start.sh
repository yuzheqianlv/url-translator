#!/bin/bash

# URL翻译工具 - 快速启动脚本
# 一键设置和启动所有服务

set -e

ECHO_PREFIX="🚀 [QUICK-START]"
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "$ECHO_PREFIX 欢迎使用 URL翻译工具快速启动向导！"
echo "$ECHO_PREFIX 项目目录: $PROJECT_ROOT"

cd "$PROJECT_ROOT"

# 检查依赖
check_dependencies() {
    echo "$ECHO_PREFIX 检查系统依赖..."
    
    local missing_deps=()
    
    if ! command -v docker >/dev/null 2>&1; then
        missing_deps+=("docker")
    fi
    
    if ! command -v docker-compose >/dev/null 2>&1; then
        missing_deps+=("docker-compose")
    fi
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        echo "$ECHO_PREFIX ❗ 缺少以下依赖: ${missing_deps[*]}"
        echo "$ECHO_PREFIX 请先安装 Docker 和 Docker Compose"
        exit 1
    fi
    
    echo "$ECHO_PREFIX ✅ 依赖检查通过"
}

# 生成环境配置
generate_env_config() {
    if [ ! -f ".env" ]; then
        echo "$ECHO_PREFIX 生成环境配置文件..."
        if [ -f "scripts/generate-env.sh" ]; then
            ./scripts/generate-env.sh
        else
            echo "$ECHO_PREFIX ❗ 找不到 generate-env.sh 脚本"
            echo "$ECHO_PREFIX 请手动复制 .env.example 为 .env 并修改配置"
            exit 1
        fi
    else
        echo "$ECHO_PREFIX .env 文件已存在，跳过生成"
    fi
}

# 检查配置
check_config() {
    echo "$ECHO_PREFIX 检查配置文件..."
    
    if ! grep -q "DEEPLX_API_URL=http://localhost:1188" .env; then
        echo "$ECHO_PREFIX ⚠️  检测到默认的 DeepLX API 地址"
        echo "$ECHO_PREFIX 请确保更新 .env 文件中的 DEEPLX_API_URL 为您的实际API地址"
        read -p "$ECHO_PREFIX 是否继续? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "$ECHO_PREFIX 请修改 .env 文件后重新运行此脚本"
            exit 0
        fi
    fi
}

# 构建和启动服务
start_services() {
    echo "$ECHO_PREFIX 正在构建和启动服务..."
    
    # 停止可能存在的旧容器
    echo "$ECHO_PREFIX 清理旧容器..."
    docker-compose down --remove-orphans || true
    
    # 启动数据库服务
    echo "$ECHO_PREFIX 启动数据库服务..."
    docker-compose up -d postgres redis meilisearch
    
    # 等待数据库启动
    echo "$ECHO_PREFIX 等待数据库服务启动..."
    sleep 10
    
    # 检查数据库状态
    echo "$ECHO_PREFIX 检查服务状态..."
    docker-compose ps
    
    # 注意: 后端服务需要单独开发，这里先不启动
    echo "$ECHO_PREFIX 注意: 后端API服务需要单独开发，当前只启动了数据库服务"
}

# 检查服务状态
check_services() {
    echo "$ECHO_PREFIX 检查服务健康状态..."
    
    # 检查 PostgreSQL
    if docker-compose exec -T postgres pg_isready -U admin -d markdown_manager >/dev/null 2>&1; then
        echo "$ECHO_PREFIX ✅ PostgreSQL 数据库正常运行"
    else
        echo "$ECHO_PREFIX ❌ PostgreSQL 数据库连接失败"
    fi
    
    # 检查 Redis
    if docker-compose exec -T redis redis-cli ping >/dev/null 2>&1; then
        echo "$ECHO_PREFIX ✅ Redis 缓存正常运行"
    else
        echo "$ECHO_PREFIX ❌ Redis 缓存连接失败"
    fi
    
    # 检查 MeiliSearch
    if curl -s http://localhost:7700/health >/dev/null 2>&1; then
        echo "$ECHO_PREFIX ✅ MeiliSearch 搜索引擎正常运行"
    else
        echo "$ECHO_PREFIX ❌ MeiliSearch 搜索引擎连接失败"
    fi
}

# 显示使用信息
show_usage_info() {
    echo ""
    echo "$ECHO_PREFIX 🎉 服务启动完成！"
    echo ""
    echo "$ECHO_PREFIX 🔗 服务地址:"
    echo "$ECHO_PREFIX   PostgreSQL: localhost:5432 (admin/[查看.env文件])"
    echo "$ECHO_PREFIX   Redis:      localhost:6379 ([查看.env文件])"
    echo "$ECHO_PREFIX   MeiliSearch: http://localhost:7700"
    echo ""
    echo "$ECHO_PREFIX 🛠️  接下来的步骤:"
    echo "$ECHO_PREFIX   1. 开发后端API服务 (backend/)"
    echo "$ECHO_PREFIX   2. 更新前端代码集成后端API"
    echo "$ECHO_PREFIX   3. 测试和部署完整系统"
    echo ""
    echo "$ECHO_PREFIX 📁 相关命令:"
    echo "$ECHO_PREFIX   查看日志: docker-compose logs -f"
    echo "$ECHO_PREFIX   停止服务: docker-compose down"
    echo "$ECHO_PREFIX   重启服务: docker-compose restart"
    echo "$ECHO_PREFIX   进入数据库: docker-compose exec postgres psql -U admin -d markdown_manager"
    echo ""
}

# 主流程
main() {
    echo "$ECHO_PREFIX 开始快速启动流程..."
    
    check_dependencies
    generate_env_config
    check_config
    start_services
    
    # 等待服务完全启动
    echo "$ECHO_PREFIX 等待服务完全启动..."
    sleep 5
    
    check_services
    show_usage_info
}

# 处理命令行参数
case "${1:-}" in
    "--help" | "-h")
        echo "使用方法: $0 [options]"
        echo ""
        echo "选项:"
        echo "  --help, -h    显示帮助信息"
        echo "  --check       仅检查服务状态"
        echo "  --stop        停止所有服务"
        echo "  --restart     重启所有服务"
        exit 0
        ;;
    "--check")
        check_services
        exit 0
        ;;
    "--stop")
        echo "$ECHO_PREFIX 停止所有服务..."
        docker-compose down
        echo "$ECHO_PREFIX 服务已停止"
        exit 0
        ;;
    "--restart")
        echo "$ECHO_PREFIX 重启所有服务..."
        docker-compose restart
        check_services
        exit 0
        ;;
    "")
        main
        ;;
    *)
        echo "$ECHO_PREFIX 错误: 未知参数 '$1'"
        echo "$ECHO_PREFIX 使用 '$0 --help' 查看帮助信息"
        exit 1
        ;;
esac