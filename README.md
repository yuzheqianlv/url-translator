# URL翻译工具

🌐 一个基于Rust和Leptos框架的Web应用，用于提取网页内容并进行翻译，保持原始Markdown格式。

## ✨ 功能特性

- 📄 **智能内容提取**: 使用Jina AI Reader服务自动提取网页正文内容
- 🌍 **多语言翻译**: 集成DeepLX API，支持多种语言互译
- 📝 **格式保持**: 保持原始Markdown格式，确保内容结构完整
- 💾 **一键下载**: 翻译完成后可直接下载Markdown文件
- ⚙️ **灵活配置**: 支持自定义API端点和默认语言设置
- 🎨 **现代化UI**: 基于Tailwind CSS的响应式界面设计

## 🛠️ 技术栈

- **前端框架**: Leptos 0.6.x (Rust全栈框架)
- **HTTP客户端**: Reqwest 0.11.x
- **异步运行时**: Tokio 1.x
- **序列化**: Serde 1.x
- **样式框架**: Tailwind CSS 3.x + Catppuccin主题
- **构建工具**: Trunk (WASM构建工具)
- **容器化**: Docker + Nginx
- **部署工具**: Docker Compose + 自动化脚本

## 🚀 快速开始

### 环境要求

- Rust 1.70+
- Node.js (用于Tailwind CSS)

### 安装依赖

1. **安装Rust工具链**
```bash
# 安装WASM目标
rustup target add wasm32-unknown-unknown

# 安装Trunk构建工具
cargo install trunk

# 可选：安装wasm-pack
cargo install wasm-pack
```

2. **安装系统依赖** (解决OpenSSL编译问题)
```bash
# Ubuntu/Debian
sudo apt update && sudo apt install pkg-config libssl-dev

# CentOS/RHEL/Fedora
sudo dnf install pkg-config openssl-devel

# macOS
brew install pkg-config openssl
```

### 运行项目

1. **克隆项目**
```bash
git clone <repository-url>
cd url-translator
```

2. **开发模式运行**
```bash
# 自动选择可用端口
trunk serve --open

# 或指定端口运行
trunk serve --port 3000 --open

# 监听所有网络接口
trunk serve --address 0.0.0.0 --open
```
Trunk会自动选择可用端口并在浏览器中打开应用

3. **生产构建**
```bash
trunk build --release
```

### 端口配置

项目支持灵活的端口配置：

- **开发环境**: Trunk会自动选择可用端口，避免端口冲突
- **自定义端口**: 可通过 `--port` 参数指定特定端口
- **网络访问**: 使用 `--address 0.0.0.0` 允许局域网访问
- **配置文件**: 可在 `Trunk.toml` 中修改默认设置

## 🐳 Docker 部署

### 快速部署

使用增强版部署脚本，支持多种部署选项：

```bash
# 标准部署
./deploy.sh

# 指定端口部署
./deploy.sh -p 8080 deploy

# 强制重新构建
./deploy.sh -f build

# 查看帮助
./deploy.sh --help
```

### 部署命令详解

```bash
# 完整部署流程（默认）
./deploy.sh deploy

# 仅构建镜像
./deploy.sh build

# 启动容器
./deploy.sh start

# 停止容器
./deploy.sh stop

# 重启容器
./deploy.sh restart

# 查看状态
./deploy.sh status

# 查看日志
./deploy.sh logs

# 清理资源
./deploy.sh clean
```

### 健康检查

增强版健康检查工具，支持全面的状态监控：

```bash
# 基本健康检查
./health-check.sh

# 指定端口检查
./health-check.sh -p 8080

# 详细输出模式
./health-check.sh -v

# 自定义超时和重试
./health-check.sh -t 60 -r 5

# 检查远程服务器
./health-check.sh -h your-server.com -p 3000
```

健康检查包含以下验证：
- ✅ 端口连通性检查
- ✅ HTTP响应状态检查
- ✅ 响应时间监控
- ✅ Docker容器状态
- ✅ 应用功能验证

### Docker Compose 配置

项目提供生产就绪的Docker Compose配置：

```yaml
# 特性
- 健康检查和自动重启
- 网络隔离和安全配置
- Traefik标签支持
- 卷挂载和缓存优化
```

### 多环境支持

```bash
# 开发环境
./deploy.sh -d deploy    # 开发模式部署

# 生产环境
./deploy.sh deploy       # 生产模式部署

# 自定义端口
export PORT=8080
./deploy.sh deploy
```

### 访问地址

- **本地访问**: http://localhost:3000
- **自定义端口**: http://localhost:PORT
- **局域网访问**: http://your-ip:3000
- **健康检查**: http://localhost:3000/health

## 📖 使用指南

### 基本使用

1. **输入URL**: 在首页输入要翻译的网页URL
2. **开始翻译**: 点击"开始翻译"按钮
3. **查看结果**: 翻译完成后在下方查看结果
4. **下载文件**: 点击"下载Markdown"按钮保存文件

### 配置设置

访问设置页面可以自定义：
- DeepLX API端点
- Jina AI Reader端点  
- 默认源语言和目标语言

### 支持的语言

- 🇨🇳 中文 (ZH)
- 🇺🇸 英语 (EN)
- 🇯🇵 日语 (JA)
- 🇫🇷 法语 (FR)
- 🇩🇪 德语 (DE)
- 🇪🇸 西班牙语 (ES)

## 🏗️ 项目结构

```
url-translator/
├── Cargo.toml              # 项目依赖配置
├── Trunk.toml              # Trunk构建配置
├── index.html              # HTML模板
├── style.css               # 样式文件
├── src/
│   ├── main.rs             # 应用入口
│   ├── app.rs              # 主应用组件
│   ├── components/         # UI组件模块
│   │   ├── mod.rs
│   │   ├── header.rs       # 头部导航
│   │   ├── settings.rs     # 设置页面
│   │   ├── url_input.rs    # URL输入组件
│   │   ├── translation_result.rs  # 结果显示组件
│   │   └── preview_panel.rs      # 预览面板组件
│   ├── services/           # 业务服务层
│   │   ├── mod.rs
│   │   ├── jina_service.rs     # Jina AI Reader服务
│   │   ├── deeplx_service.rs   # DeepLX翻译服务
│   │   ├── config_service.rs   # 配置管理服务
│   │   └── preview_service.rs  # 预览服务
│   ├── hooks/              # Leptos钩子
│   │   ├── mod.rs
│   │   └── use_preview.rs  # 预览钩子
│   └── types/              # 类型定义
│       ├── mod.rs
│       └── api_types.rs    # API数据结构
├── docker/                 # Docker配置
│   ├── Dockerfile          # Docker镜像构建
│   ├── docker-compose.yml  # 容器编排
│   └── .dockerignore       # 构建忽略文件
├── scripts/                # 部署脚本
│   ├── deploy.sh           # 增强版部署脚本
│   └── health-check.sh     # 健康检查脚本
├── docs/                   # 项目文档
│   ├── CLAUDE.md           # 项目说明
│   └── Todos.md            # 开发任务
└── README.md               # 项目介绍
```

## 🔧 API集成

### Jina AI Reader

项目使用[Jina AI Reader](https://jina.ai/reader/)服务提取网页内容：
- 默认端点: `https://r.jina.ai`
- 支持智能内容提取和格式保持
- 自动处理动态网页内容

### DeepLX

集成[DeepLX](https://github.com/OwO-Network/DeepLX) API进行翻译：
- 默认端点: `https://api.deeplx.org/translate`
- 支持多种语言对翻译
- 保持高质量翻译效果

## ⚙️ 部署配置

### 环境变量

```bash
# API配置
DEEPLX_API_URL=https://your-deeplx-api.com
JINA_API_URL=https://r.jina.ai

# 性能配置
MAX_REQUESTS_PER_SECOND=10
MAX_TEXT_LENGTH=5000

# 部署配置
PORT=3000
NGINX_HOST=localhost
```

### 构建优化

- **依赖缓存**: Docker多阶段构建优化
- **静态资源**: Nginx高效服务
- **WASM优化**: Release模式编译
- **体积优化**: Alpine Linux基础镜像

## 🤝 贡献指南

1. Fork本项目
2. 创建feature分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 创建Pull Request

## 📝 开发计划

### ✅ 已完成功能

#### 核心功能
- ✅ URL内容提取和翻译
- ✅ Markdown格式保持
- ✅ 多语言支持
- ✅ 配置管理系统
- ✅ 批量翻译功能
- ✅ 代码块保护机制

#### 性能优化
- ✅ 速率限制器
- ✅ 重试机制
- ✅ 智能文本分块
- ✅ 异步处理优化

#### 部署优化
- ✅ Docker容器化
- ✅ 增强版部署脚本
- ✅ 健康检查系统
- ✅ 生产环境配置

### 🚧 规划中功能

#### 阶段1：架构完善
- 🔄 Catppuccin主题系统
- 📋 统一错误处理模块
- 🧪 测试框架覆盖

#### 阶段2：功能增强
- 📚 翻译历史记录
- 🎯 组件架构重构
- 🔧 增强配置管理
- 👁️ 翻译预览功能

#### 阶段3：体验优化
- 🏷️ 智能文件命名
- 📊 性能监控
- 🌐 多语言界面

#### 阶段4：高级功能
- 🔌 插件系统
- 📱 PWA支持
- 🤖 AI辅助功能
- 📈 使用统计

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

## 🙏 致谢

- [Leptos](https://leptos.dev/) - 优秀的Rust全栈框架
- [Jina AI](https://jina.ai/) - 强大的内容提取服务
- [DeepLX](https://github.com/OwO-Network/DeepLX) - 免费的翻译API
- [Tailwind CSS](https://tailwindcss.com/) - 现代化的CSS框架

---

<p align="center">
  <a href="#top">回到顶部</a>
</p>