# URL翻译工具

🌐 基于Rust的现代化全栈Web应用，提供智能网页内容翻译服务，支持用户管理、项目协作和云端数据同步。

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/Leptos-0.6-blue.svg)](https://leptos.dev/)
[![Docker](https://img.shields.io/badge/Docker-Ready-blue.svg)](https://www.docker.com/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## ✨ 核心特性

### 🎯 智能翻译
- 📄 **智能内容提取**: 基于Jina AI Reader的网页内容智能识别和提取
- 🌍 **高质量翻译**: 集成DeepLX API，支持多语言高质量翻译
- 📝 **格式保持**: 完美保持Markdown格式，支持代码块保护
- 🔄 **批量处理**: 支持多URL批量翻译，智能队列管理

### 👥 用户系统
- 🔐 **安全认证**: JWT基础的用户认证系统，支持注册/登录
- 📊 **项目管理**: 创建和管理翻译项目，团队协作支持
- 📚 **历史记录**: 云端翻译历史同步，支持搜索和分类
- ⚙️ **个性化配置**: 用户级别的API配置和偏好设置

### 🏗️ 技术架构
- 🚀 **全栈Rust**: 前后端统一技术栈，类型安全保证
- 💾 **数据持久化**: PostgreSQL + Redis缓存 + MeiliSearch搜索
- 🎨 **现代UI**: Leptos响应式组件 + Catppuccin主题系统
- 🐳 **容器化部署**: Docker + 自动化部署脚本

## 🛠️ 技术栈

### 前端技术
- **框架**: Leptos 0.6 (Rust WASM全栈框架)
- **构建**: Trunk + WebAssembly
- **样式**: Tailwind CSS + Catppuccin主题
- **状态管理**: Leptos Signals + 响应式编程

### 后端技术
- **Web框架**: Axum (高性能异步Web框架)
- **数据库**: PostgreSQL (主数据库) + SQLx (类型安全ORM)
- **缓存**: Redis (性能缓存)
- **搜索**: MeiliSearch (全文搜索引擎)
- **认证**: JWT + Argon2 (安全认证和密码哈希)

### 基础设施
- **容器化**: Docker + Docker Compose
- **反向代理**: Nginx
- **部署**: 自动化脚本 + 健康检查
- **监控**: 集成日志和性能监控

## 🚀 快速开始

### 方式一：Docker部署（推荐）

最简单的方式，适合生产环境和快速体验：

```bash
# 1. 克隆项目
git clone <repository-url>
cd url-translator

# 2. 一键部署
./deploy.sh

# 3. 访问应用
# 前端: http://localhost:3001
# 后端API: http://localhost:3002
```

### 方式二：开发环境搭建

适合开发者和需要自定义配置的用户：

#### 环境要求
- Rust 1.70+
- Docker & Docker Compose
- just (任务运行器，可选)

#### 1. 初始化项目

```bash
# 克隆项目
git clone <repository-url>
cd url-translator

# 使用just初始化（推荐）
just init

# 或手动初始化
cp .env.example .env
rustup target add wasm32-unknown-unknown
cargo install trunk
```

#### 2. 配置环境

编辑 `.env` 文件，配置必要的参数：

```bash
# 后端API配置
FRONTEND_API_BASE_URL=http://localhost:3002/api/v1

# 数据库配置
DATABASE_URL=postgres://admin:your_password@localhost:5432/markdown_manager

# API密钥（生产环境请使用安全的密钥）
JWT_SECRET=your_jwt_secret_key_here
```

#### 3. 启动服务

```bash
# 启动后端服务（数据库、Redis、后端API）
cd backend && docker-compose up -d

# 启动前端开发服务器
just dev
# 或使用: ./scripts/dev.sh
```

#### 4. 验证部署

```bash
# 验证配置
just validate

# 检查服务状态
./health-check.sh
```

### 访问地址

- 🌐 **前端应用**: http://localhost:3001
- 🔧 **后端API**: http://localhost:3002
- 📊 **API文档**: http://localhost:3002/docs
- 💊 **健康检查**: http://localhost:3002/health

## 🐳 Docker 部署

### 架构概览

本项目采用微服务架构，使用Docker Compose编排所有服务：

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Nginx         │    │   Frontend      │    │   Backend       │
│   (反向代理)     │───▶│   (Leptos WASM) │───▶│   (Axum API)    │
│   Port: 80      │    │   Port: 3001    │    │   Port: 3002    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                                              │
         ▼                                              ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   PostgreSQL    │    │     Redis       │    │  MeiliSearch    │
│   (主数据库)     │    │    (缓存)       │    │   (搜索引擎)     │
│   Port: 5432    │    │   Port: 6379    │    │   Port: 7700    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 快速部署

#### 一键部署（生产环境）

```bash
# 完整部署（包含所有服务）
./deploy.sh

# 指定端口部署
./deploy.sh -p 8080

# 强制重新构建
./deploy.sh -f

# 开发模式部署
./deploy.sh -d
```

#### 分步部署

```bash
# 1. 启动基础服务（数据库、缓存、搜索）
docker-compose up -d postgres redis meilisearch

# 2. 启动后端API
docker-compose up -d backend

# 3. 启动前端和代理
docker-compose up -d frontend nginx
```

### 部署脚本命令

| 命令 | 描述 | 示例 |
|------|------|------|
| `deploy` | 完整部署流程 | `./deploy.sh deploy` |
| `build` | 仅构建镜像 | `./deploy.sh build` |
| `start` | 启动所有服务 | `./deploy.sh start` |
| `stop` | 停止所有服务 | `./deploy.sh stop` |
| `restart` | 重启服务 | `./deploy.sh restart` |
| `status` | 查看服务状态 | `./deploy.sh status` |
| `logs` | 查看服务日志 | `./deploy.sh logs backend` |
| `clean` | 清理资源 | `./deploy.sh clean` |
| `backup` | 备份数据 | `./deploy.sh backup` |
| `restore` | 恢复数据 | `./deploy.sh restore backup.sql` |

### 健康检查系统

#### 自动健康检查

```bash
# 全面健康检查
./health-check.sh

# 检查特定服务
./health-check.sh -s backend

# 详细输出模式
./health-check.sh -v

# 连续监控模式
./health-check.sh -w
```

#### 健康检查端点

| 服务 | 健康检查URL | 描述 |
|------|-------------|------|
| Frontend | `http://localhost:3001/` | 前端应用状态 |
| Backend | `http://localhost:3002/health` | 后端API健康状态 |
| Database | `http://localhost:3002/health/db` | 数据库连接状态 |
| Redis | `http://localhost:3002/health/cache` | 缓存服务状态 |
| Search | `http://localhost:3002/health/search` | 搜索引擎状态 |

### 环境配置

#### 生产环境变量

```bash
# 复制环境变量模板
cp .env.example .env

# 编辑关键配置
vim .env
```

#### 必要配置项

```bash
# API配置
FRONTEND_API_BASE_URL=http://localhost:3002/api/v1
BACKEND_API_URL=http://localhost:3002

# 数据库配置
POSTGRES_PASSWORD=your_secure_password_here
DATABASE_URL=postgres://admin:${POSTGRES_PASSWORD}@postgres:5432/markdown_manager

# 认证配置
JWT_SECRET=your_secure_jwt_secret_here

# 搜索引擎配置
MEILI_MASTER_KEY=your_secure_meilisearch_key_here

# Redis配置
REDIS_PASSWORD=your_secure_redis_password_here
```

### 数据管理

#### 数据持久化

```bash
# 数据卷挂载点
/var/lib/postgresql/data   # PostgreSQL数据
/var/lib/redis             # Redis数据
/var/lib/meilisearch       # MeiliSearch数据
```

#### 数据备份

```bash
# 自动备份
./deploy.sh backup

# 手动备份
docker-compose exec postgres pg_dump -U admin markdown_manager > backup.sql

# 备份到远程
./deploy.sh backup --remote s3://your-bucket/backups/
```

#### 数据恢复

```bash
# 从备份恢复
./deploy.sh restore backup.sql

# 从远程恢复
./deploy.sh restore --remote s3://your-bucket/backups/latest.sql
```

## 📖 使用指南

### 用户注册和登录

#### 1. 新用户注册
1. 访问应用首页，点击"登录"按钮
2. 在登录模态框中点击"没有账号？点击注册"
3. 填写用户名、邮箱和密码
4. 点击"注册"完成账户创建

#### 2. 用户登录
1. 点击页面右上角的"登录"按钮
2. 输入邮箱和密码
3. 点击"登录"进入应用

### 项目管理

#### 创建翻译项目
1. 登录后点击导航栏中的"项目管理"
2. 点击"新建项目"按钮
3. 填写项目信息：
   - 项目名称（必填）
   - 项目描述（可选）
   - 源语言和目标语言
4. 点击"创建项目"完成创建

#### 管理现有项目
- **编辑项目**: 点击项目卡片右上角的编辑图标
- **删除项目**: 点击删除图标，确认后删除
- **打开项目**: 点击"打开项目"按钮进入项目详情

### 翻译功能

#### 单页翻译
1. 在首页URL输入框中输入要翻译的网页地址
2. 选择源语言和目标语言（可选，支持自动检测）
3. 点击"开始翻译"按钮
4. 等待翻译完成，查看结果
5. 点击"下载Markdown"保存翻译结果

#### 批量翻译
1. 点击导航栏中的"批量翻译"
2. 通过以下方式添加URL：
   - 手动输入多个URL（每行一个）
   - 上传包含URL的文本文件
3. 配置翻译设置
4. 点击"开始批量翻译"
5. 监控翻译进度，下载完成的结果

### 历史记录

#### 查看翻译历史
1. 登录后点击"历史记录"
2. 浏览所有历史翻译记录
3. 支持按日期、项目、语言筛选

#### 搜索历史记录
1. 在历史记录页面使用搜索框
2. 输入关键词搜索翻译内容
3. 使用高级筛选功能精确查找

### 个性化设置

#### API配置
1. 点击"设置"页面
2. 在"API配置"部分设置：
   - DeepLX API端点
   - Jina AI Reader端点
   - API请求超时时间

#### 偏好设置
- **默认语言**: 设置常用的源语言和目标语言
- **主题选择**: 切换应用外观主题
- **通知设置**: 配置翻译完成通知

### 支持的语言

| 语言 | 代码 | 支持状态 |
|------|------|----------|
| 🇨🇳 中文（简体） | zh | ✅ 完全支持 |
| 🇭🇰 中文（繁体） | zh-TW | ✅ 完全支持 |
| 🇺🇸 英语 | en | ✅ 完全支持 |
| 🇯🇵 日语 | ja | ✅ 完全支持 |
| 🇰🇷 韩语 | ko | ✅ 完全支持 |
| 🇫🇷 法语 | fr | ✅ 完全支持 |
| 🇩🇪 德语 | de | ✅ 完全支持 |
| 🇪🇸 西班牙语 | es | ✅ 完全支持 |
| 🇮🇹 意大利语 | it | ✅ 完全支持 |
| 🇷🇺 俄语 | ru | ✅ 完全支持 |
| 🇵🇹 葡萄牙语 | pt | ✅ 完全支持 |
| 🇳🇱 荷兰语 | nl | ✅ 完全支持 |

### 高级功能

#### 代码块保护
- 自动识别和保护代码块内容
- 支持多种编程语言语法高亮
- 保持代码格式和缩进

#### 智能分块翻译
- 自动将长文本分块处理
- 保持上下文连贯性
- 优化翻译质量和速度

#### 翻译质量优化
- 智能术语识别和保护
- 上下文感知翻译
- 格式和样式保持

## 🏗️ 项目结构

### 整体架构

```
url-translator/
├── 📁 frontend/            # 前端应用（当前目录）
├── 📁 backend/             # 后端API服务
├── 📁 docs/               # 项目文档
├── 📁 scripts/            # 自动化脚本
├── 🐳 docker-compose.yml  # 服务编排
├── 🔧 .env.example        # 环境变量模板
└── 📋 justfile            # 任务运行器配置
```

### 前端结构（Leptos WASM）

```
src/
├── 🚀 main.rs                    # 应用入口点
├── 📱 app.rs                     # 主应用组件
├── ⚙️ lib.rs                     # 库导出
├── 🧩 components/                # UI组件库
│   ├── mod.rs                   
│   ├── auth_modal.rs            # 认证模态框
│   ├── header.rs                # 导航头部
│   ├── url_input.rs             # URL输入组件
│   ├── translation_result.rs    # 翻译结果组件
│   ├── batch_translation.rs     # 批量翻译组件
│   ├── progress_indicator.rs    # 进度指示器
│   └── theme_selector.rs        # 主题选择器
├── 📄 pages/                     # 页面组件
│   ├── mod.rs
│   ├── home.rs                  # 首页
│   ├── projects.rs              # 项目管理页
│   ├── history.rs               # 历史记录页
│   ├── batch.rs                 # 批量翻译页
│   └── settings.rs              # 设置页面
├── 🔗 hooks/                     # Leptos Hooks
│   ├── mod.rs
│   ├── use_auth.rs              # 认证状态管理
│   ├── use_backend_translation.rs # 后端翻译集成
│   ├── use_batch_translation.rs # 批量翻译Hook
│   └── use_config.rs            # 配置管理Hook
├── 🌐 services/                  # 业务服务层
│   ├── mod.rs
│   ├── api_client.rs            # 后端API客户端
│   ├── jina_service.rs          # Jina AI Reader服务
│   ├── deeplx_service.rs        # DeepLX翻译服务
│   ├── config_service.rs        # 配置管理服务
│   └── rate_limiter.rs          # 速率限制器
├── ⚙️ config/                    # 配置管理
│   ├── mod.rs
│   ├── env.rs                   # 环境变量管理
│   ├── features.rs              # 功能开关管理
│   └── runtime.rs               # 运行时配置
├── 🎨 theme/                     # 主题系统
│   ├── mod.rs
│   ├── catppuccin.rs           # Catppuccin主题
│   └── provider.rs             # 主题提供者
├── 🔧 types/                     # 类型定义
│   ├── mod.rs
│   ├── api_types.rs            # API数据结构
│   ├── translation.rs          # 翻译相关类型
│   └── history.rs              # 历史记录类型
├── ❌ error/                     # 错误处理
│   ├── mod.rs
│   ├── app_error.rs            # 应用错误类型
│   └── error_handler.rs        # 错误处理器
└── 🛠️ utils/                     # 工具函数
    ├── mod.rs
    ├── validation.rs           # 输入验证
    └── sanitization.rs         # 数据清理
```

### 后端结构（Axum API）

```
backend/src/
├── 🚀 main.rs                    # 后端入口
├── ⚙️ config.rs                  # 配置管理
├── 💾 database.rs                # 数据库连接
├── 🛣️ routes.rs                  # 路由配置
├── 📊 models/                    # 数据模型
│   ├── mod.rs
│   ├── user.rs                  # 用户模型
│   ├── project.rs               # 项目模型
│   ├── translation.rs           # 翻译模型
│   └── search.rs                # 搜索模型
├── 🎯 handlers/                  # API处理器
│   ├── mod.rs
│   ├── auth.rs                  # 认证API
│   ├── users.rs                 # 用户管理
│   ├── projects.rs              # 项目管理
│   ├── translations.rs          # 翻译API
│   ├── search.rs                # 搜索API
│   └── health.rs                # 健康检查
├── 🏢 services/                  # 业务服务
│   ├── mod.rs
│   ├── auth_service.rs          # 认证服务
│   ├── user_service.rs          # 用户服务
│   ├── translation_service.rs   # 翻译服务
│   ├── search_service.rs        # 搜索服务
│   └── redis_service.rs         # Redis缓存服务
├── 🛡️ middleware/                # 中间件
│   ├── mod.rs
│   ├── auth.rs                  # 认证中间件
│   ├── rate_limit.rs            # 速率限制
│   └── request_id.rs            # 请求ID追踪
└── ❌ error.rs                   # 错误定义
```

### 配置和脚本

```
📁 scripts/
├── 🛠️ dev.sh                     # 开发环境启动
├── 🏗️ build-prod.sh             # 生产构建脚本
├── ✅ validate-config.sh         # 配置验证脚本
├── 🐳 deploy.sh                  # 部署脚本
└── 💊 health-check.sh            # 健康检查脚本

📁 docs/
├── 📋 README.md                  # API文档
├── 🏗️ backend-architecture.md   # 后端架构说明
├── 🌍 environment-setup.md      # 环境配置指南
├── 🔧 getting-started.md        # 快速开始指南
└── 🩺 troubleshooting.md        # 故障排除指南
```

## 🔧 API 集成

### 后端API架构

完整的RESTful API设计，支持现代Web应用的所有需求：

#### 认证API
```bash
POST /api/v1/auth/login      # 用户登录
POST /api/v1/auth/register   # 用户注册
POST /api/v1/auth/refresh    # Token刷新
POST /api/v1/auth/logout     # 用户登出
```

#### 翻译API
```bash
POST /api/v1/translations/translate        # 翻译URL
GET  /api/v1/translations/history         # 获取历史记录
DELETE /api/v1/translations/history/{id}  # 删除翻译记录
```

#### 项目管理API
```bash
GET    /api/v1/projects           # 获取项目列表
POST   /api/v1/projects           # 创建项目
PUT    /api/v1/projects/{id}      # 更新项目
DELETE /api/v1/projects/{id}      # 删除项目
```

#### 搜索API
```bash
GET /api/v1/search                    # 搜索翻译内容
GET /api/v1/search/suggestions        # 获取搜索建议
```

### 第三方服务集成

#### Jina AI Reader
- **服务**: [Jina AI Reader](https://jina.ai/reader/)
- **端点**: `https://r.jina.ai`
- **功能**: 智能网页内容提取
- **特性**: 保持格式、处理动态内容、多语言支持

#### DeepLX 翻译
- **服务**: [DeepLX](https://github.com/OwO-Network/DeepLX)
- **端点**: 可配置的DeepL兼容API
- **功能**: 高质量机器翻译
- **特性**: 多语言支持、上下文感知、术语保护

## 💻 开发指南

### Just任务运行器

使用 `just` 命令简化开发流程：

```bash
# 项目初始化
just init                    # 安装依赖和初始化环境
just setup-env               # 设置环境变量文件

# 开发流程
just dev                     # 启动开发服务器
just check-env               # 检查开发环境
just validate                # 验证配置

# 构建和测试
just build                   # 开发构建
just build-prod              # 生产构建
just test                    # 运行测试
just clippy                  # 代码检查

# 代码质量
just fmt                     # 代码格式化
just quality                 # 完整质量检查
```

### 环境配置

详细的环境配置选项：

```bash
# 前端配置
FRONTEND_API_BASE_URL=http://localhost:3002/api/v1
FRONTEND_PORT=3001
DEFAULT_THEME=latte

# 后端配置
DATABASE_URL=postgres://admin:password@localhost:5432/markdown_manager
REDIS_URL=redis://:password@localhost:6379
MEILISEARCH_URL=http://localhost:7700

# 功能开关
ENABLE_PROJECT_MANAGEMENT=true
ENABLE_HISTORY=true
ENABLE_SEARCH=true
ENABLE_BATCH_TRANSLATION=true

# 性能配置
MAX_REQUESTS_PER_SECOND=10
MAX_TEXT_LENGTH=5000
API_CACHE_EXPIRE_MINUTES=5
```

### 开发工具链

推荐的开发环境和工具：

- **Rust**: 1.70+ (推荐最新稳定版)
- **Node.js**: 18+ (用于前端工具链)
- **Docker**: 20.10+ (容器化部署)
- **just**: 1.0+ (任务运行器)
- **IDE**: VS Code + rust-analyzer

## 🔧 运维指南

### 监控和日志

```bash
# 查看服务状态
docker-compose ps

# 查看服务日志
docker-compose logs -f backend
docker-compose logs -f frontend

# 监控资源使用
docker stats

# 健康检查
./health-check.sh -v
```

### 性能优化

#### 数据库优化
- 配置连接池大小
- 设置查询超时
- 启用查询缓存
- 定期数据备份

#### 缓存策略
- Redis缓存热点数据
- API响应缓存
- 静态资源CDN
- 数据库查询优化

#### 前端优化
- WASM代码分割
- 静态资源压缩
- 图片懒加载
- 代码缓存策略

### 安全考虑

#### 认证安全
- JWT Token安全存储
- 密码哈希(Argon2)
- 会话管理
- CORS配置

#### 数据安全
- 数据库加密
- API速率限制
- 输入验证和清理
- 敏感信息脱敏

## 🤝 贡献指南

### 贡献流程

1. **Fork项目**: 点击GitHub页面的Fork按钮
2. **创建分支**: `git checkout -b feature/amazing-feature`
3. **开发功能**: 编写代码并确保测试通过
4. **提交更改**: `git commit -m 'feat: add amazing feature'`
5. **推送分支**: `git push origin feature/amazing-feature`
6. **创建PR**: 在GitHub上创建Pull Request

### 代码规范

- **Rust代码**: 遵循官方风格指南，使用 `cargo fmt`
- **提交信息**: 使用[Conventional Commits](https://conventionalcommits.org/)
- **文档**: 为公共API提供详细文档
- **测试**: 为新功能编写单元测试

### 开发环境

```bash
# 设置开发环境
git clone https://github.com/your-username/url-translator.git
cd url-translator
just init

# 运行开发服务器
just dev

# 运行测试
just test

# 代码质量检查
just quality
```

## 📊 项目状态

### 功能完成度

| 模块 | 完成度 | 状态 |
|------|--------|------|
| 🔐 用户认证 | 100% | ✅ 完成 |
| 🌐 前端界面 | 100% | ✅ 完成 |
| 🔧 后端API | 100% | ✅ 完成 |
| 📊 项目管理 | 100% | ✅ 完成 |
| 📚 历史记录 | 100% | ✅ 完成 |
| 🔍 搜索功能 | 100% | ✅ 完成 |
| 🔄 批量翻译 | 95% | 🚧 即将完成 |
| 🎨 主题系统 | 90% | 🚧 进行中 |
| 📱 PWA支持 | 0% | ⏳ 规划中 |

### 技术债务

- [ ] 完善端到端测试覆盖
- [ ] 优化前端包大小
- [ ] 添加性能监控
- [ ] 完善错误追踪

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

感谢以下开源项目和服务：

- **框架**: [Leptos](https://leptos.dev/) - 现代化Rust全栈框架
- **后端**: [Axum](https://github.com/tokio-rs/axum) - 高性能异步Web框架
- **数据库**: [PostgreSQL](https://postgresql.org/) - 世界上最先进的开源数据库
- **搜索**: [MeiliSearch](https://meilisearch.com/) - 快速、相关的搜索引擎
- **AI服务**: [Jina AI](https://jina.ai/) - 强大的内容提取服务
- **翻译**: [DeepLX](https://github.com/OwO-Network/DeepLX) - 开源翻译API
- **样式**: [Tailwind CSS](https://tailwindcss.com/) - 现代化CSS框架
- **主题**: [Catppuccin](https://catppuccin.com/) - 优雅的配色方案

## 📞 联系我们

- 📧 **邮箱**: your-email@example.com
- 🐛 **Bug报告**: [GitHub Issues](https://github.com/your-username/url-translator/issues)
- 💡 **功能建议**: [GitHub Discussions](https://github.com/your-username/url-translator/discussions)
- 📚 **文档**: [项目文档](./docs/)

---

<div align="center">

**[🏠 首页](#url翻译工具) | [🚀 快速开始](#-快速开始) | [📖 使用指南](#-使用指南) | [🐳 Docker部署](#-docker-部署) | [🤝 贡献指南](#-贡献指南)**

[![Made with ❤️ and Rust](https://img.shields.io/badge/Made%20with-%E2%9D%A4%EF%B8%8F%20and%20Rust-orange.svg)](https://www.rust-lang.org/)

</div>