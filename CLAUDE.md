# URL内容翻译工具 - 项目文档

## 🎯 项目概述

URL内容翻译工具是一个基于Rust技术栈开发的现代化全栈Web应用，采用**文件库为核心的搜索导向架构**。系统提供高效、智能的网页内容翻译服务，以**文件库搜索**为主界面，支持用户认证、翻译历史、文件管理和实时通知系统。

**🔄 当前项目状态**: 正在进行**架构重设计**，将项目转向以Meilisearch驱动的文件库搜索为核心的用户体验，整合所有翻译功能到统一界面。

### 🎨 新界面架构设计

#### 🏠 项目首页 (文件库搜索中心)
```
┌─────────────────────────────────────────────────────────────────┐
│  🔍 文件库搜索                                          [菜单] ⚙️  │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  🔍 搜索所有翻译文件...                                      │  │
│  │      Powered by Meilisearch                                │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  📁 搜索结果 / 最近文件                                           │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │ 📄 文档标题                                [原文] [译文] [双语] │  │
│  │    翻译时间: 2024-XX-XX  大小: 125KB                         │  │
│  │    标签: #技术文档 #API参考                                   │  │
│  └─────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  🔄 快速翻译入口                                                 │
│  ┌─────────────────────────────────────────────────────────────┐  │
│  │  📎 拖拽URL或输入链接进行翻译...                              │  │
│  └─────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

#### 📋 右上角菜单组件
```
┌─────────────────┐
│ 🌐 翻译组件      │  ← 整合单页+批量翻译
│ 📚 历史组件      │  ← 用户个人翻译记录搜索
│ ⚙️  设置组件      │  ← 项目配置管理
│ 📁 文件库组件    │  ← 当前页面（高亮显示）
│ 🔔 通知 (3)     │  ← 统一通知系统
└─────────────────┘
```

### 🗃️ 文件库核心功能

#### 文件查看模式
- **原文模式**: 显示提取的原始内容（Jina Reader结果）
- **译文模式**: 显示翻译后的内容
- **双语模式**: 原文译文对照显示，支持段落级对齐

#### 文件操作功能
- **即时查看**: 点击文件可全屏查看，支持三种模式切换
- **下载功能**: 支持Markdown格式下载，包含元数据
- **搜索高亮**: Meilisearch驱动的全文搜索，支持中英文
- **标签系统**: 自动提取和手动添加文件标签

## 🏗️ 技术架构

### 核心技术栈

#### 前端技术
- **框架**: Leptos 0.6 (Rust WASM全栈框架)
- **构建工具**: Trunk + WebAssembly
- **状态管理**: Leptos Signals + 响应式编程
- **样式系统**: Tailwind CSS + Catppuccin主题
- **HTTP客户端**: 集成后端API客户端

#### 后端技术 (新增)
- **Web框架**: Axum 0.7 (高性能异步Web框架)
- **数据库**: PostgreSQL + SQLx (类型安全ORM)
- **缓存**: Redis (性能缓存和会话存储)
- **搜索**: MeiliSearch (全文搜索引擎)
- **认证**: JWT + Argon2 (安全认证和密码哈希)
- **翻译服务**: Jina AI Reader + DeepLX API

#### 基础设施
- **容器化**: Docker + Docker Compose
- **反向代理**: Nginx (生产环境)
- **部署**: 自动化脚本 + 健康检查
- **监控**: 集成日志和性能监控

### 新架构设计 (文件库为中心)
```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            前端 Leptos SPA                                  │
├─────────────────┬─────────────────┬─────────────────┬─────────────────────┤
│  🔍 文件库搜索   │  🌐 翻译组件     │  📚 历史组件     │  🔔 通知系统         │
│                │                │                │                    │
│ • Meilisearch  │ • 单页翻译       │ • 用户搜索       │ • WebSocket通知     │
│   集成搜索      │ • 批量翻译       │ • 历史记录       │ • 任务状态更新       │
│ • 文件预览      │ • 实时进度       │ • 数据导出       │ • 错误提醒          │
│ • 三种查看模式   │ • 队列管理       │                │                    │
└─────────────────┴─────────────────┴─────────────────┴─────────────────────┘
                                    │
                            JWT认证 + RESTful API + WebSocket
                                    │
┌─────────────────────────────────────────────────────────────────────────────┐
│                           后端 Axum API Gateway                             │
├─────────────────┬─────────────────┬─────────────────┬─────────────────────┤
│  🔍 搜索服务     │  🌐 翻译服务     │  📚 历史服务     │  🔔 通知服务         │
│                │                │                │                    │
│ • 文件索引管理   │ • 异步任务队列   │ • 数据持久化     │ • 实时推送          │
│ • 全文搜索API   │ • 进度跟踪       │ • 分页查询       │ • 事件聚合          │
│ • 标签系统      │ • 重试机制       │ • 数据导出       │                    │
└─────────────────┴─────────────────┴─────────────────┴─────────────────────┘
                                    │
┌─────────────────┬─────────────────┬─────────────────┬─────────────────────┐
│  PostgreSQL     │  Redis          │  MeiliSearch    │  外部API            │
│                │                │                │                    │
│ • 翻译记录存储   │ • 任务队列       │ • 文件索引       │ • Jina AI Reader   │
│ • 用户数据      │ • 缓存层        │ • 搜索引擎       │ • DeepLX翻译       │
│ • 项目管理      │ • 会话存储       │ • 实时更新       │                    │
└─────────────────┴─────────────────┴─────────────────┴─────────────────────┘
```

## 🎨 界面设计方案

### Catppuccin主题集成

项目采用现代化的Catppuccin设计系统，提供优雅的用户界面体验。

```rust
// src/theme/catppuccin.rs
use leptos::*;

#[derive(Clone, Debug)]
pub struct CatppuccinLatte {
    pub rosewater: &'static str,
    pub flamingo: &'static str,
    pub pink: &'static str,
    pub mauve: &'static str,
    pub red: &'static str,
    pub maroon: &'static str,
    pub peach: &'static str,
    pub yellow: &'static str,
    pub green: &'static str,
    pub teal: &'static str,
    pub sky: &'static str,
    pub sapphire: &'static str,
    pub blue: &'static str,
    pub lavender: &'static str,
    pub text: &'static str,
    pub subtext1: &'static str,
    pub subtext0: &'static str,
    pub overlay2: &'static str,
    pub overlay1: &'static str,
    pub overlay0: &'static str,
    pub surface2: &'static str,
    pub surface1: &'static str,
    pub surface0: &'static str,
    pub base: &'static str,
    pub mantle: &'static str,
    pub crust: &'static str,
}

pub const CATPPUCCIN_LATTE: CatppuccinLatte = CatppuccinLatte {
    rosewater: "#dc8a78",
    flamingo: "#dd7878",
    pink: "#ea76cb",
    mauve: "#8839ef",
    red: "#d20f39",
    maroon: "#e64553",
    peach: "#fe640b",
    yellow: "#df8e1d",
    green: "#40a02b",
    teal: "#179299",
    sky: "#04a5e5",
    sapphire: "#209fb5",
    blue: "#1e66f5",
    lavender: "#7287fd",
    text: "#4c4f69",
    subtext1: "#5c5f77",
    subtext0: "#6c6f85",
    overlay2: "#7c7f93",
    overlay1: "#8c8fa1",
    overlay0: "#9ca0b0",
    surface2: "#acb0be",
    surface1: "#bcc0cc",
    surface0: "#ccd0da",
    base: "#eff1f5",
    mantle: "#e6e9ef",
    crust: "#dce0e8",
};
```

### 响应式设计
- **移动优先**: 采用移动优先的响应式设计
- **断点系统**: sm(640px), md(768px), lg(1024px), xl(1280px)
- **组件适配**: 所有组件支持多设备尺寸适配

## 🚀 核心功能设计

### 1. 🔍 文件库搜索系统 (核心功能)
- **Meilisearch集成**: 高性能全文搜索引擎，支持中英文混合搜索
- **文件索引管理**: 自动为翻译文件建立搜索索引，包含原文和译文
- **实时搜索建议**: 输入即搜索，支持模糊匹配和拼写纠错
- **标签分类系统**: 自动提取文档类型、技术标签，支持手动标记
- **搜索结果预览**: 搜索结果显示匹配片段，支持高亮显示

### 2. 🌐 统一翻译组件 (整合设计)
#### 单页翻译模式
- **智能URL识别**: 自动识别和验证输入的URL格式
- **实时进度显示**: WebSocket驱动的实时翻译进度更新
- **三种输出模式**: 原文、译文、双语对照可切换显示

#### 批量翻译模式 (整合到单页)
- **批量URL输入**: 支持多行URL输入和文件导入,保持提取目录结构URL的功能
- **队列可视化**: 显示翻译队列状态和预计完成时间
- **并行处理管理**: 智能调度多个翻译任务，避免API限流



### 3. 📚 个人历史组件 (用户中心)
- **个人搜索界面**: 仅搜索当前用户的翻译文件
- **时间线浏览**: 按时间排序的翻译历史记录
- **分类筛选**: 按项目、语言、文档类型筛选历史记录
- **使用统计**: 翻译字数、文档数量、使用频率统计

### 4. 🔔 统一通知系统 (体验优化)
- **实时通知中心**: 替换所有弹窗提示，统一在右上角显示
- **通知分类**: 成功、警告、错误、信息四种类型
- **通知历史**: 保存最近通知历史，支持回顾查看
- **WebSocket推送**: 任务完成、系统维护等实时通知

### 5. ⚙️ 设置组件 (配置管理)
- **API配置管理**: DeepLX、Jina AI等服务的API密钥管理
- **翻译偏好设置**: 默认语言对、翻译模式偏好
- **界面个性化**: 主题切换、布局偏好、通知设置
- **数据管理**: 导出个人数据、清理历史记录

### 6. 📄 文件查看和管理
#### 三种查看模式
- **原文模式**: 显示Jina Reader提取的纯净原文内容
- **译文模式**: 显示翻译后的目标语言内容
- **双语模式**: 原文译文段落级对照，支持段落导航

#### 文件操作
- **全屏阅读**: 沉浸式阅读体验，支持键盘导航
- **智能下载**: 根据查看模式下载对应格式的Markdown文件
- **文件分享**: 生成只读分享链接（规划功能）
- **版本管理**: 同一URL重新翻译时保存历史版本

### 7. 🚀 Leptos框架优势展示
#### 响应式状态管理
- **信号系统**: 利用Leptos Signals实现细粒度响应式更新
- **组件通信**: 通过信号和上下文实现组件间高效通信
- **状态持久化**: 结合localStorage实现客户端状态持久化

#### 性能优化
- **懒加载路由**: 按需加载页面组件，减少初始包大小
- **虚拟滚动**: 文件列表和搜索结果的高性能渲染
- **客户端缓存**: 智能缓存搜索结果和文件内容

#### 类型安全
- **编译时检查**: Rust类型系统确保前端代码的类型安全
- **API类型共享**: 前后端共享数据类型定义，避免接口不匹配
- **错误处理**: 统一的Result类型错误处理模式

## 📁 项目结构 (新架构)

```
url-translator/
├── frontend/               # Leptos前端应用
│   ├── src/
│   │   ├── app.rs         # 主应用和路由配置
│   │   ├── main.rs        # WASM入口点
│   │   ├── components/    # UI组件模块
│   │   │   ├── mod.rs
│   │   │   ├── layout/    # 布局组件
│   │   │   │   ├── header.rs      # 顶部导航和菜单
│   │   │   │   ├── sidebar.rs     # 侧边栏 (移动端)
│   │   │   │   └── notification.rs # 通知系统
│   │   │   ├── search/    # 搜索相关组件
│   │   │   │   ├── search_bar.rs  # 搜索输入框
│   │   │   │   ├── search_results.rs # 搜索结果显示
│   │   │   │   └── file_preview.rs # 文件预览卡片
│   │   │   ├── translation/ # 翻译功能组件
│   │   │   │   ├── url_input.rs    # URL输入组件
│   │   │   │   ├── batch_input.rs  # 批量输入组件
│   │   │   │   ├── progress.rs     # 进度显示
│   │   │   │   └── result_viewer.rs # 结果查看器
│   │   │   ├── history/   # 历史记录组件
│   │   │   │   ├── history_list.rs # 历史列表
│   │   │   │   └── history_search.rs # 历史搜索
│   │   │   ├── file_library/ # 文件库组件
│   │   │   │   ├── file_viewer.rs  # 文件查看器
│   │   │   │   ├── view_mode.rs    # 查看模式切换
│   │   │   │   └── download.rs     # 下载功能
│   │   │   └── settings/  # 设置组件
│   │   │       ├── api_config.rs   # API配置
│   │   │       ├── preferences.rs  # 用户偏好
│   │   │       └── theme.rs        # 主题设置
│   │   ├── pages/         # 页面组件
│   │   │   ├── home.rs            # 首页 (文件库搜索)
│   │   │   ├── translation.rs     # 翻译页面
│   │   │   ├── history.rs         # 历史页面
│   │   │   └── settings.rs        # 设置页面
│   │   ├── services/      # 前端服务层
│   │   │   ├── mod.rs
│   │   │   ├── api_client.rs      # 后端API客户端
│   │   │   ├── websocket.rs       # WebSocket连接
│   │   │   ├── search_service.rs  # 搜索服务
│   │   │   └── notification_service.rs # 通知服务
│   │   ├── types/         # 类型定义
│   │   │   ├── mod.rs
│   │   │   ├── api_types.rs       # API数据结构
│   │   │   ├── search_types.rs    # 搜索相关类型
│   │   │   └── notification_types.rs # 通知类型
│   │   ├── utils/         # 工具函数
│   │   │   ├── mod.rs
│   │   │   ├── formatting.rs      # 格式化工具
│   │   │   ├── validation.rs      # 验证工具
│   │   │   └── storage.rs         # 本地存储
│   │   └── theme/         # 主题系统
│   │       ├── mod.rs
│   │       ├── catppuccin.rs      # Catppuccin主题
│   │       └── theme_provider.rs  # 主题提供者
│   ├── Cargo.toml         # 前端依赖
│   ├── Trunk.toml         # Trunk构建配置
│   └── public/            # 静态资源
├── backend/               # Axum后端API
│   ├── src/
│   │   ├── main.rs        # 后端入口
│   │   ├── config.rs      # 配置管理
│   │   ├── handlers/      # API处理器
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs            # 认证相关
│   │   │   ├── translation.rs     # 翻译API
│   │   │   ├── search.rs          # 搜索API
│   │   │   ├── history.rs         # 历史记录API
│   │   │   └── websocket.rs       # WebSocket处理
│   │   ├── services/      # 业务逻辑服务
│   │   │   ├── mod.rs
│   │   │   ├── translation_service.rs # 翻译服务
│   │   │   ├── search_service.rs      # Meilisearch服务
│   │   │   ├── task_queue.rs          # 任务队列
│   │   │   ├── notification_service.rs # 通知服务
│   │   │   └── file_service.rs        # 文件管理服务
│   │   ├── models/        # 数据模型
│   │   ├── database/      # 数据库连接
│   │   └── middleware/    # 中间件
│   ├── Cargo.toml         # 后端依赖
│   ├── migrations/        # 数据库迁移
│   └── config/            # 配置文件
├── shared/                # 前后端共享代码
│   ├── src/
│   │   ├── lib.rs
│   │   ├── api_types.rs   # 共享API类型
│   │   └── constants.rs   # 共享常量
│   └── Cargo.toml         # 共享依赖
├── docker-compose.yml     # 完整服务编排
├── Dockerfile.frontend    # 前端Docker镜像
├── Dockerfile.backend     # 后端Docker镜像
└── docs/                  # 项目文档
    ├── api.md             # API文档
    ├── deployment.md      # 部署指南
    └── architecture.md    # 架构说明
```

## ⚙️ 配置管理

### 默认配置参数
```rust
pub struct AppConfig {
    // API 配置
    pub deeplx_api_url: String,           // DeepLX API地址
    pub jina_api_url: String,             // Jina AI API地址

    // 语言配置
    pub default_source_lang: String,      // 默认源语言 (auto)
    pub default_target_lang: String,      // 默认目标语言 (zh)

    // 性能配置
    pub max_requests_per_second: u32,     // 速率限制 (10/秒)
    pub max_text_length: usize,          // 单次翻译最大长度 (5000字符)
    pub max_paragraphs_per_request: usize, // 单次翻译最大段落数 (10)
}
```

### 环境变量支持
```bash
# API配置
DEEPLX_API_URL=https://your-deeplx-api.com
JINA_API_URL=https://r.jina.ai

# 性能配置
MAX_REQUESTS_PER_SECOND=10
MAX_TEXT_LENGTH=5000
```

## 🛠️ 开发指南

### 本地开发环境
```bash
# 安装依赖
rustup target add wasm32-unknown-unknown
cargo install trunk

# 启动开发服务器
trunk serve --port 3000 --open

# 构建生产版本
trunk build --release
```

### Docker部署
```bash
# 快速部署
./deploy.sh

# 手动部署
docker-compose up -d

# 健康检查
./health-check.sh
```

### 代码规范
- **命名规范**: 使用snake_case命名Rust函数和变量
- **模块组织**: 按功能模块化组织代码
- **错误处理**: 统一使用Result类型处理错误
- **文档注释**: 为公共API提供详细的文档注释
- **测试覆盖**: 为核心功能编写单元测试和集成测试

## 📝 开发任务清单 (重新规划)

### ✅ 已完成功能 (基础架构)

#### 后端核心服务 (已完成)
- ✅ **Axum API服务器架构**
  - ✅ JWT认证和用户管理系统
  - ✅ PostgreSQL数据库集成
  - ✅ Redis缓存和任务队列
  - ✅ WebSocket实时通信
- ✅ **翻译服务核心逻辑**
  - ✅ Jina AI Reader内容提取
  - ✅ DeepLX翻译引擎集成
  - ✅ 智能文本分块处理
  - ✅ 异步任务队列系统
- ✅ **数据持久化和API**
  - ✅ 翻译历史记录存储
  - ✅ 用户认证API端点
  - ✅ 翻译和历史查询API

#### 前端基础框架 (已完成)
- ✅ **Leptos应用架构**
  - ✅ 基础组件系统
  - ✅ 前端翻译流程 (待迁移)
  - ✅ 本地存储和配置管理
  - ✅ Catppuccin主题系统基础

### 🚧 当前阶段：架构重设计

#### 阶段1：核心功能重构 (高优先级)
- 🔄 **文件库搜索系统**
  - ⏳ 集成Meilisearch搜索引擎
  - ⏳ 实现文件索引和搜索API
  - ⏳ 前端搜索界面组件
  - ⏳ 搜索结果展示和预览
- 🔄 **统一翻译组件**
  - ⏳ 单页翻译功能重构
  - ⏳ 批量翻译模式整合
  - ⏳ 实时进度显示优化
  - ⏳ 代码块保护机制

#### 阶段2：用户体验优化 (高优先级)
- ⏳ **统一通知系统**
  - ⏳ 替换现有弹窗提示
  - ⏳ WebSocket实时通知推送
  - ⏳ 通知历史和管理
- ⏳ **文件查看系统**
  - ⏳ 三种查看模式实现
  - ⏳ 全屏文件查看器
  - ⏳ 智能下载功能
  - ⏳ 段落级双语对照

#### 阶段3：个性化功能 (中优先级)
- ⏳ **个人历史组件**
  - ⏳ 用户专属搜索界面
  - ⏳ 历史记录筛选和排序
  - ⏳ 使用统计和分析
- ⏳ **设置和配置**
  - ⏳ API配置管理界面
  - ⏳ 用户偏好设置
  - ⏳ 主题切换和个性化

### ⏳ 高级功能规划

#### Leptos框架优势展示
- ⏳ **响应式状态管理**
  - ⏳ 信号系统深度应用
  - ⏳ 细粒度响应式更新
  - ⏳ 跨组件状态共享
- ⏳ **性能优化**
  - ⏳ 懒加载路由实现
  - ⏳ 虚拟滚动优化
  - ⏳ 客户端缓存策略
- ⏳ **类型安全**
  - ⏳ 前后端类型共享
  - ⏳ 编译时错误检查
  - ⏳ API接口类型安全

#### 高级翻译功能
- ⏳ **智能翻译增强**
  - ⏳ 专业术语词典
  - ⏳ 翻译质量评估
  - ⏳ 上下文连贯性保持
- ⏳ **协作功能**
  - ⏳ 文件分享和协作
  - ⏳ 版本控制和历史
  - ⏳ 团队权限管理

#### 系统优化和运维
- ⏳ **监控和分析**
  - ⏳ 使用统计仪表板
  - ⏳ 性能监控系统
  - ⏳ 错误追踪和报告
- ⏳ **测试和质量**
  - ⏳ 单元测试覆盖
  - ⏳ 集成测试套件
  - ⏳ 端到端测试

### 🎯 开发优先级

#### 本周重点 (P0)
1. **Meilisearch集成和搜索系统**
2. **文件库首页界面实现**
3. **统一通知系统开发**
4. **三种文件查看模式**

#### 下周计划 (P1)
1. **批量翻译功能整合**
2. **个人历史搜索界面**
3. **设置页面重构**
4. **WebSocket通知完善**

#### 月度目标 (P2)
1. **完整的Leptos响应式系统**
2. **高级翻译功能实现**
3. **性能优化和测试**
4. **文档和部署完善**

## 🎯 性能指标

### 目标性能
- **翻译响应时间**: < 5秒 (短文本), < 15秒 (长文本)
- **系统可用性**: > 99%
- **错误率**: < 1%
- **并发支持**: 100+ 用户

### 监控指标
- API请求成功率
- 平均翻译时长
- 重试频率统计
- 用户操作流程分析

## 🔒 安全考虑

### 数据安全
- **API密钥管理**: 安全的密钥存储和轮换
- **输入验证**: 严格的URL和文本内容验证
- **XSS防护**: 输出内容的安全转义
- **CORS配置**: 适当的跨域资源共享设置

### 隐私保护
- **数据最小化**: 仅处理必要的用户数据
- **本地存储**: 配置信息仅存储在用户本地
- **无日志记录**: 不记录用户翻译内容
- **透明度**: 清晰的数据使用说明

## 📚 文档资源

### 技术文档
- [Jina AI Reader服务实现](./docs/jina-service.md)
- [DeepLX翻译服务实现](./docs/deeplx-service.md)
- [故障排除指南](./docs/troubleshooting.md)
- [API参考文档](./docs/README.md)

### 快速链接
- [项目README](./README.md) - 项目概述和快速开始
- [任务清单](./Todos.md) - 详细的开发任务记录
- [Docker配置](./docker-compose.yml) - 容器化部署配置

## 🤝 贡献指南

### 参与开发
1. **Fork项目** 并创建功能分支
2. **遵循代码规范** 和最佳实践
3. **编写测试** 确保功能稳定性
4. **更新文档** 同步修改相关文档
5. **提交PR** 详细描述修改内容

### 问题反馈
- **Bug报告**: 使用GitHub Issues提交问题
- **功能建议**: 在Discussions中讨论新功能
- **文档改进**: 直接提交文档修正PR

---

## 📋 技术实现细节

### Meilisearch搜索引擎配置
```rust
// 搜索文档结构
#[derive(Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub original_content: String,
    pub translated_content: String,
    pub source_lang: String,
    pub target_lang: String,
    pub url: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub user_id: String,
}

// 搜索配置
searchable_fields: ["title", "original_content", "translated_content", "tags"]
filterable_fields: ["source_lang", "target_lang", "user_id", "tags", "created_at"]
sortable_fields: ["created_at", "title"]
```

### Leptos组件设计模式
```rust
// 响应式搜索组件示例
#[component]
pub fn FileLibrarySearch() -> impl IntoView {
    let (search_query, set_search_query) = create_signal(String::new());
    let (search_results, set_search_results) = create_signal(Vec::<SearchResult>::new());

    // 防抖搜索
    let debounced_search = create_memo(move |_| {
        let query = search_query.get();
        if query.len() >= 2 {
            spawn_local(async move {
                let results = search_files(&query).await;
                set_search_results.set(results);
            });
        }
    });

    view! {
        <div class="search-container">
            <SearchInput query=search_query set_query=set_search_query />
            <SearchResults results=search_results />
        </div>
    }
}
```

### WebSocket通知系统架构
```rust
// 通知类型定义
#[derive(Clone, Serialize, Deserialize)]
pub enum NotificationType {
    TranslationComplete { task_id: String, title: String },
    TranslationProgress { task_id: String, progress: f32 },
    TranslationError { task_id: String, error: String },
    SystemMaintenance { message: String },
}

// 通知管理器
pub struct NotificationManager {
    notifications: RwSignal<Vec<Notification>>,
    websocket: WebSocketConnection,
}
```

---

**版本**: v2.0.0 (架构重设计)
**最后更新**: 2025-07-11
**维护者**: Claude Code Assistant

**重要变更说明**:
- 🔄 **架构转向**: 从单页翻译应用转向文件库搜索为核心的用户体验
- 🔍 **搜索优先**: Meilisearch驱动的全文搜索成为主要交互方式
- 🌐 **功能整合**: 单页和批量翻译功能整合到统一翻译组件
- 🔔 **通知系统**: 统一替换所有弹窗提示，提供更好的用户体验
- 🎨 **Leptos优势**: 充分利用Leptos框架的响应式和类型安全特性
