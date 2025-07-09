use js_sys::Promise;
use std::sync::Arc;
use wasm_bindgen_futures::JsFuture;

pub struct RateLimiter {
    max_requests: u32,
    window_duration_ms: u32,
    last_requests: Arc<std::sync::Mutex<Vec<f64>>>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_duration_ms: u32) -> Self {
        Self {
            max_requests,
            window_duration_ms,
            last_requests: Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    pub async fn acquire(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = js_sys::Date::now(); // 获取当前时间戳（毫秒）

        // 清理过期的请求记录
        {
            let mut requests = self.last_requests.lock().unwrap();
            requests.retain(|&time| now - time < self.window_duration_ms as f64);
        }

        // 检查是否超过速率限制
        let current_count = {
            let requests = self.last_requests.lock().unwrap();
            requests.len() as u32
        };

        if current_count >= self.max_requests {
            // 计算需要等待的时间
            let oldest_request = {
                let requests = self.last_requests.lock().unwrap();
                requests.first().copied()
            };

            if let Some(oldest) = oldest_request {
                let wait_time_ms = self.window_duration_ms as f64 - (now - oldest);
                if wait_time_ms > 0.0 {
                    // 最少等待100ms，避免过长等待
                    let actual_wait = std::cmp::min(wait_time_ms as u32, 500);
                    web_sys::console::log_1(
                        &format!("速率限制触发，等待 {}ms", actual_wait).into(),
                    );
                    self.sleep_ms(actual_wait).await;
                }
            }
        }

        // 记录这次请求
        {
            let mut requests = self.last_requests.lock().unwrap();
            requests.push(now);
        }

        Ok(())
    }

    async fn sleep_ms(&self, duration_ms: u32) {
        let promise = Promise::new(&mut |resolve, _| {
            web_sys::window()
                .unwrap()
                .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, duration_ms as i32)
                .unwrap();
        });

        let _ = JsFuture::from(promise).await;
    }
}

pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u32,
    pub max_delay_ms: u32,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 2,         // 减少重试次数
            base_delay_ms: 200,      // 减少基础延迟
            max_delay_ms: 2000,      // 减少最大延迟
            backoff_multiplier: 1.5, // 减少退避倍数
        }
    }
}

pub async fn retry_with_backoff<F, T, E>(
    operation: F,
    config: &RetryConfig,
    rate_limiter: &RateLimiter,
) -> Result<T, E>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
    E: std::fmt::Display,
{
    let mut delay_ms = config.base_delay_ms;

    for attempt in 1..=config.max_attempts {
        // 应用速率限制
        if let Err(e) = rate_limiter.acquire().await {
            web_sys::console::log_1(&format!("速率限制错误: {}", e).into());
        }

        web_sys::console::log_1(&format!("尝试请求 (第 {} 次)", attempt).into());

        let result = operation().await;

        match result {
            Ok(value) => {
                if attempt > 1 {
                    web_sys::console::log_1(&format!("重试成功 (第 {} 次尝试)", attempt).into());
                }
                return Ok(value);
            }
            Err(e) => {
                if attempt == config.max_attempts {
                    web_sys::console::log_1(&format!("所有重试都失败了: {}", e).into());
                    return Err(e);
                }

                web_sys::console::log_1(
                    &format!(
                        "第 {} 次尝试失败: {}，等待 {}ms 后重试",
                        attempt, e, delay_ms
                    )
                    .into(),
                );

                // 指数退避延迟
                let promise = Promise::new(&mut |resolve, _| {
                    web_sys::window()
                        .unwrap()
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            &resolve,
                            delay_ms as i32,
                        )
                        .unwrap();
                });

                let _ = JsFuture::from(promise).await;

                delay_ms = std::cmp::min(
                    (delay_ms as f64 * config.backoff_multiplier) as u32,
                    config.max_delay_ms,
                );
            }
        }
    }

    unreachable!()
}
