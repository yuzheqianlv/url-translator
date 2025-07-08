# URL翻译工具技术文档

## 文档导航

本目录包含URL翻译工具的详细技术文档，帮助开发者理解和维护项目。

### 📚 服务实现文档

- **[Jina AI Reader 服务](./jina-service.md)** - 网页内容提取服务的完整实现
- **[DeepLX 翻译服务](./deeplx-service.md)** - 翻译服务的核心实现和优化策略

### 🏗️ 架构概览

```
URL输入 → Jina AI Reader → 内容提取 → DeepLX翻译 → Markdown下载
    ↓           ↓              ↓           ↓            ↓
  验证URL    网页抓取      文本分块    批量翻译    格式化输出
```

### 🔧 技术栈

- **前端框架**: Leptos 0.6 (Rust全栈)
- **构建工具**: Trunk + WASM
- **HTTP客户端**: Reqwest
- **异步运行时**: 原生WASM异步
- **数据序列化**: Serde

### 📋 快速参考

#### 核心配置参数
```rust
max_requests_per_second: 10,    // 速率限制
max_text_length: 5000,          // 单次翻译最大字符数
max_paragraphs_per_request: 10, // 单次翻译最大段落数
```

#### API端点
- **Jina AI**: `https://r.jina.ai/{url}`
- **DeepLX**: `https://deepl3.fileaiwork.online/dptrans?token=...`

#### 重试策略
- **最大重试次数**: 2次
- **基础延迟**: 200ms
- **最大延迟**: 2秒
- **退避倍数**: 1.5x

### 🚀 性能特性

#### 智能翻译策略
- **短文本(≤5000字符)**: 直接翻译
- **长文本(>5000字符)**: 智能分块翻译
- **并发控制**: 速率限制保护API

#### 错误处理
- **重试机制**: 自动重试失败请求
- **降级处理**: 部分失败时继续处理
- **详细日志**: 完整的调试信息

### 🔍 调试指南

#### 常用调试命令
```javascript
// 浏览器控制台查看日志
console.log(); // 查看详细请求日志

// 检查配置
localStorage.getItem('app_config');

// 清除缓存
localStorage.clear();
```

#### 性能监控
- 翻译成功率
- 平均响应时间
- 重试频率
- 分块效率

### 📈 优化建议

1. **缓存机制**: 实现内容缓存减少重复请求
2. **并发控制**: 基于API容量动态调整并发数
3. **智能分块**: 基于内容类型优化分块策略
4. **错误恢复**: 实现更智能的错误恢复机制

### 🛠️ 开发工具

#### 本地开发
```bash
trunk serve --port 3000 --open
```

#### Docker部署
```bash
./deploy.sh
```

#### 健康检查
```bash
./health-check.sh
```

### 📝 贡献指南

1. **代码规范**: 遵循Rust官方代码风格
2. **错误处理**: 使用Result类型处理所有可能的错误
3. **日志记录**: 在关键点添加调试日志
4. **测试覆盖**: 为新功能编写单元测试
5. **文档更新**: 及时更新相关文档

### 🔗 相关链接

- [项目主README](../README.md)
- [任务清单](../Todos.md)
- [项目配置](../CLAUDE.md)
- [Docker配置](../Dockerfile)