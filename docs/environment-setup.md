# 环境配置指南

本文档详细说明了URL翻译工具的环境变量配置和构建设置。

## 📁 配置文件结构

```
url-translator/
├── .env.example              # 环境变量配置示例（包含所有配置项）
├── .env.local.example        # 本地开发配置示例
├── .env.production.example   # 生产环境配置示例
├── .env                      # 本地环境配置（需要手动创建）
├── .env.local               # 本地覆盖配置（可选，不会提交到Git）
├── Trunk.toml               # Trunk构建工具配置
├── build.rs                 # Rust构建脚本
└── scripts/
    ├── dev.sh              # 开发环境启动脚本
    ├── build-prod.sh       # 生产环境构建脚本
    └── validate-config.sh  # 配置验证脚本
```

## 🚀 快速开始

### 1. 初始化环境配置

```bash
# 方法1：使用 just 命令（推荐）
just setup-env

# 方法2：手动复制
cp .env.example .env
cp .env.local.example .env.local
```

### 2. 编辑配置文件

编辑 `.env` 文件，修改以下关键配置：

```bash
# API配置
FRONTEND_API_BASE_URL=http://localhost:3002/api/v1
FRONTEND_API_TIMEOUT_SECONDS=30

# 功能开关
ENABLE_PROJECT_MANAGEMENT=true
ENABLE_HISTORY=true
ENABLE_SEARCH=true
ENABLE_BATCH_TRANSLATION=true

# UI配置
DEFAULT_THEME=latte
```

### 3. 验证配置

```bash
# 验证配置是否正确
just validate

# 检查开发环境
just check-env
```

### 4. 启动开发服务器

```bash
# 使用完整的开发脚本（推荐）
just dev

# 或使用简单模式
just dev-simple
```

## ⚙️ 环境变量详解

### API配置

| 变量名 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| `FRONTEND_API_BASE_URL` | string | `http://localhost:3002/api/v1` | 后端API的基础URL |
| `FRONTEND_API_TIMEOUT_SECONDS` | number | `30` | API请求超时时间（秒） |

### 功能开关

| 变量名 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| `ENABLE_PROJECT_MANAGEMENT` | boolean | `true` | 是否启用项目管理功能 |
| `ENABLE_HISTORY` | boolean | `true` | 是否启用历史记录功能 |
| `ENABLE_SEARCH` | boolean | `true` | 是否启用搜索功能 |
| `ENABLE_BATCH_TRANSLATION` | boolean | `true` | 是否启用批量翻译功能 |
| `ENABLE_USER_AUTHENTICATION` | boolean | `true` | 是否启用用户认证功能 |

### UI/UX配置

| 变量名 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| `DEFAULT_THEME` | string | `latte` | 默认主题 (`latte`, `frappe`, `macchiato`, `mocha`) |
| `ENABLE_THEME_SWITCHING` | boolean | `true` | 是否启用主题切换 |
| `MAX_FILE_SIZE_MB` | number | `10` | 最大文件大小限制（MB） |
| `ENABLE_KEYBOARD_SHORTCUTS` | boolean | `true` | 是否启用键盘快捷键 |

### 开发环境配置

| 变量名 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| `DEBUG_MODE` | boolean | `true` | 是否启用调试模式 |
| `RUST_LOG` | string | `debug` | Rust日志级别 |
| `WASM_LOG` | string | `debug` | WASM日志级别 |
| `ENABLE_DEVTOOLS` | boolean | `true` | 是否启用开发工具 |
| `HOT_RELOAD` | boolean | `true` | 是否启用热重载 |

### 生产环境配置

| 变量名 | 类型 | 默认值 | 描述 |
|--------|------|--------|------|
| `PRODUCTION_MODE` | boolean | `false` | 是否为生产模式 |
| `ENABLE_COMPRESSION` | boolean | `true` | 是否启用压缩 |
| `ENABLE_MINIFICATION` | boolean | `true` | 是否启用代码压缩 |
| `ENABLE_HTTPS` | boolean | `false` | 是否启用HTTPS |

## 🏗️ 构建配置

### 开发构建

```bash
# 启动开发服务器
just dev

# 开发构建（不启动服务器）
trunk build
```

### 生产构建

```bash
# 生产构建（使用脚本，包含优化）
just build-prod

# 简单生产构建
trunk build --release
```

### 构建优化

生产构建脚本 `scripts/build-prod.sh` 包含以下优化：

1. **代码检查**: 运行 `cargo clippy` 进行代码质量检查
2. **测试**: 运行单元测试确保代码质量
3. **WASM优化**: 使用 `wasm-opt` 压缩WASM文件
4. **文件压缩**: 启用gzip压缩
5. **构建信息**: 生成构建时间、版本、Git哈希等信息

## 🔧 配置验证

### 自动验证

运行配置验证脚本：

```bash
just validate
```

验证内容包括：
- ✅ API URL格式验证
- ✅ 数值范围检查
- ✅ 布尔值格式验证
- ✅ 主题名称验证
- ✅ 网络连接测试
- ✅ 构建工具检查
- ✅ 必要文件存在性检查

### 手动验证

检查关键配置项：

```bash
# 检查API连接
curl -s "$FRONTEND_API_BASE_URL/health"

# 检查构建工具
trunk --version
rustup target list --installed | grep wasm32-unknown-unknown

# 检查环境变量
echo $FRONTEND_API_BASE_URL
echo $DEFAULT_THEME
```

## 🌍 多环境配置

### 开发环境

1. 使用 `.env` 文件存储基本配置
2. 使用 `.env.local` 文件覆盖个人配置
3. 启用调试模式和开发工具

```bash
DEBUG_MODE=true
ENABLE_DEVTOOLS=true
RUST_LOG=debug
```

### 测试环境

1. 复制 `.env.example` 为 `.env.test`
2. 修改API地址指向测试服务器
3. 启用测试相关功能

```bash
FRONTEND_API_BASE_URL=https://test-api.yourdomain.com/api/v1
DEBUG_MODE=false
ENABLE_ERROR_TRACKING=true
```

### 生产环境

1. 复制 `.env.production.example` 为 `.env.production`
2. 配置生产环境API地址
3. 启用生产优化

```bash
FRONTEND_API_BASE_URL=https://api.yourdomain.com/api/v1
PRODUCTION_MODE=true
DEBUG_MODE=false
ENABLE_COMPRESSION=true
ENABLE_HTTPS=true
```

## 🔒 安全考虑

### 敏感信息处理

1. **不要提交敏感配置**: `.env*` 文件已在 `.gitignore` 中忽略
2. **使用环境变量**: 在CI/CD中使用环境变量而不是文件
3. **定期轮换**: 定期更换API密钥和敏感配置

### 配置文件优先级

配置文件的加载优先级（从高到低）：

1. 环境变量
2. `.env.local`
3. `.env`
4. `.env.example`（默认值）

## 🔍 故障排除

### 常见问题

#### 1. API连接失败

```bash
# 检查API地址
echo $FRONTEND_API_BASE_URL

# 测试连接
curl -v "$FRONTEND_API_BASE_URL/health"

# 检查防火墙和网络设置
```

#### 2. 构建失败

```bash
# 检查Rust和WASM工具链
rustc --version
trunk --version
rustup target list --installed

# 清理并重新构建
trunk clean
cargo clean
trunk build
```

#### 3. 环境变量不生效

```bash
# 验证文件是否存在
ls -la .env*

# 检查文件内容
cat .env

# 验证语法
./scripts/validate-config.sh
```

#### 4. 主题不工作

```bash
# 检查主题配置
echo $DEFAULT_THEME

# 验证主题名称
./scripts/validate-config.sh | grep THEME
```

### 调试技巧

1. **启用详细日志**:
   ```bash
   RUST_LOG=debug
   WASM_LOG=debug
   ```

2. **使用浏览器开发工具**: 检查控制台错误和网络请求

3. **检查构建输出**: 查看 `dist/` 目录中的文件

4. **验证环境变量**: 在代码中打印环境变量值

## 📚 相关文档

- [项目README](../README.md) - 项目概述和快速开始
- [开发指南](./getting-started.md) - 详细的开发环境设置
- [部署指南](./deployment.md) - 生产环境部署说明
- [故障排除](./troubleshooting.md) - 常见问题解决方案

## 🤝 贡献

如果您发现配置相关的问题或有改进建议，请：

1. 提交Issue描述问题
2. 提供详细的环境信息
3. 包含相关的配置文件内容（隐藏敏感信息）
4. 如果可能，提供修复建议或PR

---

**最后更新**: 2025-07-10
**版本**: v1.0.0