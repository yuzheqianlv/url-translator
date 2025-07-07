#!/bin/bash

# 健康检查脚本

PORT=${1:-3000}
HOST=${2:-localhost}

echo "🔍 检查应用健康状态..."
echo "📍 地址: http://$HOST:$PORT"

# 检查HTTP响应
response=$(curl -s -o /dev/null -w "%{http_code}" "http://$HOST:$PORT" || echo "000")

if [ "$response" = "200" ]; then
    echo "✅ 应用运行正常"
    echo "🌐 访问地址: http://$HOST:$PORT"
    exit 0
elif [ "$response" = "000" ]; then
    echo "❌ 无法连接到应用"
    echo "💡 请检查容器是否正在运行: docker-compose ps"
    exit 1
else
    echo "⚠️  应用响应异常 (HTTP $response)"
    echo "💡 请检查应用日志: docker-compose logs"
    exit 1
fi