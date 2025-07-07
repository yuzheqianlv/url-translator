use reqwest::Client;
use crate::types::api_types::{DeepLXRequest, DeepLXResponse, AppConfig};
use super::rate_limiter::{RateLimiter, RetryConfig, retry_with_backoff};

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
            return self.translate_chunk(text, source_lang, target_lang, config).await;
        }
        
        // 文本较长，需要分块处理
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

    fn split_text_into_chunks(&self, text: &str, max_length: usize, _max_paragraphs: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        
        // 简化分块逻辑：直接按字符数分割，每块5000字符
        if text.len() <= max_length {
            chunks.push(text.to_string());
            return chunks;
        }
        
        let mut start = 0;
        while start < text.len() {
            let end = std::cmp::min(start + max_length, text.len());
            
            // 尝试在合适的位置分割（避免在单词中间分割）
            let mut actual_end = end;
            if end < text.len() {
                // 向前查找最近的空格、换行或标点符号
                for i in (start..end).rev() {
                    let ch = text.chars().nth(i).unwrap_or(' ');
                    if ch == ' ' || ch == '\n' || ch == '.' || ch == '!' || ch == '?' || ch == '；' || ch == '。' {
                        actual_end = i + 1;
                        break;
                    }
                }
                
                // 如果找不到合适的分割点，就按最大长度分割
                if actual_end == end && end - start < max_length / 2 {
                    actual_end = end;
                }
            }
            
            let chunk = text[start..actual_end].trim().to_string();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }
            
            start = actual_end;
        }
        
        // 如果没有分块，返回原文本
        if chunks.is_empty() {
            chunks.push(text.to_string());
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
        web_sys::console::log_1(&format!("请求数据: {:?}", request).into());
        
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
                        web_sys::console::log_1(&format!("使用POST JSON请求到: {}", config.deeplx_api_url).into());
                        
                        client
                            .post(&config.deeplx_api_url)
                            .header("Content-Type", "application/json")
                            .header("Accept", "application/json, text/plain, */*")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| format!("DeepLX网络请求失败: {}. 可能是CORS问题或API不可用", e))?
                    } else {
                        // 标准DeepLX格式：使用POST请求和JSON body
                        client
                            .post(&config.deeplx_api_url)
                            .header("Content-Type", "application/json")
                            .header("Accept", "application/json")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| format!("DeepLX网络请求失败: {}. 可能是CORS问题或API不可用", e))?
                    };
                        
                    let status = response.status();
                    web_sys::console::log_1(&format!("DeepLX响应状态: {}", status).into());
                        
                    if response.status().is_success() {
                        let response_text = response.text().await
                            .map_err(|e| format!("读取响应文本失败: {}", e))?;
                        
                        web_sys::console::log_1(&format!("API响应内容: {}", response_text).into());
                        
                        // 尝试解析为标准DeepLX格式
                        if let Ok(result) = serde_json::from_str::<DeepLXResponse>(&response_text) {
                            web_sys::console::log_1(&format!("标准DeepLX响应代码: {}", result.code).into());
                            
                            if result.code == 200 {
                                if result.data.is_empty() {
                                    Err(Box::from("DeepLX返回了空的翻译结果") as Box<dyn std::error::Error>)
                                } else {
                                    Ok(result.data)
                                }
                            } else {
                                Err(format!("DeepLX翻译失败，返回代码: {}，可能是语言不支持或文本格式问题", result.code).into())
                            }
                        } else {
                            // 如果不是标准格式，检查是否是纯文本翻译结果
                            if response_text.trim().is_empty() {
                                Err(Box::from("API返回了空的翻译结果") as Box<dyn std::error::Error>)
                            } else if response_text.starts_with("{") {
                                // 可能是其他JSON格式，尝试提取翻译结果
                                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                    // 尝试各种可能的字段名
                                    if let Some(translated) = json_value.get("translated_text")
                                        .or_else(|| json_value.get("result"))
                                        .or_else(|| json_value.get("translation"))
                                        .or_else(|| json_value.get("data"))
                                        .and_then(|v| v.as_str()) {
                                        Ok(translated.to_string())
                                    } else {
                                        Err(format!("无法从JSON响应中提取翻译结果: {}", response_text).into())
                                    }
                                } else {
                                    Err(format!("无法解析JSON响应: {}", response_text).into())
                                }
                            } else {
                                // 假设是纯文本翻译结果
                                web_sys::console::log_1(&"假设响应是纯文本翻译结果".into());
                                Ok(response_text)
                            }
                        }
                    } else {
                        let error_text = response.text().await.unwrap_or_else(|_| "无法读取错误信息".to_string());
                        Err(format!("DeepLX API请求失败: {} - {}，请检查API地址是否正确", status, error_text).into())
                    }
                })
            },
            &retry_config,
            &self.rate_limiter,
        ).await?;
        
        Ok(result)
    }
}