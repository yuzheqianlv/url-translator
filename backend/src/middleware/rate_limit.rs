//! Rate limiting middleware

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::config::AppConfig;

/// Simple rate limiter implementation
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<IpAddr, Vec<Instant>>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let mut requests = self.requests.lock().unwrap();
        let now = Instant::now();
        
        let ip_requests = requests.entry(ip).or_insert_with(Vec::new);
        
        // Remove old requests outside the window
        ip_requests.retain(|&time| now.duration_since(time) < self.window);
        
        if ip_requests.len() >= self.max_requests {
            false
        } else {
            ip_requests.push(now);
            true
        }
    }
}

/// Create rate limiting middleware
pub fn create_rate_limit_layer(config: &AppConfig) -> RateLimiter {
    RateLimiter::new(
        config.rate_limiting.requests_per_minute as usize,
        Duration::from_secs(60),
    )
}

/// Rate limiting middleware function
pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
    rate_limiter: RateLimiter,
) -> Result<Response, StatusCode> {
    // Extract IP address from request
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|header| header.to_str().ok())
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.trim().parse::<IpAddr>().ok())
        .unwrap_or_else(|| IpAddr::from([127, 0, 0, 1])); // Default to localhost

    if rate_limiter.check_rate_limit(ip) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::TOO_MANY_REQUESTS)
    }
}