#!/bin/bash

# URL翻译工具 Docker部署脚本

echo "🚀 开始部署URL翻译工具..."

# 检查Docker是否安装
if ! command -v docker &> /dev/null; then
    echo "❌ Docker未安装，请先安装Docker"
    exit 1
fi

# 检查Docker Compose是否安装
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose未安装，请先安装Docker Compose"
    exit 1
fi

# 停止并删除现有容器
echo "🛑 停止现有容器..."
docker-compose down

# 构建并启动容器
echo "🔨 构建应用镜像..."
docker-compose build

echo "▶️  启动应用..."
docker-compose up -d

# 检查容器状态
echo "✅ 检查容器状态..."
docker-compose ps

echo ""
echo "🎉 部署完成！"
echo "📍 访问地址: http://localhost:3000"
echo ""
echo "管理命令:"
echo "  停止应用: docker-compose down"
echo "  查看日志: docker-compose logs -f"
echo "  重启应用: docker-compose restart"