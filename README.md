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
- **样式框架**: Tailwind CSS 3.x
- **构建工具**: Trunk (WASM构建工具)

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

## Docker 部署

### 快速部署

使用提供的部署脚本一键部署：

```bash
./deploy.sh
```

### 手动部署

1. **构建镜像**
```bash
docker build -t url-translator .
```

2. **运行容器**
```bash
docker run -d -p 3000:80 --name url-translator url-translator
```

3. **使用 Docker Compose**
```bash
# 启动服务
docker-compose up -d

# 停止服务
docker-compose down

# 查看日志
docker-compose logs -f
```

### 健康检查

```bash
# 检查应用状态
./health-check.sh

# 检查特定端口
./health-check.sh 3000

# 检查远程服务器
./health-check.sh 3000 your-server.com
```

### Docker 环境访问

- **本地访问**: http://localhost:3000
- **局域网访问**: http://your-ip:3000

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
├── src/
│   ├── main.rs             # 应用入口
│   ├── app.rs              # 主应用组件
│   ├── components/         # UI组件
│   │   ├── mod.rs
│   │   ├── header.rs       # 头部导航
│   │   ├── settings.rs     # 设置页面
│   │   ├── url_input.rs    # URL输入组件
│   │   └── translation_result.rs  # 结果显示组件
│   ├── services/           # 业务服务
│   │   ├── mod.rs
│   │   ├── jina_service.rs     # Jina AI服务
│   │   ├── deeplx_service.rs   # DeepLX服务
│   │   └── config_service.rs   # 配置服务
│   └── types/              # 类型定义
│       ├── mod.rs
│       └── api_types.rs    # API数据结构
└── README.md
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

## 🐳 Docker部署

```bash
# 构建镜像
docker build -t url-translator .

# 运行容器 (使用端口8080，可根据需要修改)
docker run -p 8080:80 url-translator

# 或者让Docker自动分配端口
docker run -P url-translator
```

## 🤝 贡献指南

1. Fork本项目
2. 创建feature分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送分支 (`git push origin feature/AmazingFeature`)
5. 创建Pull Request

## 📝 开发计划

### 近期功能
- [ ] 添加翻译历史记录
- [ ] 支持批量URL翻译
- [ ] 实现翻译质量评估
- [ ] 添加快捷键支持

### 性能优化
- [ ] 实现请求缓存机制
- [ ] 添加内容预处理
- [ ] 支持WebWorker处理
- [ ] 优化大文件处理

### 功能扩展
- [ ] 支持更多翻译引擎
- [ ] 添加自定义术语库
- [ ] 实现PWA功能
- [ ] 支持离线使用

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