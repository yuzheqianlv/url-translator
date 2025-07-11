#!/bin/bash

# OrbStack 服务连接验证脚本
# OrbStack Services Connection Verification Script

set -e

echo "🔍 正在检查 OrbStack 服务连接状态..."
echo "🔍 Checking OrbStack services connection status..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 服务配置
POSTGRES_HOST="postgres.markdown-manager.orb.local"
POSTGRES_PORT="5432"
REDIS_HOST="redis.markdown-manager.orb.local"
REDIS_PORT="6379"
MEILISEARCH_HOST="meilisearch.markdown-manager.orb.local"
MEILISEARCH_PORT="7700"

# 检查服务连接性
check_service() {
    local service_name=$1
    local host=$2
    local port=$3
    
    echo -n "  检查 $service_name ($host:$port)... "
    
    if nc -z "$host" "$port" 2>/dev/null; then
        echo -e "${GREEN}✅ 连接成功${NC}"
        return 0
    else
        echo -e "${RED}❌ 连接失败${NC}"
        return 1
    fi
}

# 检查DNS解析
check_dns() {
    local host=$1
    echo -n "  检查 DNS 解析 ($host)... "
    
    if nslookup "$host" >/dev/null 2>&1; then
        echo -e "${GREEN}✅ 解析成功${NC}"
        return 0
    else
        echo -e "${RED}❌ 解析失败${NC}"
        return 1
    fi
}

# 检查PostgreSQL认证
check_postgres_auth() {
    echo -n "  检查 PostgreSQL 认证... "
    
    # 使用URL编码的密码
    local encoded_password="4%29LzTzN%29mT4Zn25y%5Ep43k.q%22%2C%3BN%7D%214W%3Fdm5206"
    local connection_string="postgres://admin:${encoded_password}@${POSTGRES_HOST}:${POSTGRES_PORT}/markdown_manager"
    
    if command -v psql >/dev/null 2>&1; then
        if psql "$connection_string" -c "SELECT 1;" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ 认证成功${NC}"
            return 0
        else
            echo -e "${RED}❌ 认证失败${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  psql 未安装，跳过认证测试${NC}"
        return 0
    fi
}

# 检查Redis认证
check_redis_auth() {
    echo -n "  检查 Redis 认证... "
    
    local password="4)LzTzN)mT4Zn25y^p43k.q\",;N}!4W?dm5206"
    
    if command -v redis-cli >/dev/null 2>&1; then
        if redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" -a "$password" ping >/dev/null 2>&1; then
            echo -e "${GREEN}✅ 认证成功${NC}"
            return 0
        else
            echo -e "${RED}❌ 认证失败${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  redis-cli 未安装，跳过认证测试${NC}"
        return 0
    fi
}

# 检查MeiliSearch API
check_meilisearch_api() {
    echo -n "  检查 MeiliSearch API... "
    
    local api_key="4)LzTzN)mT4Zn25y^p43k.q\",;N}!4W?dm5206"
    
    if command -v curl >/dev/null 2>&1; then
        if curl -s -H "Authorization: Bearer $api_key" "http://${MEILISEARCH_HOST}:${MEILISEARCH_PORT}/health" >/dev/null 2>&1; then
            echo -e "${GREEN}✅ API 响应正常${NC}"
            return 0
        else
            echo -e "${RED}❌ API 响应失败${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  curl 未安装，跳过API测试${NC}"
        return 0
    fi
}

# 主检查流程
main() {
    echo "📋 开始 OrbStack 服务检查..."
    echo
    
    local failed_services=0
    
    # 检查DNS解析
    echo "1. DNS 解析检查："
    check_dns "$POSTGRES_HOST" || ((failed_services++))
    check_dns "$REDIS_HOST" || ((failed_services++))
    check_dns "$MEILISEARCH_HOST" || ((failed_services++))
    echo
    
    # 检查服务连接
    echo "2. 服务连接检查："
    check_service "PostgreSQL" "$POSTGRES_HOST" "$POSTGRES_PORT" || ((failed_services++))
    check_service "Redis" "$REDIS_HOST" "$REDIS_PORT" || ((failed_services++))
    check_service "MeiliSearch" "$MEILISEARCH_HOST" "$MEILISEARCH_PORT" || ((failed_services++))
    echo
    
    # 检查服务认证
    echo "3. 服务认证检查："
    check_postgres_auth || ((failed_services++))
    check_redis_auth || ((failed_services++))
    check_meilisearch_api || ((failed_services++))
    echo
    
    # 总结
    if [ $failed_services -eq 0 ]; then
        echo -e "${GREEN}🎉 所有服务检查通过！${NC}"
        echo -e "${GREEN}🚀 可以启动后端服务${NC}"
        exit 0
    else
        echo -e "${RED}❌ 有 $failed_services 个服务检查失败${NC}"
        echo -e "${RED}🔧 请检查 OrbStack 容器状态和网络配置${NC}"
        exit 1
    fi
}

# 检查必要的系统工具
check_requirements() {
    if ! command -v nc >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  netcat (nc) 未安装，某些连接测试可能不可用${NC}"
    fi
    
    if ! command -v nslookup >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  nslookup 未安装，DNS 解析测试可能不可用${NC}"
    fi
}

# 显示帮助信息
show_help() {
    echo "OrbStack 服务连接验证脚本"
    echo
    echo "用法: $0 [选项]"
    echo
    echo "选项:"
    echo "  -h, --help     显示此帮助信息"
    echo "  -v, --verbose  显示详细输出"
    echo
    echo "此脚本检查以下服务的连接状态："
    echo "  - PostgreSQL (postgres.markdown-manager.orb.local:5432)"
    echo "  - Redis (redis.markdown-manager.orb.local:6379)"
    echo "  - MeiliSearch (meilisearch.markdown-manager.orb.local:7700)"
}

# 处理命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            set -x
            shift
            ;;
        *)
            echo "未知参数: $1"
            show_help
            exit 1
            ;;
    esac
done

# 执行主流程
check_requirements
main