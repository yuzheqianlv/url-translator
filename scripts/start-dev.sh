#!/bin/bash

# URL翻译工具开发环境启动脚本
# URL Translator Development Environment Startup Script

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
echo -e "${BLUE}📁 项目根目录: $PROJECT_ROOT${NC}"

# 检查是否在项目根目录
if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
    echo -e "${RED}❌ 错误: 未找到项目根目录${NC}"
    exit 1
fi

# 日志函数
log_info() {
    echo -e "${GREEN}ℹ️  $1${NC}"
}

log_warn() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# 检查必要工具
check_requirements() {
    log_info "检查开发环境依赖..."
    
    local missing_tools=()
    
    if ! command -v cargo >/dev/null 2>&1; then
        missing_tools+=("cargo (Rust)")
    fi
    
    if ! command -v trunk >/dev/null 2>&1; then
        missing_tools+=("trunk (Rust WASM工具)")
    fi
    
    if ! command -v nc >/dev/null 2>&1; then
        missing_tools+=("netcat (网络工具)")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        log_error "缺少必要工具: ${missing_tools[*]}"
        echo "请安装缺少的工具:"
        echo "  cargo: https://rustup.rs/"
        echo "  trunk: cargo install trunk"
        echo "  netcat: apt-get install netcat (Ubuntu/Debian)"
        exit 1
    fi
    
    log_info "所有必要工具已安装 ✅"
}

# 检查OrbStack服务
check_orbstack_services() {
    log_info "检查 OrbStack 服务状态..."
    
    if [[ -f "$PROJECT_ROOT/scripts/check-orbstack-services.sh" ]]; then
        if ! "$PROJECT_ROOT/scripts/check-orbstack-services.sh"; then
            log_error "OrbStack 服务检查失败"
            log_warn "请确保以下容器正在运行:"
            log_warn "  - postgres.markdown-manager.orb.local"
            log_warn "  - redis.markdown-manager.orb.local"
            log_warn "  - meilisearch.markdown-manager.orb.local"
            exit 1
        fi
    else
        log_warn "OrbStack 服务检查脚本不存在，跳过检查"
    fi
}

# 启动后端服务
start_backend() {
    log_info "启动后端服务..."
    
    cd "$PROJECT_ROOT/backend"
    
    # 检查环境配置文件
    if [[ -f ".env.local" ]]; then
        log_info "使用本地环境配置文件: .env.local"
        export $(cat .env.local | grep -v '^#' | grep -v '^$' | xargs)
    elif [[ -f ".env" ]]; then
        log_info "使用环境配置文件: .env"
        export $(cat .env | grep -v '^#' | grep -v '^$' | xargs)
    else
        log_warn "未找到环境配置文件，使用默认配置"
    fi
    
    # 检查端口是否被占用
    if nc -z localhost 3002 2>/dev/null; then
        log_warn "端口 3002 已被占用，请先关闭现有服务"
        exit 1
    fi
    
    # 启动后端服务
    log_info "在端口 3002 启动后端服务..."
    cargo run &
    BACKEND_PID=$!
    
    # 等待后端服务启动
    for i in {1..30}; do
        if nc -z localhost 3002 2>/dev/null; then
            log_info "后端服务启动成功 ✅"
            break
        fi
        sleep 1
    done
    
    if ! nc -z localhost 3002 2>/dev/null; then
        log_error "后端服务启动失败"
        kill $BACKEND_PID 2>/dev/null || true
        exit 1
    fi
}

# 启动前端服务
start_frontend() {
    log_info "启动前端服务..."
    
    cd "$PROJECT_ROOT"
    
    # 检查端口是否被占用
    if nc -z localhost 3001 2>/dev/null; then
        log_warn "端口 3001 已被占用，请先关闭现有服务"
        exit 1
    fi
    
    # 启动前端服务
    log_info "在端口 3001 启动前端服务..."
    trunk serve --port 3001 &
    FRONTEND_PID=$!
    
    # 等待前端服务启动
    for i in {1..30}; do
        if nc -z localhost 3001 2>/dev/null; then
            log_info "前端服务启动成功 ✅"
            break
        fi
        sleep 1
    done
    
    if ! nc -z localhost 3001 2>/dev/null; then
        log_error "前端服务启动失败"
        kill $FRONTEND_PID 2>/dev/null || true
        exit 1
    fi
}

# 显示服务信息
show_service_info() {
    echo
    echo -e "${GREEN}🎉 开发环境启动成功！${NC}"
    echo
    echo "服务地址:"
    echo -e "  ${BLUE}前端应用: http://localhost:3001${NC}"
    echo -e "  ${BLUE}后端API:  http://localhost:3002${NC}"
    echo -e "  ${BLUE}健康检查: http://localhost:3002/health${NC}"
    echo
    echo "外部服务:"
    echo -e "  ${BLUE}PostgreSQL: postgres.markdown-manager.orb.local:5432${NC}"
    echo -e "  ${BLUE}Redis:      redis.markdown-manager.orb.local:6379${NC}"
    echo -e "  ${BLUE}MeiliSearch: meilisearch.markdown-manager.orb.local:7700${NC}"
    echo
    echo "按 Ctrl+C 停止所有服务"
}

# 清理函数
cleanup() {
    log_info "正在停止服务..."
    
    if [[ -n "$BACKEND_PID" ]]; then
        kill $BACKEND_PID 2>/dev/null || true
        log_info "后端服务已停止"
    fi
    
    if [[ -n "$FRONTEND_PID" ]]; then
        kill $FRONTEND_PID 2>/dev/null || true
        log_info "前端服务已停止"
    fi
    
    log_info "所有服务已停止"
    exit 0
}

# 设置信号处理
trap cleanup SIGINT SIGTERM

# 显示帮助信息
show_help() {
    echo "URL翻译工具开发环境启动脚本"
    echo
    echo "用法: $0 [选项]"
    echo
    echo "选项:"
    echo "  -h, --help        显示此帮助信息"
    echo "  --skip-check      跳过 OrbStack 服务检查"
    echo "  --backend-only    仅启动后端服务"
    echo "  --frontend-only   仅启动前端服务"
    echo
    echo "此脚本将启动："
    echo "  - 后端服务 (http://localhost:3002)"
    echo "  - 前端服务 (http://localhost:3001)"
    echo "  - 并检查 OrbStack 外部服务连接"
}

# 解析命令行参数
SKIP_CHECK=false
BACKEND_ONLY=false
FRONTEND_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        --skip-check)
            SKIP_CHECK=true
            shift
            ;;
        --backend-only)
            BACKEND_ONLY=true
            shift
            ;;
        --frontend-only)
            FRONTEND_ONLY=true
            shift
            ;;
        *)
            echo "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 主流程
main() {
    echo -e "${BLUE}🚀 启动 URL翻译工具开发环境...${NC}"
    echo
    
    # 检查环境
    check_requirements
    
    # 检查OrbStack服务
    if [[ "$SKIP_CHECK" != "true" ]]; then
        check_orbstack_services
    fi
    
    # 启动服务
    if [[ "$FRONTEND_ONLY" == "true" ]]; then
        start_frontend
    elif [[ "$BACKEND_ONLY" == "true" ]]; then
        start_backend
    else
        start_backend
        start_frontend
    fi
    
    # 显示服务信息
    show_service_info
    
    # 等待用户中断
    while true; do
        sleep 1
    done
}

# 执行主流程
main