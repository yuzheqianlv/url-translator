#!/bin/bash

# URL翻译工具 Docker部署脚本
# 增强版：支持多种部署选项和环境配置

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# 项目配置
PROJECT_NAME="url-translator"
DEFAULT_PORT="3000"
HEALTH_CHECK_TIMEOUT=60

# 显示帮助信息
show_help() {
    echo -e "${BLUE}URL翻译工具 Docker部署脚本${NC}"
    echo ""
    echo "用法: $0 [选项] [命令]"
    echo ""
    echo "命令:"
    echo "  deploy    完整部署 (默认)"
    echo "  build     仅构建镜像"
    echo "  start     启动容器"
    echo "  stop      停止容器"
    echo "  restart   重启容器"
    echo "  status    查看状态"
    echo "  logs      查看日志"
    echo "  clean     清理资源"
    echo ""
    echo "选项:"
    echo "  -p, --port PORT     指定端口 (默认: 3000)"
    echo "  -d, --dev           开发模式部署"
    echo "  -f, --force         强制重新构建"
    echo "  -h, --help          显示帮助"
    echo ""
    echo "示例:"
    echo "  $0                  # 标准部署"
    echo "  $0 -p 8080 deploy   # 指定端口部署"
    echo "  $0 -f build         # 强制重新构建"
    echo "  $0 logs             # 查看日志"
}

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# 检查依赖
check_dependencies() {
    log_step "检查依赖环境..."
    
    if ! command -v docker &> /dev/null; then
        log_error "Docker未安装，请先安装Docker"
        echo "安装指南: https://docs.docker.com/get-docker/"
        exit 1
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_error "Docker Compose未安装，请先安装Docker Compose"
        echo "安装指南: https://docs.docker.com/compose/install/"
        exit 1
    fi
    
    # 检查Docker服务是否运行
    if ! docker info &> /dev/null; then
        log_error "Docker服务未运行，请启动Docker服务"
        exit 1
    fi
    
    log_success "依赖检查通过"
}

# 设置端口
setup_port() {
    if [ -n "$CUSTOM_PORT" ]; then
        log_info "使用自定义端口: $CUSTOM_PORT"
        export PORT=$CUSTOM_PORT
        # 更新docker-compose.yml中的端口映射
        sed -i.bak "s/\"[0-9]*:80\"/\"$CUSTOM_PORT:80\"/" docker-compose.yml
    else
        export PORT=$DEFAULT_PORT
        log_info "使用默认端口: $DEFAULT_PORT"
    fi
}

# 构建镜像
build_image() {
    log_step "构建Docker镜像..."
    
    if [ "$FORCE_BUILD" = "true" ]; then
        log_info "强制重新构建 (--no-cache)"
        docker-compose build --no-cache
    else
        docker-compose build
    fi
    
    log_success "镜像构建完成"
}

# 启动容器
start_containers() {
    log_step "启动容器..."
    docker-compose up -d
    log_success "容器启动完成"
}

# 等待服务就绪
wait_for_service() {
    log_step "等待服务启动..."
    
    local timeout=$HEALTH_CHECK_TIMEOUT
    local count=0
    
    while [ $count -lt $timeout ]; do
        if curl -f -s "http://localhost:$PORT" > /dev/null 2>&1; then
            log_success "服务已就绪"
            return 0
        fi
        
        echo -n "."
        sleep 1
        ((count++))
    done
    
    echo ""
    log_warning "服务启动超时，请检查日志"
    return 1
}

# 停止容器
stop_containers() {
    log_step "停止容器..."
    docker-compose down
    log_success "容器已停止"
}

# 显示状态
show_status() {
    log_step "应用状态:"
    docker-compose ps
    
    echo ""
    log_step "资源使用:"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}" $(docker-compose ps -q) 2>/dev/null || echo "无运行容器"
}

# 查看日志
show_logs() {
    log_step "查看应用日志..."
    docker-compose logs -f --tail=100
}

# 清理资源
cleanup() {
    log_step "清理Docker资源..."
    
    # 停止并删除容器
    docker-compose down --volumes
    
    # 删除项目镜像
    if docker images | grep -q "$PROJECT_NAME"; then
        docker rmi $(docker images "*$PROJECT_NAME*" -q) 2>/dev/null || true
    fi
    
    # 清理未使用的资源
    docker system prune -f
    
    log_success "清理完成"
}

# 完整部署
deploy() {
    echo -e "${BLUE}"
    echo "🚀 开始部署URL翻译工具..."
    echo "=================================="
    echo -e "${NC}"
    
    check_dependencies
    setup_port
    
    # 停止现有容器
    if docker-compose ps | grep -q "Up"; then
        log_step "停止现有容器..."
        docker-compose down
    fi
    
    build_image
    start_containers
    
    # 健康检查
    if wait_for_service; then
        echo ""
        echo -e "${GREEN}🎉 部署完成！${NC}"
        echo -e "${BLUE}📍 访问地址: http://localhost:$PORT${NC}"
        echo ""
        echo "管理命令:"
        echo "  查看状态: $0 status"
        echo "  查看日志: $0 logs"
        echo "  停止应用: $0 stop"
        echo "  重启应用: $0 restart"
        echo ""
        
        # 运行健康检查
        if [ -f "./health-check.sh" ]; then
            log_info "运行健康检查..."
            ./health-check.sh $PORT
        fi
    else
        log_error "部署可能存在问题，请检查日志"
        show_logs
    fi
}

# 解析命令行参数
FORCE_BUILD=false
DEV_MODE=false
CUSTOM_PORT=""
COMMAND="deploy"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -p|--port)
            CUSTOM_PORT="$2"
            shift 2
            ;;
        -f|--force)
            FORCE_BUILD=true
            shift
            ;;
        -d|--dev)
            DEV_MODE=true
            shift
            ;;
        deploy|build|start|stop|restart|status|logs|clean)
            COMMAND="$1"
            shift
            ;;
        *)
            log_error "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 执行命令
case $COMMAND in
    deploy)
        deploy
        ;;
    build)
        check_dependencies
        setup_port
        build_image
        ;;
    start)
        check_dependencies
        setup_port
        start_containers
        wait_for_service
        ;;
    stop)
        stop_containers
        ;;
    restart)
        check_dependencies
        setup_port
        stop_containers
        start_containers
        wait_for_service
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    clean)
        cleanup
        ;;
    *)
        log_error "未知命令: $COMMAND"
        show_help
        exit 1
        ;;
esac