#!/bin/bash

# URL翻译工具健康检查脚本
# 增强版：支持更全面的健康状态检查

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 默认配置
DEFAULT_PORT=3000
DEFAULT_HOST="localhost"
TIMEOUT=30
RETRY_COUNT=3
CHECK_INTERVAL=2

# 显示帮助信息
show_help() {
    echo -e "${BLUE}URL翻译工具健康检查脚本${NC}"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  -p, --port PORT     指定端口 (默认: 3000)"
    echo "  -h, --host HOST     指定主机 (默认: localhost)"
    echo "  -t, --timeout SEC   超时时间 (默认: 30秒)"
    echo "  -r, --retry COUNT   重试次数 (默认: 3次)"
    echo "  -v, --verbose       详细输出"
    echo "  --help              显示帮助"
    echo ""
    echo "示例:"
    echo "  $0                  # 基本健康检查"
    echo "  $0 -p 8080          # 检查8080端口"
    echo "  $0 -v               # 详细输出"
    echo "  $0 -t 60 -r 5       # 60秒超时，重试5次"
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

log_verbose() {
    if [ "$VERBOSE" = "true" ]; then
        echo -e "${NC}[DEBUG] $1"
    fi
}

# 检查端口是否开放
check_port() {
    log_verbose "检查端口 $HOST:$PORT 是否开放..."
    
    if command -v nc &> /dev/null; then
        if nc -z "$HOST" "$PORT" &> /dev/null; then
            log_verbose "端口 $PORT 已开放"
            return 0
        else
            log_verbose "端口 $PORT 未开放"
            return 1
        fi
    else
        log_verbose "netcat未安装，跳过端口检查"
        return 0
    fi
}

# 检查HTTP响应
check_http_response() {
    local url="http://$HOST:$PORT"
    log_verbose "检查HTTP响应: $url"
    
    local response
    response=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 10 --max-time "$TIMEOUT" "$url" 2>/dev/null || echo "000")
    
    log_verbose "HTTP响应码: $response"
    echo "$response"
}

# 检查响应时间
check_response_time() {
    local url="http://$HOST:$PORT"
    log_verbose "检查响应时间..."
    
    local time
    time=$(curl -s -o /dev/null -w "%{time_total}" --connect-timeout 10 --max-time "$TIMEOUT" "$url" 2>/dev/null || echo "999")
    
    log_verbose "响应时间: ${time}秒"
    echo "$time"
}

# 检查Docker容器状态
check_docker_status() {
    log_verbose "检查Docker容器状态..."
    
    if ! command -v docker &> /dev/null; then
        log_verbose "Docker未安装，跳过容器检查"
        return 0
    fi
    
    if ! command -v docker-compose &> /dev/null; then
        log_verbose "Docker Compose未安装，跳过容器检查"
        return 0
    fi
    
    local container_status
    container_status=$(docker-compose ps --filter "status=running" --quiet 2>/dev/null | wc -l)
    
    log_verbose "运行中的容器数量: $container_status"
    
    if [ "$container_status" -gt 0 ]; then
        return 0
    else
        return 1
    fi
}

# 检查应用特定功能
check_app_features() {
    local url="http://$HOST:$PORT"
    log_verbose "检查应用特定功能..."
    
    # 检查是否能获取页面内容
    local content
    content=$(curl -s --connect-timeout 10 --max-time "$TIMEOUT" "$url" 2>/dev/null || echo "")
    
    if echo "$content" | grep -q "URL翻译工具" &> /dev/null; then
        log_verbose "应用页面内容正常"
        return 0
    else
        log_verbose "应用页面内容异常"
        return 1
    fi
}

# 主健康检查函数
health_check() {
    local url="http://$HOST:$PORT"
    local attempt=1
    local success=false
    
    echo -e "${BLUE}"
    echo "🔍 开始健康检查..."
    echo "=================================="
    echo -e "${NC}"
    
    log_info "目标地址: $url"
    log_info "超时时间: ${TIMEOUT}秒"
    log_info "重试次数: $RETRY_COUNT"
    
    while [ $attempt -le $RETRY_COUNT ] && [ "$success" = "false" ]; do
        echo ""
        log_info "第 $attempt 次检查..."
        
        # 1. 检查端口
        if check_port; then
            log_success "端口连通性检查通过"
        else
            log_warning "端口连通性检查失败"
        fi
        
        # 2. 检查HTTP响应
        local http_code
        http_code=$(check_http_response)
        
        case "$http_code" in
            "200")
                log_success "HTTP响应检查通过 (200 OK)"
                
                # 3. 检查响应时间
                local response_time
                response_time=$(check_response_time)
                
                if (( $(echo "$response_time < 5.0" | bc -l) )); then
                    log_success "响应时间检查通过 (${response_time}秒)"
                else
                    log_warning "响应时间较慢 (${response_time}秒)"
                fi
                
                # 4. 检查应用功能
                if check_app_features; then
                    log_success "应用功能检查通过"
                else
                    log_warning "应用功能检查失败"
                fi
                
                success=true
                ;;
            "000")
                log_error "无法连接到应用"
                ;;
            *)
                log_error "HTTP响应异常 (状态码: $http_code)"
                ;;
        esac
        
        if [ "$success" = "false" ] && [ $attempt -lt $RETRY_COUNT ]; then
            log_info "等待 ${CHECK_INTERVAL} 秒后重试..."
            sleep $CHECK_INTERVAL
        fi
        
        ((attempt++))
    done
    
    echo ""
    echo "=================================="
    
    if [ "$success" = "true" ]; then
        log_success "✅ 应用健康状态良好"
        echo -e "${GREEN}🌐 访问地址: $url${NC}"
        
        # 显示Docker容器状态
        if check_docker_status; then
            log_success "Docker容器运行正常"
        else
            log_warning "Docker容器状态异常"
        fi
        
        return 0
    else
        log_error "❌ 应用健康检查失败"
        echo ""
        echo "故障排除建议:"
        echo "1. 检查容器状态: docker-compose ps"
        echo "2. 查看应用日志: docker-compose logs"
        echo "3. 检查端口占用: netstat -tlnp | grep :$PORT"
        echo "4. 重启应用: ./deploy.sh restart"
        
        return 1
    fi
}

# 解析命令行参数
PORT=$DEFAULT_PORT
HOST=$DEFAULT_HOST
VERBOSE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -p|--port)
            PORT="$2"
            shift 2
            ;;
        -h|--host)
            HOST="$2"
            shift 2
            ;;
        -t|--timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -r|--retry)
            RETRY_COUNT="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            # 兼容旧版本调用方式
            if [[ "$1" =~ ^[0-9]+$ ]]; then
                PORT="$1"
                shift
            elif [ -z "$2" ]; then
                HOST="$1"
                shift
            else
                log_error "未知参数: $1"
                show_help
                exit 1
            fi
            ;;
    esac
done

# 验证参数
if ! [[ "$PORT" =~ ^[0-9]+$ ]] || [ "$PORT" -lt 1 ] || [ "$PORT" -gt 65535 ]; then
    log_error "无效的端口号: $PORT"
    exit 1
fi

if ! [[ "$TIMEOUT" =~ ^[0-9]+$ ]] || [ "$TIMEOUT" -lt 1 ]; then
    log_error "无效的超时时间: $TIMEOUT"
    exit 1
fi

# 运行健康检查
health_check