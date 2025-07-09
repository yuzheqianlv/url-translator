use super::rate_limiter::{retry_with_backoff, RateLimiter, RetryConfig};
use crate::types::api_types::{AppConfig, DeepLXRequest, DeepLXResponse};
use reqwest::Client;

/// DeepLX 翻译服务
/// 
/// 这个服务提供了与 DeepLX API 的集成，支持高质量的机器翻译。
/// 包含以下功能：
/// - 智能文本分块处理
/// - 速率限制和重试机制
/// - 多种 API 格式支持
/// - 错误处理和恢复
/// 
/// # 使用方法
/// 
/// ```rust
/// let config = AppConfig::default();
/// let service = DeepLXService::new(&config);
/// 
/// let translated = service.translate(
///     "Hello, world!",
///     "en",
///     "zh",
///     &config
/// ).await?;
/// ```
pub struct DeepLXService {
    client: Client,
    rate_limiter: RateLimiter,
}

impl DeepLXService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            rate_limiter: RateLimiter::new(config.max_requests_per_second, 1000), // 1秒 = 1000毫秒
        }
    }

    /// 翻译文本
    /// 
    /// 这个方法提供了智能的文本翻译功能，能够：
    /// - 自动判断是否需要分块处理长文本
    /// - 对短文本进行直接翻译
    /// - 对长文本进行分块翻译并合并结果
    /// - 处理各种 API 响应格式
    /// 
    /// # 参数
    /// 
    /// * `text` - 要翻译的文本内容
    /// * `source_lang` - 源语言代码 (如 "en", "auto")
    /// * `target_lang` - 目标语言代码 (如 "zh", "ja")
    /// * `config` - 应用配置，包含 API 设置和限制参数
    /// 
    /// # 返回值
    /// 
    /// 返回 `Result<String, Box<dyn std::error::Error>>`：
    /// - `Ok(String)` - 翻译成功，包含翻译后的文本
    /// - `Err(...)` - 翻译失败，包含错误信息
    /// 
    /// # 错误处理
    /// 
    /// 可能的错误包括：
    /// - 网络连接失败
    /// - API 限制或认证错误
    /// - 无效的语言代码
    /// - 响应格式解析错误
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// let service = DeepLXService::new(&config);
    /// 
    /// // 翻译短文本
    /// let result = service.translate(
    ///     "Hello, world!",
    ///     "en",
    ///     "zh",
    ///     &config
    /// ).await?;
    /// 
    /// // 翻译长文本（自动分块）
    /// let long_text = "很长的文本内容...";
    /// let result = service.translate(
    ///     long_text,
    ///     "auto",
    ///     "en",
    ///     &config
    /// ).await?;
    /// ```
    pub async fn translate(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
        config: &AppConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        web_sys::console::log_1(&format!("文本总长度: {} 字符", text.len()).into());

        // 如果文本长度小于等于最大长度，直接翻译
        if text.len() <= config.max_text_length {
            web_sys::console::log_1(&"文本较短，直接翻译".into());
            return self
                .translate_chunk(text, source_lang, target_lang, config)
                .await;
        }

        // 文本较长，需要分块处理
        let chunks = self.split_text_into_chunks(
            text,
            config.max_text_length,
            config.max_paragraphs_per_request,
        );
        web_sys::console::log_1(&format!("文本较长，分为 {} 块进行翻译", chunks.len()).into());

        let mut translated_chunks = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            web_sys::console::log_1(
                &format!("翻译第 {} 块，长度: {} 字符", i + 1, chunk.len()).into(),
            );

            let translated_chunk = self
                .translate_chunk(chunk, source_lang, target_lang, config)
                .await?;
            translated_chunks.push(translated_chunk);
        }

        Ok(translated_chunks.join("\n\n"))
    }

    /// 智能文本分块处理
    /// 
    /// 这个方法实现了智能的文本分块策略：
    /// 1. 优先从空行处分割（双换行符）
    /// 2. 其次从段落分隔符分割（单换行符）
    /// 3. 最后从句子边界分割（句号等标点）
    /// 4. 确保每块不超过最大长度限制
    /// 
    /// # 参数
    /// 
    /// * `text` - 要分块的文本内容
    /// * `max_length` - 每块的最大字符数
    /// * `max_paragraphs` - 每块的最大段落数（暂未使用）
    /// 
    /// # 返回值
    /// 
    /// 返回字符串向量，每个元素是一个文本块
    fn split_text_into_chunks(
        &self,
        text: &str,
        max_length: usize,
        _max_paragraphs: usize,
    ) -> Vec<String> {
        let mut chunks = Vec::new();

        // 如果文本长度小于等于最大长度，直接返回
        if text.len() <= max_length {
            chunks.push(text.to_string());
            return chunks;
        }

        web_sys::console::log_1(&"开始智能文本分块处理".into());

        // 首先尝试从空行处分割（双换行符）
        let empty_line_splits: Vec<&str> = text.split("\n\n").collect();
        
        if empty_line_splits.len() > 1 {
            web_sys::console::log_1(&format!("发现 {} 个空行分割点", empty_line_splits.len()).into());
            
            let mut current_chunk = String::new();
            
            for section in empty_line_splits {
                let section_text = section.trim();
                if section_text.is_empty() {
                    continue;
                }
                
                // 检查加入这个段落是否会超过长度限制
                let test_chunk = if current_chunk.is_empty() {
                    section_text.to_string()
                } else {
                    format!("{}\n\n{}", current_chunk, section_text)
                };
                
                if test_chunk.len() <= max_length {
                    // 不会超过限制，加入当前块
                    current_chunk = test_chunk;
                } else {
                    // 会超过限制，保存当前块并开始新块
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                        web_sys::console::log_1(&format!("空行分割创建块，长度: {} 字符", current_chunk.len()).into());
                    }
                    
                    // 检查单个段落是否超过长度限制
                    if section_text.len() > max_length {
                        // 单个段落过长，需要进一步分割
                        let sub_chunks = self.split_long_section(section_text, max_length);
                        chunks.extend(sub_chunks);
                        current_chunk = String::new();
                    } else {
                        current_chunk = section_text.to_string();
                    }
                }
            }
            
            // 添加最后一个块
            if !current_chunk.is_empty() {
                chunks.push(current_chunk);
            }
        } else {
            // 没有空行分割点，尝试从段落分割
            web_sys::console::log_1(&"没有空行分割点，尝试段落分割".into());
            let paragraph_splits: Vec<&str> = text.split('\n').collect();
            
            if paragraph_splits.len() > 1 {
                let mut current_chunk = String::new();
                
                for paragraph in paragraph_splits {
                    let paragraph_text = paragraph.trim();
                    if paragraph_text.is_empty() {
                        continue;
                    }
                    
                    let test_chunk = if current_chunk.is_empty() {
                        paragraph_text.to_string()
                    } else {
                        format!("{}\n{}", current_chunk, paragraph_text)
                    };
                    
                    if test_chunk.len() <= max_length {
                        current_chunk = test_chunk;
                    } else {
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.clone());
                            web_sys::console::log_1(&format!("段落分割创建块，长度: {} 字符", current_chunk.len()).into());
                        }
                        
                        if paragraph_text.len() > max_length {
                            let sub_chunks = self.split_long_section(paragraph_text, max_length);
                            chunks.extend(sub_chunks);
                            current_chunk = String::new();
                        } else {
                            current_chunk = paragraph_text.to_string();
                        }
                    }
                }
                
                if !current_chunk.is_empty() {
                    chunks.push(current_chunk);
                }
            } else {
                // 没有段落分割点，使用句子边界分割
                web_sys::console::log_1(&"没有段落分割点，使用句子边界分割".into());
                let sub_chunks = self.split_long_section(text, max_length);
                chunks.extend(sub_chunks);
            }
        }

        // 如果没有分块，返回原文本
        if chunks.is_empty() {
            chunks.push(text.to_string());
        }

        web_sys::console::log_1(&format!("智能分块完成，共 {} 个块", chunks.len()).into());
        chunks
    }

    /// 分割过长的段落
    /// 
    /// 当单个段落超过最大长度时，使用句子边界进行分割
    fn split_long_section(&self, text: &str, max_length: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;
        
        while start < text.len() {
            let end = std::cmp::min(start + max_length, text.len());
            
            let mut actual_end = end;
            if end < text.len() {
                // 向前查找最近的句子边界
                for i in (start..end).rev() {
                    if let Some(ch) = text.chars().nth(i) {
                        if ch == '.' || ch == '!' || ch == '?' || ch == '。' || ch == '！' || ch == '？' {
                            actual_end = i + 1;
                            break;
                        }
                    }
                }
                
                // 如果找不到句子边界，查找空格或换行
                if actual_end == end {
                    for i in (start..end).rev() {
                        if let Some(ch) = text.chars().nth(i) {
                            if ch == ' ' || ch == '\n' || ch == '\t' {
                                actual_end = i + 1;
                                break;
                            }
                        }
                    }
                }
            }
            
            let chunk = text[start..actual_end].trim().to_string();
            if !chunk.is_empty() {
                let chunk_len = chunk.len();
                chunks.push(chunk);
                web_sys::console::log_1(&format!("句子边界分割创建块，长度: {} 字符", chunk_len).into());
            }
            
            start = actual_end;
        }
        
        chunks
    }

    async fn translate_chunk(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
        config: &AppConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request = DeepLXRequest {
            text: text.to_string(),
            source_lang: source_lang.to_string(),
            target_lang: target_lang.to_string(),
        };

        web_sys::console::log_1(&format!("发送翻译请求到: {}", config.deeplx_api_url).into());
        web_sys::console::log_1(&format!("翻译文本长度: {} 字符", text.len()).into());
        web_sys::console::log_1(&format!("请求数据: {request:?}").into());

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
                    // 检查是否是新格式的API (带有token和参数的URL)
                    let response = if config.deeplx_api_url.contains("dptrans") {
                        // 新格式：使用POST请求和JSON数据
                        web_sys::console::log_1(
                            &format!("使用POST JSON请求到: {}", config.deeplx_api_url).into(),
                        );

                        client
                            .post(&config.deeplx_api_url)
                            .header("Content-Type", "application/json")
                            .header("Accept", "application/json, text/plain, */*")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| {
                                format!("DeepLX网络请求失败: {e}. 可能是CORS问题或API不可用")
                            })?
                    } else {
                        // 标准DeepLX格式：使用POST请求和JSON body
                        client
                            .post(&config.deeplx_api_url)
                            .header("Content-Type", "application/json")
                            .header("Accept", "application/json")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| {
                                format!("DeepLX网络请求失败: {e}. 可能是CORS问题或API不可用")
                            })?
                    };

                    let status = response.status();
                    web_sys::console::log_1(&format!("DeepLX响应状态: {status}").into());

                    if response.status().is_success() {
                        let response_text = response
                            .text()
                            .await
                            .map_err(|e| format!("读取响应文本失败: {e}"))?;

                        web_sys::console::log_1(&format!("API响应内容: {response_text}").into());

                        // 尝试解析为标准DeepLX格式
                        if let Ok(result) = serde_json::from_str::<DeepLXResponse>(&response_text) {
                            web_sys::console::log_1(
                                &format!("标准DeepLX响应代码: {}", result.code).into(),
                            );

                            if result.code == 200 {
                                if result.data.is_empty() {
                                    Err(Box::from("DeepLX返回了空的翻译结果")
                                        as Box<dyn std::error::Error>)
                                } else {
                                    Ok(result.data)
                                }
                            } else {
                                Err(format!(
                                    "DeepLX翻译失败，返回代码: {}，可能是语言不支持或文本格式问题",
                                    result.code
                                )
                                .into())
                            }
                        } else {
                            // 如果不是标准格式，检查是否是纯文本翻译结果
                            if response_text.trim().is_empty() {
                                Err(Box::from("API返回了空的翻译结果")
                                    as Box<dyn std::error::Error>)
                            } else if response_text.starts_with("{") {
                                // 可能是其他JSON格式，尝试提取翻译结果
                                if let Ok(json_value) =
                                    serde_json::from_str::<serde_json::Value>(&response_text)
                                {
                                    // 尝试各种可能的字段名
                                    if let Some(translated) = json_value
                                        .get("translated_text")
                                        .or_else(|| json_value.get("result"))
                                        .or_else(|| json_value.get("translation"))
                                        .or_else(|| json_value.get("data"))
                                        .and_then(|v| v.as_str())
                                    {
                                        Ok(translated.to_string())
                                    } else {
                                        Err(format!(
                                            "无法从JSON响应中提取翻译结果: {response_text}"
                                        )
                                        .into())
                                    }
                                } else {
                                    Err(format!("无法解析JSON响应: {response_text}").into())
                                }
                            } else {
                                // 假设是纯文本翻译结果
                                web_sys::console::log_1(&"假设响应是纯文本翻译结果".into());
                                Ok(response_text)
                            }
                        }
                    } else {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "无法读取错误信息".to_string());
                        Err(format!(
                            "DeepLX API请求失败: {status} - {error_text}，请检查API地址是否正确"
                        )
                        .into())
                    }
                })
            },
            &retry_config,
            &self.rate_limiter,
        )
        .await?;

        Ok(result)
    }
}
