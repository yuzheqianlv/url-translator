# DeepLX 翻译服务实现文档

## 概述

DeepLX 服务负责将提取的网页内容翻译为目标语言。该服务是URL翻译工具的核心翻译组件，支持智能文本分块、重试机制和速率限制。

## 服务架构

### 文件位置
```
src/services/deeplx_service.rs
```

### 核心结构体
```rust
pub struct DeepLXService {
    client: Client,           // HTTP客户端
    rate_limiter: RateLimiter, // 速率限制器
}
```

## API 配置

### 默认端点
- **API URL**: `https://deepl3.fileaiwork.online/dptrans?token=...`
- **请求方法**: POST
- **内容类型**: application/json

### 请求格式
```json
{
    "text": "要翻译的文本",
    "source_lang": "auto",
    "target_lang": "zh"
}
```

### 响应格式
```json
{
    "code": 200,
    "data": "翻译结果",
    "alternatives": []
}
```

## 代码实现详解

### 1. 服务初始化

```rust
impl DeepLXService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            // 速率限制：每秒10个请求，1秒窗口期
            rate_limiter: RateLimiter::new(config.max_requests_per_second, 1000),
        }
    }
}
```

### 2. 智能翻译策略

```rust
pub async fn translate(&self, text: &str, source_lang: &str, target_lang: &str, config: &AppConfig) -> Result<String, Box<dyn std::error::Error>> {
    web_sys::console::log_1(&format!("文本总长度: {} 字符", text.len()).into());
    
    // 短文本直接翻译策略
    if text.len() <= config.max_text_length {
        web_sys::console::log_1(&"文本较短，直接翻译".into());
        return self.translate_chunk(text, source_lang, target_lang, config).await;
    }
    
    // 长文本分块翻译策略
    let chunks = self.split_text_into_chunks(text, config.max_text_length, config.max_paragraphs_per_request);
    web_sys::console::log_1(&format!("文本较长，分为 {} 块进行翻译", chunks.len()).into());
    
    let mut translated_chunks = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        web_sys::console::log_1(&format!("翻译第 {} 块，长度: {} 字符", i + 1, chunk.len()).into());
        let translated_chunk = self.translate_chunk(chunk, source_lang, target_lang, config).await?;
        translated_chunks.push(translated_chunk);
    }
    
    Ok(translated_chunks.join("\n\n"))
}
```

#### 策略说明
- **≤5000字符**: 直接翻译，无需分块
- **>5000字符**: 自动分块，每块最大5000字符
- **智能拼接**: 翻译结果用双换行符连接

### 3. 智能文本分块算法

```rust
fn split_text_into_chunks(&self, text: &str, max_length: usize, _max_paragraphs: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    
    if text.len() <= max_length {
        chunks.push(text.to_string());
        return chunks;
    }
    
    let mut start = 0;
    while start < text.len() {
        let end = std::cmp::min(start + max_length, text.len());
        
        // 智能断点选择
        let mut actual_end = end;
        if end < text.len() {
            // 寻找合适的分割点
            for i in (start..end).rev() {
                let ch = text.chars().nth(i).unwrap_or(' ');
                if ch == ' ' || ch == '\n' || ch == '.' || ch == '!' || ch == '?' || ch == '；' || ch == '。' {
                    actual_end = i + 1;
                    break;
                }
            }
        }
        
        let chunk = text[start..actual_end].trim().to_string();
        if !chunk.is_empty() {
            chunks.push(chunk);
        }
        
        start = actual_end;
    }
    
    chunks
}
```

#### 分块策略
1. **优先断点**: 句号、感叹号、问号、分号
2. **次级断点**: 空格、换行符
3. **强制分割**: 找不到合适断点时按字符数分割
4. **避免空块**: 自动过滤空内容

### 4. 单块翻译实现

```rust
async fn translate_chunk(&self, text: &str, source_lang: &str, target_lang: &str, config: &AppConfig) -> Result<String, Box<dyn std::error::Error>> {
    let request = DeepLXRequest {
        text: text.to_string(),
        source_lang: source_lang.to_string(),
        target_lang: target_lang.to_string(),
    };
    
    let retry_config = RetryConfig::default();
    let client = &self.client;
    let config_clone = config.clone();
    let request_clone = request.clone();
    
    let result = retry_with_backoff(
        || {
            let client = client.clone();
            let config = config_clone.clone();
            let request = request_clone.clone();
            
            Box::pin(async move {
                // API兼容性检测
                let response = if config.deeplx_api_url.contains("dptrans") {
                    // 新格式API
                    client
                        .post(&config.deeplx_api_url)
                        .header("Content-Type", "application/json")
                        .header("Accept", "application/json, text/plain, */*")
                        .json(&request)
                        .send()
                        .await?
                } else {
                    // 标准DeepLX格式
                    client
                        .post(&config.deeplx_api_url)
                        .header("Content-Type", "application/json")
                        .header("Accept", "application/json")
                        .json(&request)
                        .send()
                        .await?
                };
                
                // 响应处理...
            })
        },
        &retry_config,
        &self.rate_limiter,
    ).await?;
    
    Ok(result)
}
```

## API 兼容性处理

### 1. 多种API格式支持

```rust
// 检测API类型
if config.deeplx_api_url.contains("dptrans") {
    // 自定义API格式
    .header("Accept", "application/json, text/plain, */*")
} else {
    // 标准DeepLX格式
    .header("Accept", "application/json")
}
```

### 2. 响应格式解析

```rust
// 标准DeepLX响应
if let Ok(result) = serde_json::from_str::<DeepLXResponse>(&response_text) {
    if result.code == 200 {
        Ok(result.data)
    } else {
        Err(format!("DeepLX翻译失败，返回代码: {}", result.code).into())
    }
}
// 纯文本响应
else if !response_text.starts_with("{") {
    Ok(response_text)
}
// 其他JSON格式
else {
    // 尝试提取常见字段
    if let Some(translated) = json_value.get("translated_text")
        .or_else(|| json_value.get("result"))
        .or_else(|| json_value.get("translation"))
        .or_else(|| json_value.get("data")) {
        Ok(translated.to_string())
    }
}
```

## 数据类型定义

### 请求结构体
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLXRequest {
    pub text: String,         // 要翻译的文本
    pub source_lang: String,  // 源语言代码
    pub target_lang: String,  // 目标语言代码
}
```

### 响应结构体
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLXResponse {
    pub code: i32,           // 状态码
    pub data: String,        // 翻译结果
    pub alternatives: Vec<String>, // 备选翻译
}
```

## 语言代码支持

### 常用语言代码
| 语言 | 代码 | 说明 |
|------|------|------|
| 自动检测 | `auto` | 自动识别源语言 |
| 中文 | `zh` | 简体中文 |
| 英语 | `en` | 英语 |
| 日语 | `ja` | 日语 |
| 法语 | `fr` | 法语 |
| 德语 | `de` | 德语 |
| 西班牙语 | `es` | 西班牙语 |

## 性能优化

### 1. 分块策略优化

```rust
// 配置参数
max_text_length: 5000,        // 单块最大字符数
max_paragraphs_per_request: 10, // 单块最大段落数
max_requests_per_second: 10,   // 每秒最大请求数
```

### 2. 重试策略优化

```rust
RetryConfig {
    max_attempts: 2,        // 减少重试次数
    base_delay_ms: 200,     // 减少基础延迟
    max_delay_ms: 2000,     // 减少最大延迟
    backoff_multiplier: 1.5, // 减少退避倍数
}
```

### 3. 速率限制优化

```rust
// 限制最大等待时间
let actual_wait = std::cmp::min(wait_time_ms as u32, 500);
```

## 错误处理

### 错误类型分类

#### 1. 网络错误
```rust
.map_err(|e| format!("DeepLX网络请求失败: {}. 可能是CORS问题或API不可用", e))
```

#### 2. API错误
```rust
if result.code != 200 {
    Err(format!("DeepLX翻译失败，返回代码: {}，可能是语言不支持或文本格式问题", result.code))
}
```

#### 3. 内容错误
```rust
if result.data.is_empty() {
    Err(Box::from("DeepLX返回了空的翻译结果"))
}
```

### 错误恢复机制
1. **自动重试**: 网络错误和临时故障
2. **降级处理**: 部分翻译失败时继续处理其他块
3. **详细日志**: 记录所有错误和调试信息

## 使用注意事项

### 1. API密钥管理
- **安全性**: 不要在前端暴露API密钥
- **配置**: 通过配置文件管理API URL
- **轮换**: 定期更换API密钥

### 2. 文本预处理
- **清理**: 移除多余的空白字符
- **格式**: 保持Markdown格式
- **编码**: 确保UTF-8编码

### 3. 并发控制
- **速率限制**: 遵守API提供商的限制
- **资源管理**: 避免过多并发请求
- **错误处理**: 优雅处理API限制

### 4. 质量保证
- **验证**: 检查翻译结果的完整性
- **格式**: 保持原文的格式结构
- **长度**: 监控翻译前后的长度变化

## 监控和调试

### 调试日志
```rust
web_sys::console::log_1(&format!("文本总长度: {} 字符", text.len()).into());
web_sys::console::log_1(&format!("文本分为 {} 块进行翻译", chunks.len()).into());
web_sys::console::log_1(&format!("翻译第 {} 块，长度: {} 字符", i + 1, chunk.len()).into());
```

### 性能指标
- 翻译成功率
- 平均翻译时间
- 分块数量统计
- 重试频率

## 配置参数详解

### AppConfig 中相关参数
```rust
pub struct AppConfig {
    pub deeplx_api_url: String,           // DeepLX API地址
    pub default_source_lang: String,      // 默认源语言
    pub default_target_lang: String,      // 默认目标语言
    pub max_requests_per_second: u32,     // 速率限制
    pub max_text_length: usize,          // 最大文本长度
    pub max_paragraphs_per_request: usize, // 最大段落数
}
```

### 默认配置
```rust
deeplx_api_url: "https://deepl3.fileaiwork.online/dptrans?token=...".to_string(),
default_source_lang: "auto".to_string(),
default_target_lang: "zh".to_string(),
max_requests_per_second: 10,
max_text_length: 5000,
max_paragraphs_per_request: 10,
```

## 故障排除

### 常见问题

1. **翻译失败**
   - 检查API密钥是否有效
   - 验证网络连接
   - 查看控制台错误日志

2. **内容丢失**
   - 检查文本分块逻辑
   - 验证API响应完整性
   - 确认字符编码正确

3. **速度慢**
   - 调整分块大小
   - 优化重试参数
   - 提高并发限制

4. **格式错乱**
   - 检查Markdown格式保持
   - 验证换行符处理
   - 确认特殊字符转义

### 调试技巧
1. **启用详细日志**: 检查所有请求和响应
2. **单步测试**: 逐个测试翻译块
3. **API测试**: 直接测试API端点
4. **配置验证**: 确认所有配置参数正确