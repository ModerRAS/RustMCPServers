//! 限流中间件

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// 限流算法类型
#[derive(Debug, Clone)]
pub enum RateLimitAlgorithm {
    /// 令牌桶算法
    TokenBucket,
    /// 滑动窗口算法
    SlidingWindow,
    /// 固定窗口算法
    FixedWindow,
}

/// 限流配置
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// 每个时间窗口的最大请求数
    pub max_requests: u32,
    /// 时间窗口大小
    pub window_size: Duration,
    /// 限流算法
    pub algorithm: RateLimitAlgorithm,
    /// 是否启用全局限流
    pub global_limit: Option<u32>,
    /// IP白名单
    pub whitelist: Vec<String>,
    /// 限流后的响应消息
    pub error_message: String,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_size: Duration::from_secs(60),
            algorithm: RateLimitAlgorithm::TokenBucket,
            global_limit: None,
            whitelist: vec![],
            error_message: "Rate limit exceeded".to_string(),
        }
    }
}

/// 限流状态
#[derive(Debug, Clone)]
struct RateLimitState {
    /// 当前令牌数（令牌桶算法）
    tokens: f64,
    /// 最后更新时间
    last_update: Instant,
    /// 窗口开始时间（滑动窗口算法）
    window_start: Instant,
    /// 窗口内请求数
    window_requests: u32,
}

/// 限流器
pub struct RateLimiter {
    /// 限流配置
    config: RateLimitConfig,
    /// 限流状态存储
    states: Arc<Mutex<HashMap<String, RateLimitState>>>,
    /// 全局请求计数
    global_counter: Arc<Mutex<u32>>,
    /// 全局计数重置时间
    global_reset_time: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    /// 创建新的限流器
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            states: Arc::new(Mutex::new(HashMap::new())),
            global_counter: Arc::new(Mutex::new(0)),
            global_reset_time: Arc::new(Mutex::new(Instant::now())),
        }
    }
    
    /// 检查是否允许请求通过
    pub fn is_allowed(&self, key: &str) -> bool {
        // 检查白名单
        if self.config.whitelist.contains(&key.to_string()) {
            debug!("Key {} is in whitelist, allowing request", key);
            return true;
        }
        
        // 检查全局限流
        if let Some(global_limit) = self.config.global_limit {
            if !self.check_global_limit(global_limit) {
                warn!("Global rate limit exceeded");
                return false;
            }
        }
        
        match self.config.algorithm {
            RateLimitAlgorithm::TokenBucket => self.check_token_bucket(key),
            RateLimitAlgorithm::SlidingWindow => self.check_sliding_window(key),
            RateLimitAlgorithm::FixedWindow => self.check_fixed_window(key),
        }
    }
    
    /// 检查令牌桶算法
    fn check_token_bucket(&self, key: &str) -> bool {
        let mut states = self.states.lock().unwrap();
        let now = Instant::now();
        
        let state = states.entry(key.to_string()).or_insert_with(|| RateLimitState {
            tokens: self.config.max_requests as f64,
            last_update: now,
            window_start: now,
            window_requests: 0,
        });
        
        // 计算令牌补充
        let time_passed = now.duration_since(state.last_update);
        let tokens_to_add = (time_passed.as_secs_f64() / self.config.window_size.as_secs_f64()) 
            * self.config.max_requests as f64;
        
        state.tokens = (state.tokens + tokens_to_add).min(self.config.max_requests as f64);
        state.last_update = now;
        
        // 检查是否有足够令牌
        if state.tokens >= 1.0 {
            state.tokens -= 1.0;
            debug!("Token bucket: {} tokens remaining for key {}", state.tokens, key);
            true
        } else {
            warn!("Token bucket: rate limit exceeded for key {}", key);
            false
        }
    }
    
    /// 检查滑动窗口算法
    fn check_sliding_window(&self, key: &str) -> bool {
        let mut states = self.states.lock().unwrap();
        let now = Instant::now();
        
        let state = states.entry(key.to_string()).or_insert_with(|| RateLimitState {
            tokens: 0.0,
            last_update: now,
            window_start: now,
            window_requests: 0,
        });
        
        // 检查是否需要重置窗口
        if now.duration_since(state.window_start) >= self.config.window_size {
            state.window_start = now;
            state.window_requests = 0;
        }
        
        // 检查窗口内请求数
        if state.window_requests < self.config.max_requests {
            state.window_requests += 1;
            debug!("Sliding window: {} requests in current window for key {}", 
                   state.window_requests, key);
            true
        } else {
            warn!("Sliding window: rate limit exceeded for key {}", key);
            false
        }
    }
    
    /// 检查固定窗口算法
    fn check_fixed_window(&self, key: &str) -> bool {
        let mut states = self.states.lock().unwrap();
        let now = Instant::now();
        
        let state = states.entry(key.to_string()).or_insert_with(|| RateLimitState {
            tokens: 0.0,
            last_update: now,
            window_start: now,
            window_requests: 0,
        });
        
        // 计算当前窗口
        let current_window = now.duration_since(Instant::now()).as_secs() 
            / self.config.window_size.as_secs();
        let state_window = state.window_start.duration_since(Instant::now()).as_secs() 
            / self.config.window_size.as_secs();
        
        // 重置窗口
        if current_window != state_window {
            state.window_start = now;
            state.window_requests = 0;
        }
        
        // 检查窗口内请求数
        if state.window_requests < self.config.max_requests {
            state.window_requests += 1;
            debug!("Fixed window: {} requests in current window for key {}", 
                   state.window_requests, key);
            true
        } else {
            warn!("Fixed window: rate limit exceeded for key {}", key);
            false
        }
    }
    
    /// 检查全局限流
    fn check_global_limit(&self, limit: u32) -> bool {
        let mut counter = self.global_counter.lock().unwrap();
        let mut reset_time = self.global_reset_time.lock().unwrap();
        
        // 检查是否需要重置计数器
        if Instant::now().duration_since(*reset_time) >= self.config.window_size {
            *counter = 0;
            *reset_time = Instant::now();
        }
        
        if *counter < limit {
            *counter += 1;
            debug!("Global rate limit: {} requests", *counter);
            true
        } else {
            warn!("Global rate limit exceeded: {} requests", *counter);
            false
        }
    }
    
    /// 获取剩余请求数
    pub fn get_remaining_requests(&self, key: &str) -> u32 {
        let states = self.states.lock().unwrap();
        
        if let Some(state) = states.get(key) {
            match self.config.algorithm {
                RateLimitAlgorithm::TokenBucket => {
                    let now = Instant::now();
                    let time_passed = now.duration_since(state.last_update);
                    let tokens_to_add = (time_passed.as_secs_f64() / self.config.window_size.as_secs_f64()) 
                        * self.config.max_requests as f64;
                    let current_tokens = (state.tokens + tokens_to_add).min(self.config.max_requests as f64);
                    current_tokens as u32
                }
                RateLimitAlgorithm::SlidingWindow | RateLimitAlgorithm::FixedWindow => {
                    self.config.max_requests.saturating_sub(state.window_requests)
                }
            }
        } else {
            self.config.max_requests
        }
    }
    
    /// 清理过期的限流状态
    pub fn cleanup_expired_states(&self) {
        let mut states = self.states.lock().unwrap();
        let now = Instant::now();
        
        states.retain(|_, state| {
            now.duration_since(state.last_update) < self.config.window_size * 2
        });
        
        debug!("Cleaned up rate limit states, remaining: {}", states.len());
    }
}

/// 限流中间件
pub struct RateLimitLayer {
    limiter: Arc<RateLimiter>,
}

impl RateLimitLayer {
    /// 创建新的限流中间件
    pub fn new(max_requests: u32, window_size: Duration) -> Self {
        let config = RateLimitConfig {
            max_requests,
            window_size,
            ..Default::default()
        };
        
        Self {
            limiter: Arc::new(RateLimiter::new(config)),
        }
    }
    
    /// 使用自定义配置创建限流中间件
    pub fn with_config(config: RateLimitConfig) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(config)),
        }
    }
    
    /// 获取限流器引用
    pub fn get_limiter(&self) -> Arc<RateLimiter> {
        self.limiter.clone()
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for RateLimitLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        // 获取客户端标识
        let client_key = self.get_client_key(&req);
        
        debug!("Checking rate limit for client: {}", client_key);
        
        // 检查限流
        if !self.limiter.is_allowed(&client_key) {
            warn!("Rate limit exceeded for client: {}", client_key);
            
            let remaining = self.limiter.get_remaining_requests(&client_key);
            let response = create_rate_limit_response(
                &self.limiter.config.error_message,
                remaining,
                self.limiter.config.window_size.as_secs(),
            );
            
            return Ok(response);
        }
        
        // 添加限流头信息
        let remaining = self.limiter.get_remaining_requests(&client_key);
        let mut req = req;
        req.headers_mut().insert(
            "x-ratelimit-remaining",
            HeaderValue::from(remaining),
        );
        req.headers_mut().insert(
            "x-ratelimit-limit",
            HeaderValue::from(self.limiter.config.max_requests),
        );
        req.headers_mut().insert(
            "x-ratelimit-window",
            HeaderValue::from(self.limiter.config.window_size.as_secs()),
        );
        
        // 继续处理请求
        let response = next.run(req).await;
        
        debug!("Request allowed for client: {}", client_key);
        
        Ok(response)
    }
}

impl RateLimitLayer {
    /// 获取客户端标识
    fn get_client_key(&self, req: &Request) -> String {
        // 尝试从各种头信息中获取客户端标识
        req.headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                req.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
            })
            .or_else(|| {
                req.headers()
                    .get("remote-addr")
                    .and_then(|h| h.to_str().ok())
            })
            .map(|s| {
                // 取第一个IP地址
                s.split(',').next().unwrap_or(s).trim().to_string()
            })
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// 创建限流响应
fn create_rate_limit_response(message: &str, remaining: u32, window_seconds: u64) -> Response {
    let body = serde_json::json!({
        "error": {
            "code": "RATE_LIMIT_EXCEEDED",
            "message": message,
            "remaining_requests": remaining,
            "window_seconds": window_seconds,
            "retry_after": window_seconds,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        }
    });
    
    Response::builder()
        .status(http::StatusCode::TOO_MANY_REQUESTS)
        .header("content-type", "application/json")
        .header("x-ratelimit-remaining", remaining)
        .header("retry-after", window_seconds)
        .body(body.to_string().into())
        .unwrap()
}

/// 全局限流器实例
static GLOBAL_RATE_LIMITER: Lazy<Arc<RateLimiter>> = Lazy::new(|| {
    let config = RateLimitConfig::default();
    Arc::new(RateLimiter::new(config))
});

/// 获取全局限流器
pub fn get_global_rate_limiter() -> Arc<RateLimiter> {
    GLOBAL_RATE_LIMITER.clone()
}

/// 定期清理过期状态的函数
pub async fn cleanup_rate_limit_states() {
    let limiter = get_global_rate_limiter();
    limiter.cleanup_expired_states();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[test]
    fn test_rate_limiter_token_bucket() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_size: Duration::from_secs(60),
            algorithm: RateLimitAlgorithm::TokenBucket,
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 前5个请求应该通过
        for i in 0..5 {
            assert!(limiter.is_allowed(&format!("test_{}", i)), 
                   "Request {} should be allowed", i);
        }
        
        // 第6个请求应该被拒绝
        assert!(!limiter.is_allowed("test_6"), 
                "Request 6 should be rate limited");
    }

    #[test]
    fn test_rate_limiter_sliding_window() {
        let config = RateLimitConfig {
            max_requests: 3,
            window_size: Duration::from_secs(1),
            algorithm: RateLimitAlgorithm::SlidingWindow,
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 前3个请求应该通过
        for i in 0..3 {
            assert!(limiter.is_allowed(&format!("test_{}", i)), 
                   "Request {} should be allowed", i);
        }
        
        // 第4个请求应该被拒绝
        assert!(!limiter.is_allowed("test_4"), 
                "Request 4 should be rate limited");
    }

    #[test]
    fn test_rate_limiter_whitelist() {
        let config = RateLimitConfig {
            max_requests: 1,
            window_size: Duration::from_secs(60),
            algorithm: RateLimitAlgorithm::TokenBucket,
            whitelist: vec!["trusted_client".to_string()],
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 白名单客户端应该始终通过
        for i in 0..10 {
            assert!(limiter.is_allowed("trusted_client"), 
                   "Whitelisted client should always be allowed (request {})", i);
        }
    }

    #[test]
    fn test_global_rate_limit() {
        let config = RateLimitConfig {
            max_requests: 10,
            window_size: Duration::from_secs(60),
            global_limit: Some(5),
            algorithm: RateLimitAlgorithm::TokenBucket,
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 前5个请求应该通过（全局限制）
        for i in 0..5 {
            assert!(limiter.is_allowed(&format!("client_{}", i)), 
                   "Request {} should be allowed", i);
        }
        
        // 第6个请求应该被拒绝（全局限制）
        assert!(!limiter.is_allowed("client_6"), 
                "Request 6 should be globally rate limited");
    }

    #[test]
    fn test_remaining_requests() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_size: Duration::from_secs(60),
            algorithm: RateLimitAlgorithm::TokenBucket,
            ..Default::default()
        };
        
        let limiter = RateLimiter::new(config);
        
        // 初始状态应该有5个剩余请求
        assert_eq!(limiter.get_remaining_requests("test"), 5);
        
        // 使用3个请求
        for i in 0..3 {
            limiter.is_allowed(&format!("test_{}", i));
        }
        
        // 应该有2个剩余请求
        assert_eq!(limiter.get_remaining_requests("test"), 2);
    }

    #[tokio::test]
    async fn test_rate_limit_layer() {
        let layer = RateLimitLayer::new(2, Duration::from_secs(60));
        
        let request = Request::builder()
            .uri("/test")
            .method("GET")
            .header("x-forwarded-for", "127.0.0.1")
            .body(Body::empty())
            .unwrap();
        
        // 这里需要测试中间件，但需要设置完整的测试环境
        // 暂时跳过这个测试
    }
}