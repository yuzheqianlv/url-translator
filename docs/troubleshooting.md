# 故障排除指南

## 常见问题解决方案

### 🔧 编译和构建问题

#### 1. Rust版本不兼容
**错误信息**:
```
error: cannot install package `trunk 0.21.14`, it requires rustc 1.81.0 or newer
```

**解决方案**:
```bash
# 更新Rust到最新版本
rustup update

# 或安装特定版本
rustup install 1.88.0
rustup default 1.88.0
```

#### 2. WASM目标未安装
**错误信息**:
```
error[E0463]: can't find crate for `wasm_bindgen`
```

**解决方案**:
```bash
rustup target add wasm32-unknown-unknown
```

#### 3. Trunk未安装
**错误信息**:
```
command not found: trunk
```

**解决方案**:
```bash
cargo install trunk
```

### 🌐 网络和API问题

#### 1. CORS错误
**症状**: 浏览器控制台显示CORS错误，请求被阻止

**解决方案**:
- **开发环境**: 使用浏览器的CORS扩展
- **生产环境**: 确保API服务器配置了正确的CORS头
- **代理方案**: 通过后端代理请求

```bash
# 开发时使用Chrome禁用安全策略
google-chrome --disable-web-security --user-data-dir="/tmp/chrome_dev"
```

#### 2. API密钥失效
**症状**: 收到401/403错误

**检查步骤**:
1. 验证API密钥是否正确
2. 检查API密钥是否过期
3. 确认API配额是否用完

**解决方案**:
```rust
// 在设置页面更新API URL
deeplx_api_url: "https://your-new-api-endpoint.com"
```

#### 3. 速率限制触发
**症状**: 收到429 Too Many Requests错误

**解决方案**:
```rust
// 降低请求频率
max_requests_per_second: 5, // 从10降到5

// 或增加重试延迟
base_delay_ms: 500, // 从200ms增加到500ms
```

### 📱 用户界面问题

#### 1. 进度条卡住
**症状**: 显示"正在提取网页内容..."但没有进展

**调试步骤**:
```javascript
// 1. 检查浏览器控制台错误
console.log("检查错误信息");

// 2. 检查网络标签页
// 查看是否有失败的请求

// 3. 检查本地存储配置
console.log(localStorage.getItem('app_config'));
```

**解决方案**:
- 清除浏览器缓存和localStorage
- 检查目标URL是否可访问
- 验证API配置是否正确

#### 2. 翻译结果为空
**可能原因**:
- 目标网页内容为空
- API返回错误
- 网络连接问题

**解决方案**:
```rust
// 增加调试日志
web_sys::console::log_1(&format!("API响应: {}", response_text).into());
```

#### 3. 下载功能不工作
**症状**: 点击下载按钮没有反应

**检查项目**:
- 浏览器是否支持Blob API
- 是否有弹窗拦截
- 文件内容是否为空

### 🐳 Docker问题

#### 1. 构建失败
**常见错误**:
```
ERROR: failed to solve: process "/bin/sh -c cargo install trunk" did not complete successfully
```

**解决方案**:
```dockerfile
# 确保使用最新Rust版本
FROM rust:1.88 as builder

# 或指定trunk版本
RUN cargo install --version 0.21.0-rc.3 trunk
```

#### 2. 容器启动失败
**检查步骤**:
```bash
# 查看容器日志
docker-compose logs -f

# 检查容器状态
docker-compose ps

# 进入容器调试
docker-compose exec url-translator sh
```

#### 3. 端口冲突
**错误信息**:
```
Error starting userland proxy: listen tcp4 0.0.0.0:3000: bind: address already in use
```

**解决方案**:
```yaml
# 修改docker-compose.yml中的端口映射
ports:
  - "3001:80"  # 改为3001或其他可用端口
```

### ⚡ 性能问题

#### 1. 翻译速度慢
**优化策略**:
```rust
// 增加并发数
max_requests_per_second: 15,

// 减少重试延迟
base_delay_ms: 100,

// 优化分块大小
max_text_length: 7000,
```

#### 2. 内存占用高
**解决方案**:
- 减少同时处理的文本块数量
- 及时清理不需要的数据
- 优化文本分块算法

#### 3. 页面响应慢
**检查项目**:
- WASM文件大小
- 网络请求数量
- 浏览器兼容性

### 🔍 调试技巧

#### 1. 启用详细日志
```rust
// 在关键位置添加日志
web_sys::console::log_1(&"调试信息".into());

// 检查错误详情
web_sys::console::error_1(&format!("错误: {}", error).into());
```

#### 2. 网络调试
```javascript
// 监控网络请求
performance.getEntriesByType('navigation');
performance.getEntriesByType('resource');

// 检查本地存储
Object.keys(localStorage).forEach(key => {
    console.log(key, localStorage.getItem(key));
});
```

#### 3. 性能分析
```bash
# 分析构建大小
trunk build --release
du -sh dist/*

# 检查WASM文件大小
ls -lh dist/*.wasm
```

### 📊 监控和诊断

#### 1. 健康检查
```bash
# 使用提供的健康检查脚本
./health-check.sh

# 手动检查
curl -I http://localhost:3000
```

#### 2. 日志分析
```bash
# Docker日志
docker-compose logs --tail=100 -f

# 系统资源
docker stats
```

#### 3. 性能指标
关注以下指标:
- 请求成功率 (>95%)
- 平均响应时间 (<5秒)
- 重试频率 (<10%)
- 内存使用量 (<100MB)

### 🔧 配置优化

#### 开发环境配置
```rust
AppConfig {
    max_requests_per_second: 5,  // 降低开发时的请求频率
    max_text_length: 3000,       // 减少开发时的文本长度
    // ... 其他配置
}
```

#### 生产环境配置
```rust
AppConfig {
    max_requests_per_second: 15, // 生产环境可以更高
    max_text_length: 8000,       // 处理更大的文本
    // ... 其他配置
}
```

### 📞 获取帮助

当遇到无法解决的问题时:

1. **检查日志**: 首先查看浏览器控制台和服务器日志
2. **复现问题**: 尝试在不同环境中复现问题
3. **最小化案例**: 创建最小的复现案例
4. **查看文档**: 检查相关API文档
5. **社区求助**: 在相关技术社区寻求帮助

### 🛡️ 预防措施

1. **定期更新**: 保持依赖项的最新版本
2. **错误处理**: 在所有可能失败的地方添加错误处理
3. **监控告警**: 设置关键指标的监控告警
4. **备份方案**: 为关键服务准备备用API
5. **测试覆盖**: 编写充分的测试用例