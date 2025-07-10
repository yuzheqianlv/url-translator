# URL翻译工具后端架构设计

## 🏧 项目架构概览

本文档描述了URL翻译工具后端服务的架构设计和实现规划。

## 📦 技术栈

### 核心框架
- **语言**: Rust 2021 Edition
- **Web框架**: Axum 0.7.x (高性能、类型安全)
- **数据库ORM**: SQLx + SeaORM (异步、编译时检查)
- **缓存**: Redis + fred (高性能 Redis 客户端)
- **搜索**: MeiliSearch + meilisearch-sdk

### 支持库
- **序列化**: Serde + serde_json
- **异步运行时**: Tokio 1.x
- **HTTP客户端**: Reqwest 0.11.x
- **数据验证**: Validator + garde
- **日志**: tracing + tracing-subscriber
- **配置**: config + dotenvy
- **错误处理**: anyhow + thiserror
- **测试**: rstest + mockall

## 📝 项目目录结构

```
backend/
├── Cargo.toml              # 项目配置和依赖
├── Dockerfile             # Docker构建配置
├── .env.example           # 环境变量示例
├── src/
│   ├── main.rs            # 应用入口点
│   ├── lib.rs             # 库根模块
│   ├── config/            # 配置管理
│   │   ├── mod.rs
│   │   ├── app.rs         # 应用配置
│   │   ├── database.rs    # 数据库配置
│   │   └── redis.rs       # Redis配置
│   ├── handlers/          # HTTP请求处理器
│   │   ├── mod.rs
│   │   ├── auth.rs        # 认证相关
│   │   ├── translation.rs # 翻译相关
│   │   ├── search.rs      # 搜索相关
│   │   ├── history.rs     # 历史记录
│   │   ├── config.rs      # 配置管理
│   │   └── health.rs      # 健康检查
│   ├── services/          # 业务服务层
│   │   ├── mod.rs
│   │   ├── translation_service.rs
│   │   ├── search_service.rs
│   │   ├── user_service.rs
│   │   ├── cache_service.rs
│   │   └── notification_service.rs
│   ├── models/            # 数据模型
│   │   ├── mod.rs
│   │   ├── user.rs        # 用户模型
│   │   ├── translation.rs # 翻译模型
│   │   ├── project.rs     # 项目模型
│   │   └── search.rs      # 搜索模型
│   ├── dto/               # 数据传输对象
│   │   ├── mod.rs
│   │   ├── auth.rs        # 认证DTO
│   │   ├── translation.rs # 翻译DTO
│   │   ├── search.rs      # 搜索DTO
│   │   └── common.rs      # 通用DTO
│   ├── database/          # 数据库访问层
│   │   ├── mod.rs
│   │   ├── connection.rs  # 数据库连接
│   │   ├── migrations/    # 数据库迁移
│   │   └── repositories/  # 数据仓库
│   │       ├── mod.rs
│   │       ├── user_repository.rs
│   │       ├── translation_repository.rs
│   │       └── project_repository.rs
│   ├── middleware/        # 中间件
│   │   ├── mod.rs
│   │   ├── auth.rs        # 认证中间件
│   │   ├── cors.rs        # CORS中间件
│   │   ├── logging.rs     # 日志中间件
│   │   └── rate_limit.rs  # 限流中间件
│   ├── utils/             # 工具函数
│   │   ├── mod.rs
│   │   ├── validation.rs  # 数据验证
│   │   ├── crypto.rs      # 密码学工具
│   │   ├── jwt.rs         # JWT工具
│   │   └── file.rs        # 文件工具
│   └── errors/            # 错误处理
│       ├── mod.rs
│       ├── app_error.rs   # 应用错误
│       └── api_error.rs   # API错误
├── tests/                 # 集成测试
│   ├── common/
│   ├── integration/
│   └── api/
└── migrations/            # 数据库迁移文件
    └── *.sql
```

## 🔄 架构模式

### 分层架构
```
┌───────────────────────────────────────────────────┐
│                   API层 (Handlers)                  │
│  - HTTP路由和请求处理                            │
│  - 请求验证和响应序列化                        │
├───────────────────────────────────────────────────┤
│                 业务服务层 (Services)                │
│  - 业务逻辑实现                                │
│  - 外部API调用 (Jina, DeepLX)                   │
│  - 缓存策略和搜索集成                        │
├───────────────────────────────────────────────────┤
│                数据访问层 (Repositories)               │
│  - 数据库CRUD操作                             │
│  - 查询构建和结果映射                          │
├───────────────────────────────────────────────────┤
│                 数据存储层 (Storage)                 │
│  PostgreSQL    Redis Cache    MeiliSearch         │
│  (主数据库)     (缓存)        (搜索引擎)        │
└───────────────────────────────────────────────────┘
```

### 数据流向
1. **HTTP请求** → **中间件** (认证/限流/日志) → **Handler**
2. **Handler** → **数据验证** → **Service**
3. **Service** → **Repository/Cache/Search** → **数据库**
4. **结果** ← **DTO转换** ← **响应序列化** ← **Handler**

## 🔌 API 设计

### RESTful API 端点

#### 认证端点
```
POST   /api/v1/auth/register     # 用户注册
POST   /api/v1/auth/login        # 用户登录
POST   /api/v1/auth/logout       # 用户登出
POST   /api/v1/auth/refresh      # 刷新Token
GET    /api/v1/auth/profile      # 获取用户信息
```

#### 翻译端点
```
POST   /api/v1/translations           # 单页翻译
POST   /api/v1/translations/batch     # 批量翻译
GET    /api/v1/translations/{id}      # 获取翻译结果
GET    /api/v1/translations           # 翻译列表
DELETE /api/v1/translations/{id}      # 删除翻译
GET    /api/v1/translations/{id}/download # 下载文件
```

#### 项目管理端点
```
POST   /api/v1/projects              # 创建项目
GET    /api/v1/projects              # 项目列表
GET    /api/v1/projects/{id}         # 项目详情
PUT    /api/v1/projects/{id}         # 更新项目
DELETE /api/v1/projects/{id}         # 删除项目
GET    /api/v1/projects/{id}/stats   # 项目统计
```

#### 搜索端点
```
GET    /api/v1/search               # 全文搜索
GET    /api/v1/search/suggestions   # 搜索建议
GET    /api/v1/search/history       # 搜索历史
DELETE /api/v1/search/history/{id}  # 删除搜索历史
```

#### 配置端点
```
GET    /api/v1/config               # 获取用户配置
PUT    /api/v1/config               # 更新用户配置
GET    /api/v1/config/system        # 系统配置 (管理员)
```

#### 系统端点
```
GET    /api/v1/health               # 健康检查
GET    /api/v1/metrics              # 系统指标
GET    /api/v1/stats                # 使用统计
```

### 响应格式
```json
{
  "success": true,
  "data": {
    // 实际数据
  },
  "message": "操作成功",
  "timestamp": "2025-07-10T12:00:00Z",
  "request_id": "uuid"
}
```

### 错误响应
```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "参数验证失败",
    "details": {
      "field": "url",
      "reason": "Invalid URL format"
    }
  },
  "timestamp": "2025-07-10T12:00:00Z",
  "request_id": "uuid"
}
```

## 🔐 安全设计

### 认证和授权
- **JWT Token**: 无状态认证
- **密码加密**: Argon2id 哈希
- **权限控制**: 基于角色的访问控制

### 数据安全
- **输入验证**: 严格的参数校验
- **SQL注入防护**: 参数化查询
- **XSS防护**: 输出编码和清理
- **CSRF防护**: CSRF Token

### 网络安全
- **HTTPS**: 强制加密传输
- **CORS**: 配置允许的源
- **限流**: API调用频率限制

## 🔄 缓存策略

### Redis 缓存设计

#### 缓存分类
1. **翻译结果缓存** (TTL: 24小时)
   - Key: `translation:{url_hash}`
   - Value: JSON 格式的翻译结果

2. **用户会话缓存** (TTL: 30分钟)
   - Key: `session:{user_id}`
   - Value: 用户会话信息

3. **API限流缓存** (TTL: 1小时)
   - Key: `rate_limit:{user_id}:{endpoint}`
   - Value: 请求计数器

4. **搜索结果缓存** (TTL: 10分钟)
   - Key: `search:{query_hash}`
   - Value: 搜索结果列表

### 缓存策略
- **Cache-Aside**: 应用主动管理缓存
- **缓存穿透防护**: 布隆过滤器
- **缓存雪崩防护**: 熔断器模式

## 🔍 搜索集成

### MeiliSearch 索引设计

#### 索引结构
```rust
// 翻译内容索引
struct TranslationDocument {
    id: String,
    url: String,
    title: String,
    original_content: String,
    translated_content: String,
    source_lang: String,
    target_lang: String,
    created_at: i64,
    user_id: String,
    tags: Vec<String>,
}
```

#### 搜索配置
- **可搜索字段**: title, original_content, translated_content
- **过滤字段**: source_lang, target_lang, user_id, created_at
- **排序字段**: created_at, title
- **高亮字段**: title, content 片段

### 搜索功能
- **全文搜索**: 在所有翻译内容中搜索
- **高级筛选**: 按语言、日期等过滤
- **联想搜索**: 自动完成和建议
- **语义搜索**: 基于内容相似性

## 📋 数据库设计

### 主要实体关系
```
Users (1) ←→ (N) UserConfigs
Users (1) ←→ (N) TranslationProjects
TranslationProjects (1) ←→ (N) TranslationRecords
TranslationRecords (1) ←→ (1) TranslationContents
Users (1) ←→ (N) SearchHistory
```

### 数据库优化
- **索引优化**: 常用查询字段加索引
- **分区表**: 按时间分区大表
- **连接池**: 优化数据库连接管理
- **读写分离**: 未来支持主从复制

## 🛠️ 开发工具链

### 构建和部署
- **本地开发**: `cargo watch -x run`
- **数据库迁移**: `sqlx migrate run`
- **容器化**: Docker 多阶段构建
- **健康检查**: 内置的健康检查端点

### 代码质量
- **格式化**: `cargo fmt`
- **静态分析**: `cargo clippy`
- **测试**: `cargo test`
- **性能测试**: criterion.rs

### 版本控制
- **Git Hooks**: 预提交检查
- **CI/CD**: GitHub Actions
- **依赖管理**: Dependabot

## 📈 性能目标

### 响应时间
- **API响应**: < 100ms (P95)
- **数据库查询**: < 50ms (P95)
- **缓存命中**: > 80%
- **搜索响应**: < 200ms (P95)

### 吞吐量
- **并发请求**: 1000+ RPS
- **数据库连接**: 100+ 并发
- **缓存操作**: 10000+ OPS

### 资源使用
- **内存使用**: < 512MB
- **CPU使用**: < 2 cores
- **磁盘I/O**: 优化批量操作

## 📝 实施计划

### 阶段1: 基础框架 (1-2周)
- [x] 项目架构设计
- [ ] 基本框架搭建 (Axum + SQLx)
- [ ] 数据库连接和迁移
- [ ] 基础中间件 (日志/CORS/错误处理)
- [ ] 健康检查端点

### 阶段2: 核心功能 (2-3周)
- [ ] 用户认证系统
- [ ] 翻译API封装
- [ ] 数据库模型和仓库
- [ ] Redis缓存集成
- [ ] 基础CRUD操作

### 阶段3: 高级功能 (2-3周)
- [ ] MeiliSearch集成
- [ ] 批量翻译支持
- [ ] 搜索功能完善
- [ ] 限流和防护机制
- [ ] 文件上传下载

### 阶段4: 优化和部署 (1-2周)
- [ ] 性能优化
- [ ] 安全强化
- [ ] 监控和日志
- [ ] Docker化和部署
- [ ] 文档和测试

这个架构设计为项目提供了坚实的技术基础，支持高并发、高性能的翻译服务。通过分层架构和微服务设计，系统具有良好的可扩展性和可维护性。