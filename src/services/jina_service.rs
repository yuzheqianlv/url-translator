use super::rate_limiter::{retry_with_backoff, RateLimiter, RetryConfig};
use crate::types::api_types::AppConfig;
use reqwest::Client;

pub struct JinaService {
    client: Client,
    rate_limiter: RateLimiter,
}

impl JinaService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            rate_limiter: RateLimiter::new(config.max_requests_per_second, 1000), // 1秒 = 1000毫秒
        }
    }

    pub async fn extract_content(
        &self,
        url: &str,
        config: &AppConfig,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let jina_url = format!("{}/{}", config.jina_api_url, url);

        // 在控制台输出请求URL用于调试
        web_sys::console::log_1(&format!("发送请求到: {}", jina_url).into());

        let retry_config = RetryConfig::default();
        let _client = &self.client;
        let jina_url_clone = jina_url.clone();

        let result = retry_with_backoff(
            || {
                let client = self.client.clone();
                let jina_url = jina_url_clone.clone();

                Box::pin(async move {
                    web_sys::console::log_1(&format!("正在发送请求到: {}", jina_url).into());
                    let response = client
                        .get(&jina_url)
                        .header("User-Agent", "Mozilla/5.0 (compatible; URL-Translator/1.0)")
                        .header("Accept", "text/plain, text/markdown, text/html, */*")
                        .send()
                        .await
                        .map_err(|e| {
                            web_sys::console::log_1(&format!("网络请求失败: {}", e).into());
                            format!("网络请求失败: {}. 可能是CORS问题或网络连接问题", e)
                        })?;

                    let status = response.status();
                    web_sys::console::log_1(&format!("响应状态: {}", status).into());

                    if response.status().is_success() {
                        let content = response
                            .text()
                            .await
                            .map_err(|e| format!("读取响应内容失败: {}", e))?;

                        if content.is_empty() {
                            return Err(Box::from("Jina API返回了空内容，URL可能无效或无法访问")
                                as Box<dyn std::error::Error>);
                        }

                        Ok(content)
                    } else {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "无法读取错误信息".to_string());
                        Err(format!("Jina API请求失败: {} - {}", status, error_text).into())
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
