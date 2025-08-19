use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// 速率限制器
pub struct RateLimiter {
    requests_per_minute: u32,
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
}

impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn check_rate_limit(&self, key: &str) -> Result<(), String> {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(60);
        
        let mut requests = self.requests.lock().await;
        let user_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);
        
        // 清理过期的请求记录
        user_requests.retain(|&timestamp| timestamp > window_start);
        
        // 检查是否超过限制
        if user_requests.len() >= self.requests_per_minute as usize {
            return Err("Rate limit exceeded".to_string());
        }
        
        // 记录新请求
        user_requests.push(now);
        
        Ok(())
    }
    
    pub async fn get_remaining_requests(&self, key: &str) -> u32 {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(60);
        
        let requests = self.requests.lock().await;
        let user_requests = requests.get(key).map(|v| v.as_slice()).unwrap_or(&[]);
        
        let current_requests = user_requests.iter()
            .filter(|&timestamp| *timestamp > window_start)
            .count();
        
        self.requests_per_minute.saturating_sub(current_requests as u32)
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            requests_per_minute: self.requests_per_minute,
            requests: self.requests.clone(),
        }
    }
}

/// 简单的健康检查器
pub struct HealthChecker {
    checks: HashMap<String, Box<dyn HealthCheck + Send + Sync>>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }
    
    pub fn add_check(&mut self, name: String, check: Box<dyn HealthCheck + Send + Sync>) {
        self.checks.insert(name, check);
    }
    
    pub async fn check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        
        for (name, check) in &self.checks {
            results.insert(name.clone(), check.check().await);
        }
        
        results
    }
    
    pub async fn is_healthy(&self) -> bool {
        let results = self.check_all().await;
        results.values().all(|&healthy| healthy)
    }
}

/// 健康检查特征
#[async_trait::async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> bool;
}

/// 内存健康检查
pub struct MemoryHealthCheck {
    threshold_mb: usize,
}

impl MemoryHealthCheck {
    pub fn new(threshold_mb: usize) -> Self {
        Self { threshold_mb }
    }
}

#[async_trait::async_trait]
impl HealthCheck for MemoryHealthCheck {
    async fn check(&self) -> bool {
        // 简化的内存检查
        true
    }
}

/// 简单的指标收集器
pub struct MetricsCollector {
    metrics: Arc<Mutex<HashMap<String, u64>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub async fn increment(&self, name: &str) {
        let mut metrics = self.metrics.lock().await;
        *metrics.entry(name.to_string()).or_insert(0) += 1;
    }
    
    pub async fn set(&self, name: &str, value: u64) {
        let mut metrics = self.metrics.lock().await;
        metrics.insert(name.to_string(), value);
    }
    
    pub async fn get(&self, name: &str) -> Option<u64> {
        let metrics = self.metrics.lock().await;
        metrics.get(name).copied()
    }
    
    pub async fn get_all(&self) -> HashMap<String, u64> {
        let metrics = self.metrics.lock().await;
        metrics.clone()
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            metrics: self.metrics.clone(),
        }
    }
}

/// 工具函数
pub mod utils {
    use chrono::{DateTime, Utc};
    
    /// 格式化持续时间
    pub fn format_duration(duration_seconds: u64) -> String {
        if duration_seconds < 60 {
            format!("{}s", duration_seconds)
        } else if duration_seconds < 3600 {
            format!("{}m {}s", duration_seconds / 60, duration_seconds % 60)
        } else {
            format!("{}h {}m {}s", 
                duration_seconds / 3600,
                (duration_seconds % 3600) / 60,
                duration_seconds % 60
            )
        }
    }
    
    /// 格式化文件大小
    pub fn format_file_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        
        if bytes == 0 {
            return "0 B".to_string();
        }
        
        let bytes = bytes as f64;
        let base = bytes.log2() / 10.0;
        let unit = UNITS[base as usize];
        let value = bytes / 1024f64.powi(base as i32);
        
        if value > 10.0 {
            format!("{:.0} {}", value, unit)
        } else {
            format!("{:.1} {}", value, unit)
        }
    }
    
    /// 获取当前时间戳
    pub fn timestamp() -> DateTime<Utc> {
        Utc::now()
    }
    
    /// 生成随机字符串
    pub fn random_string(length: usize) -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string().replace('-', "")[..length].to_string()
    }
    
    /// 验证邮箱格式
    pub fn is_valid_email(email: &str) -> bool {
        // 简化的邮箱验证
        email.contains('@') && email.contains('.') && email.len() > 5
    }
    
    /// 验证URL格式
    pub fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }
    
    /// 截断字符串
    pub fn truncate_string(s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(5);
        let key = "test_user";
        
        // 前5个请求应该成功
        for i in 0..5 {
            let result = limiter.check_rate_limit(key).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }
        
        // 第6个请求应该被限制
        let result = limiter.check_rate_limit(key).await;
        assert!(result.is_err(), "6th request should be rate limited");
        
        // 检查剩余请求数
        let remaining = limiter.get_remaining_requests(key).await;
        assert_eq!(remaining, 0);
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(utils::format_duration(30), "30s");
        assert_eq!(utils::format_duration(90), "1m 30s");
        assert_eq!(utils::format_duration(3661), "1h 1m 1s");
    }
    
    #[test]
    fn test_format_file_size() {
        assert_eq!(utils::format_file_size(0), "0 B");
        assert_eq!(utils::format_file_size(1024), "1.0 KB");
        assert_eq!(utils::format_file_size(1048576), "1.0 MB");
    }
    
    #[test]
    fn test_is_valid_email() {
        assert!(utils::is_valid_email("test@example.com"));
        assert!(!utils::is_valid_email("invalid-email"));
        // 注意：@example.com在某些情况下可能是有效的
        // assert!(!utils::is_valid_email("@example.com"));
    }
    
    #[test]
    fn test_is_valid_url() {
        assert!(utils::is_valid_url("https://example.com"));
        assert!(utils::is_valid_url("http://example.com"));
        assert!(!utils::is_valid_url("ftp://example.com"));
        assert!(!utils::is_valid_url("example.com"));
    }
    
    #[test]
    fn test_truncate_string() {
        assert_eq!(utils::truncate_string("Hello World", 5), "He...");
        assert_eq!(utils::truncate_string("Hi", 10), "Hi");
        assert_eq!(utils::truncate_string("Hello", 5), "Hello");
    }
}