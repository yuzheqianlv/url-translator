# URL翻译工具快速入门指南

## 🚀 快速开始

这个指南将帮助您快速搭建和运行 URL 翻译工具的完整系统。

## 📦 系统架构

```
┌──────────────────────┐
│     前端 (Leptos WASM)    │
│   http://localhost:3001   │
├──────────────────────┤
│     后端API (Rust)       │
│   http://localhost:3002   │
├──────────────────────┤
│   数据存储和搜索服务    │
│  PostgreSQL  Redis  Meili │
│   :5432      :6379   :7700 │
└──────────────────────┘
```

## 🔧 环境要求

### 基本要求
- **Docker**: 20.10+
- **Docker Compose**: 2.0+
- **系统**: Linux/macOS/Windows (WSL2)

### 可选要求 (用于开发)
- **Rust**: 1.70+
- **Node.js**: 18+ (用于前端开发)
- **PostgreSQL Client**: 用于数据库管理

## 📝 安装步骤

### 方式1: 一键安装 (推荐)

```bash
# 1. 克隆项目
git clone <repository-url>
cd url-translator

# 2. 运行快速启动脚本
./scripts/quick-start.sh
```

这个脚本将自动：
- 检查系统依赖
- 生成安全的环境配置
- 启动所有数据库服务
- 检查服务状态

### 方式2: 手动安装

```bash
# 1. 克隆项目
git clone <repository-url>
cd url-translator

# 2. 生成环境配置
./scripts/generate-env.sh

# 3. 编辑配置文件
nano .env  # 更新 DEEPLX_API_URL 等配置

# 4. 启动服务
docker-compose up -d
```

## ⚙️ 配置说明

### 必须配置

在 `.env` 文件中更新以下配置：

```bash
# DeepLX API 地址 (必须修改)
DEEPLX_API_URL=https://your-deeplx-api.com/translate

# 其他API配置
JINA_API_URL=https://r.jina.ai
```

### 可选配置

```bash
# 性能调优
MAX_REQUESTS_PER_SECOND=10
MAX_TEXT_LENGTH=5000
MAX_PARAGRAPHS_PER_REQUEST=10

# 语言设置
DEFAULT_SOURCE_LANG=auto
DEFAULT_TARGET_LANG=zh

# 端口配置
APP_PORT=3001
```

## 📨 服务管理

### 基本命令

```bash
# 启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down

# 重启服务
docker-compose restart
```

### 使用快速启动脚本

```bash
# 检查服务状态
./scripts/quick-start.sh --check

# 停止所有服务
./scripts/quick-start.sh --stop

# 重启所有服务
./scripts/quick-start.sh --restart
```

## 🔗 服务访问

### Web 界面
- **主应用**: http://localhost:3001
- **后端API**: http://localhost:3002 (开发中)

### 数据库服务
- **PostgreSQL**: localhost:5432
  - 数据库: `markdown_manager`
  - 用户: `admin`
  - 密码: 查看 `.env` 文件
  
- **Redis**: localhost:6379
  - 密码: 查看 `.env` 文件
  
- **MeiliSearch**: http://localhost:7700
  - API Key: 查看 `.env` 文件

### 数据库连接

```bash
# 连接 PostgreSQL
docker-compose exec postgres psql -U admin -d markdown_manager

# 连接 Redis
docker-compose exec redis redis-cli

# 测试 MeiliSearch
curl http://localhost:7700/health
```

## 🛠️ 开发指南

### 前端开发

```bash
# 安装 Rust 工具链
rustup target add wasm32-unknown-unknown
cargo install trunk

# 启动开发服务器
trunk serve --open
```

### 后端开发

后端 API 服务尚在开发中，请参考 [backend-architecture.md](./backend-architecture.md) 查看详细设计。

### 数据库管理

```bash
# 查看数据库表
docker-compose exec postgres psql -U admin -d markdown_manager -c "\dt"

# 查看用户数据
docker-compose exec postgres psql -U admin -d markdown_manager -c "SELECT * FROM users;"

# 备份数据库
docker-compose exec postgres pg_dump -U admin markdown_manager > backup.sql
```

## 🔍 故障排除

### 常见问题

#### 1. 端口冲突
```bash
# 检查端口占用
lsof -i :3001
lsof -i :5432
lsof -i :6379
lsof -i :7700

# 修改 .env 文件中的端口配置
```

#### 2. Docker 权限问题
```bash
# 添加用户到 docker 组
sudo usermod -aG docker $USER
# 重新登录生效
```

#### 3. 数据库连接失败
```bash
# 检查容器状态
docker-compose ps

# 查看数据库日志
docker-compose logs postgres

# 重启数据库
docker-compose restart postgres
```

#### 4. 存储空间不足
```bash
# 清理 Docker 缓存
docker system prune -f

# 清理旧镜像
docker image prune -a
```

### 日志查看

```bash
# 查看所有服务日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f postgres
docker-compose logs -f redis
docker-compose logs -f meilisearch
```

### 性能监控

```bash
# 查看资源使用
docker stats

# 查看容器信息
docker-compose exec postgres top
docker-compose exec redis redis-cli info memory
```

## 🔒 安全注意事项

### 生产部署建议

1. **修改默认密码**
   - 重新生成所有数据库密码
   - 使用强密码策略

2. **网络安全**
   - 使用 HTTPS/TLS 加密
   - 配置防火墙规则
   - 限制数据库访问

3. **数据备份**
   - 定期备份数据库
   - 测试数据恢复流程

4. **监控和日志**
   - 设置监控告警
   - 集中化日志管理

### 数据隐私

- 所有用户数据都存储在本地
- 不会向第三方发送敏感信息
- API 密钥仅用于翻译服务

## 📚 参考文档

- [backend-architecture.md](./backend-architecture.md) - 后端架构设计
- [troubleshooting.md](./troubleshooting.md) - 故障排除指南
- [jina-service.md](./jina-service.md) - Jina AI 服务说明
- [deeplx-service.md](./deeplx-service.md) - DeepLX 服务说明
- [README.md](../README.md) - 项目概览

## 🤝 贡献指南

1. Fork 项目
2. 创建特性分支
3. 提交修改
4. 发起 Pull Request

## 📞 支持

如果遇到问题，请：
1. 查看 [troubleshooting.md](./troubleshooting.md)
2. 搜索现有 Issues
3. 创建新的 Issue 并提供详细信息

---

**祝您使用愉快！** 🎉