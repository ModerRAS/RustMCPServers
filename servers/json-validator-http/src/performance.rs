//! 性能优化模块
//! 
//! 这个模块包含了各种性能优化策略和实现，
//! 旨在提高服务器的吞吐量和响应速度。

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{debug, info, warn};
use serde_json::Value;
use anyhow::Result;

/// 性能优化配置
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// 并发限制
    pub max_concurrent_requests: usize,
    /// 连接池大小
    pub connection_pool_size: usize,
    /// 缓存大小
    pub cache_size: usize,
    /// 内存限制 (MB)
    pub memory_limit_mb: usize,
    /// 请求超时时间
    pub request_timeout: Duration,
    /// 启用内存池
    pub enable_memory_pool: bool,
    /// 启用连接复用
    pub enable_connection_reuse: bool,
    /// 启用压缩
    pub enable_compression: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 1000,
            connection_pool_size: 10,
            cache_size: 1000,
            memory_limit_mb: 512,
            request_timeout: Duration::from_secs(30),
            enable_memory_pool: true,
            enable_connection_reuse: true,
            enable_compression: true,
        }
    }
}

/// 性能统计信息
#[derive(Debug, Default)]
pub struct PerformanceStats {
    /// 当前并发请求数
    pub current_concurrent_requests: usize,
    /// 峰值并发请求数
    pub peak_concurrent_requests: usize,
    /// 总请求数
    pub total_requests: u64,
    /// 成功请求数
    pub successful_requests: u64,
    /// 失败请求数
    pub failed_requests: u64,
    /// 平均响应时间 (ms)
    pub avg_response_time_ms: f64,
    /// P95响应时间 (ms)
    pub p95_response_time_ms: f64,
    /// P99响应时间 (ms)
    pub p99_response_time_ms: f64,
    /// 内存使用量 (MB)
    pub memory_usage_mb: f64,
    /// CPU使用率 (%)
    pub cpu_usage_percent: f64,
    /// 缓存命中率 (%)
    pub cache_hit_rate_percent: f64,
}

/// 内存池管理器
pub struct MemoryPool {
    /// 对象池
    pool: Arc<RwLock<Vec<String>>>,
    /// 池大小
    pool_size: usize,
    /// 当前使用量
    current_usage: Arc<RwLock<usize>>,
}

impl MemoryPool {
    /// 创建新的内存池
    pub fn new(pool_size: usize) -> Self {
        let pool = Arc::new(RwLock::new(Vec::with_capacity(pool_size)));
        let current_usage = Arc::new(RwLock::new(0));
        
        Self {
            pool,
            pool_size,
            current_usage,
        }
    }
    
    /// 从池中获取字符串
    pub async fn get_string(&self, capacity: usize) -> String {
        let mut pool = self.pool.write().await;
        let mut usage = self.current_usage.write().await;
        
        if let Some(mut s) = pool.pop() {
            if s.capacity() >= capacity {
                s.clear();
                return s;
            } else {
                // 如果容量不够，放回池中
                pool.push(s);
            }
        }
        
        *usage += 1;
        String::with_capacity(capacity)
    }
    
    /// 归还字符串到池中
    pub async fn return_string(&self, s: String) {
        let mut pool = self.pool.write().await;
        let mut usage = self.current_usage.write().await;
        
        if pool.len() < self.pool_size {
            pool.push(s);
            *usage = usage.saturating_sub(1);
        }
    }
    
    /// 获取当前使用量
    pub async fn get_usage(&self) -> usize {
        *self.current_usage.read().await
    }
    
    /// 清理池
    pub async fn clear(&self) {
        let mut pool = self.pool.write().await;
        let mut usage = self.current_usage.write().await;
        
        pool.clear();
        *usage = 0;
    }
}

/// 连接池管理器
pub struct ConnectionPool<T> {
    /// 连接池
    pool: Arc<RwLock<Vec<T>>>,
    /// 池大小
    pool_size: usize,
    /// 连接工厂
    connection_factory: Box<dyn Fn() -> Result<T> + Send + Sync>,
}

impl<T> ConnectionPool<T> {
    /// 创建新的连接池
    pub fn new<F>(pool_size: usize, factory: F) -> Self 
    where
        F: Fn() -> Result<T> + Send + Sync + 'static,
    {
        let pool = Arc::new(RwLock::new(Vec::with_capacity(pool_size)));
        let connection_factory = Box::new(factory);
        
        Self {
            pool,
            pool_size,
            connection_factory,
        }
    }
    
    /// 从池中获取连接
    pub async fn get_connection(&self) -> Result<T> {
        let mut pool = self.pool.write().await;
        
        if let Some(conn) = pool.pop() {
            return Ok(conn);
        }
        
        // 如果池为空，创建新连接
        (self.connection_factory)()
    }
    
    /// 归还连接到池中
    pub async fn return_connection(&self, conn: T) {
        let mut pool = self.pool.write().await;
        
        if pool.len() < self.pool_size {
            pool.push(conn);
        }
    }
    
    /// 获取池大小
    pub async fn size(&self) -> usize {
        self.pool.read().await.len()
    }
    
    /// 清理池
    pub async fn clear(&self) {
        let mut pool = self.pool.write().await;
        pool.clear();
    }
}

/// 请求限流器
pub struct RequestLimiter {
    /// 信号量
    semaphore: Arc<Semaphore>,
    /// 当前并发数
    current_concurrent: Arc<RwLock<usize>>,
    /// 峰值并发数
    peak_concurrent: Arc<RwLock<usize>>,
}

impl RequestLimiter {
    /// 创建新的请求限流器
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            current_concurrent: Arc::new(RwLock::new(0)),
            peak_concurrent: Arc::new(RwLock::new(0)),
        }
    }
    
    /// 获取许可
    pub async fn acquire(&self) -> RequestPermit {
        let permit = self.semaphore.acquire().await.unwrap();
        
        // 更新统计信息
        let mut current = self.current_concurrent.write().await;
        *current += 1;
        
        let mut peak = self.peak_concurrent.write().await;
        if *current > *peak {
            *peak = *current;
        }
        
        drop(current);
        drop(peak);
        
        RequestPermit {
            permit,
            current_concurrent: self.current_concurrent.clone(),
        }
    }
    
    /// 获取当前并发数
    pub async fn get_current_concurrent(&self) -> usize {
        *self.current_concurrent.read().await
    }
    
    /// 获取峰值并发数
    pub async fn get_peak_concurrent(&self) -> usize {
        *self.peak_concurrent.read().await
    }
}

/// 请求许可
pub struct RequestPermit {
    permit: tokio::sync::OwnedSemaphorePermit,
    current_concurrent: Arc<RwLock<usize>>,
}

impl RequestPermit {
    /// 获取当前并发数
    pub async fn get_current_concurrent(&self) -> usize {
        *self.current_concurrent.read().await
    }
}

impl Drop for RequestPermit {
    fn drop(&mut self) {
        let current_concurrent = self.current_concurrent.clone();
        tokio::spawn(async move {
            let mut current = current_concurrent.write().await;
            *current = current.saturating_sub(1);
        });
    }
}

/// 响应时间跟踪器
pub struct ResponseTimeTracker {
    /// 响应时间历史
    response_times: Arc<RwLock<Vec<Duration>>>,
    /// 最大历史记录数
    max_history: usize,
}

impl ResponseTimeTracker {
    /// 创建新的响应时间跟踪器
    pub fn new(max_history: usize) -> Self {
        Self {
            response_times: Arc::new(RwLock::new(Vec::with_capacity(max_history))),
            max_history,
        }
    }
    
    /// 记录响应时间
    pub async fn record_response_time(&self, duration: Duration) {
        let mut times = self.response_times.write().await;
        
        times.push(duration);
        
        // 如果超过最大历史记录数，删除最旧的记录
        if times.len() > self.max_history {
            times.remove(0);
        }
    }
    
    /// 获取平均响应时间
    pub async fn get_avg_response_time(&self) -> Duration {
        let times = self.response_times.read().await;
        
        if times.is_empty() {
            return Duration::from_millis(0);
        }
        
        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }
    
    /// 获取P95响应时间
    pub async fn get_p95_response_time(&self) -> Duration {
        self.get_percentile_response_time(0.95).await
    }
    
    /// 获取P99响应时间
    pub async fn get_p99_response_time(&self) -> Duration {
        self.get_percentile_response_time(0.99).await
    }
    
    /// 获取指定百分位的响应时间
    async fn get_percentile_response_time(&self, percentile: f64) -> Duration {
        let times = self.response_times.read().await;
        
        if times.is_empty() {
            return Duration::from_millis(0);
        }
        
        let mut times: Vec<Duration> = times.iter().copied().collect();
        times.sort();
        
        let index = (times.len() as f64 * percentile).floor() as usize;
        times.get(index).copied().unwrap_or(Duration::from_millis(0))
    }
    
    /// 清理历史记录
    pub async fn clear(&self) {
        let mut times = self.response_times.write().await;
        times.clear();
    }
}

/// 性能优化管理器
pub struct PerformanceManager {
    /// 配置
    config: PerformanceConfig,
    /// 内存池
    memory_pool: Arc<MemoryPool>,
    /// 请求限流器
    request_limiter: Arc<RequestLimiter>,
    /// 响应时间跟踪器
    response_time_tracker: Arc<ResponseTimeTracker>,
    /// 统计信息
    stats: Arc<RwLock<PerformanceStats>>,
}

impl PerformanceManager {
    /// 创建新的性能管理器
    pub fn new(config: PerformanceConfig) -> Self {
        let memory_pool = Arc::new(MemoryPool::new(config.cache_size));
        let request_limiter = Arc::new(RequestLimiter::new(config.max_concurrent_requests));
        let response_time_tracker = Arc::new(ResponseTimeTracker::new(1000));
        let stats = Arc::new(RwLock::new(PerformanceStats::default()));
        
        Self {
            config,
            memory_pool,
            request_limiter,
            response_time_tracker,
            stats,
        }
    }
    
    /// 获取请求许可
    pub async fn acquire_request_permit(&self) -> Result<RequestPermit> {
        Ok(self.request_limiter.acquire().await)
    }
    
    /// 获取内存池字符串
    pub async fn get_pooled_string(&self, capacity: usize) -> String {
        if self.config.enable_memory_pool {
            self.memory_pool.get_string(capacity).await
        } else {
            String::with_capacity(capacity)
        }
    }
    
    /// 归还内存池字符串
    pub async fn return_pooled_string(&self, s: String) {
        if self.config.enable_memory_pool {
            self.memory_pool.return_string(s).await;
        }
    }
    
    /// 记录响应时间
    pub async fn record_response_time(&self, duration: Duration) {
        self.response_time_tracker.record_response_time(duration).await;
        
        // 更新统计信息
        let mut stats = self.stats.write().await;
        stats.total_requests += 1;
        
        // 更新响应时间统计
        let avg_time = self.response_time_tracker.get_avg_response_time().await;
        stats.avg_response_time_ms = avg_time.as_millis() as f64;
        
        let p95_time = self.response_time_tracker.get_p95_response_time().await;
        stats.p95_response_time_ms = p95_time.as_millis() as f64;
        
        let p99_time = self.response_time_tracker.get_p99_response_time().await;
        stats.p99_response_time_ms = p99_time.as_millis() as f64;
        
        // 更新并发统计
        stats.current_concurrent_requests = self.request_limiter.get_current_concurrent().await;
        stats.peak_concurrent_requests = self.request_limiter.get_peak_concurrent().await;
    }
    
    /// 记录成功请求
    pub async fn record_success(&self) {
        let mut stats = self.stats.write().await;
        stats.successful_requests += 1;
    }
    
    /// 记录失败请求
    pub async fn record_failure(&self) {
        let mut stats = self.stats.write().await;
        stats.failed_requests += 1;
    }
    
    /// 更新系统资源使用情况
    pub async fn update_system_stats(&self) {
        let mut stats = self.stats.write().await;
        
        // 更新内存使用情况
        stats.memory_usage_mb = self.get_memory_usage_mb();
        
        // 更新CPU使用情况
        stats.cpu_usage_percent = self.get_cpu_usage_percent();
    }
    
    /// 获取性能统计信息
    pub async fn get_stats(&self) -> PerformanceStats {
        self.stats.read().await.clone()
    }
    
    /// 获取内存使用量 (MB)
    fn get_memory_usage_mb(&self) -> f64 {
        #[cfg(target_os = "linux")]
        {
            use std::fs;
            if let Ok(content) = fs::read_to_string("/proc/self/status") {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(kb_str) = parts.get(1) {
                            if let Ok(kb) = kb_str.parse::<f64>() {
                                return kb / 1024.0;
                            }
                        }
                    }
                }
            }
        }
        
        // 通用实现：使用系统信息
        if let Ok(memory_info) = sys_info::mem_info() {
            return (memory_info.total - memory_info.free) as f64 / 1024.0;
        }
        
        0.0
    }
    
    /// 获取CPU使用率 (%)
    fn get_cpu_usage_percent(&self) -> f64 {
        // 简化实现：返回固定值
        // 在实际应用中，可以使用更精确的CPU监控
        0.0
    }
    
    /// 清理资源
    pub async fn cleanup(&self) {
        self.memory_pool.clear().await;
        self.response_time_tracker.clear().await;
    }
}

/// 性能优化工具函数
pub mod utils {
    use super::*;
    
    /// 快速哈希计算
    pub fn fast_hash(value: &Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        value.to_string().hash(&mut hasher);
        hasher.finish()
    }
    
    /// 优化的JSON序列化
    pub fn optimized_json_serialize(value: &Value) -> Result<String> {
        serde_json::to_string(value)
            .map_err(|e| anyhow::anyhow!("Serialization error: {}", e))
    }
    
    /// 优化的JSON反序列化
    pub fn optimized_json_deserialize(data: &str) -> Result<Value> {
        serde_json::from_str(data)
            .map_err(|e| anyhow::anyhow!("Deserialization error: {}", e))
    }
    
    /// 内存高效的数据结构转换
    pub fn memory_efficient_conversion<T, U>(data: T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        serde_json::from_str(&serde_json::to_string(&data)?)
            .map_err(|e| anyhow::anyhow!("Conversion error: {}", e))
    }
    
    /// 批量处理优化
    pub async fn optimized_batch_processing<F, T, R>(
        items: Vec<T>,
        processor: F,
        batch_size: usize,
    ) -> Vec<R>
    where
        F: Fn(Vec<T>) -> Vec<R> + Send + Sync,
        T: Send + Sync,
        R: Send + Sync,
    {
        let mut results = Vec::with_capacity(items.len());
        
        for chunk in items.chunks(batch_size) {
            let batch = chunk.to_vec();
            let batch_results = tokio::task::spawn_blocking(move || processor(batch))
                .await
                .unwrap_or_else(|_| Vec::new());
            
            results.extend(batch_results);
        }
        
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_memory_pool() {
        let pool = MemoryPool::new(10);
        
        // 获取字符串
        let s1 = pool.get_string(100).await;
        assert_eq!(s1.capacity(), 100);
        
        // 归还字符串
        pool.return_string(s1).await;
        
        // 检查使用量
        assert_eq!(pool.get_usage().await, 0);
    }

    #[tokio::test]
    async fn test_request_limiter() {
        let limiter = RequestLimiter::new(2);
        
        // 获取许可
        let permit1 = limiter.acquire().await;
        let permit2 = limiter.acquire().await;
        
        // 检查并发数
        assert_eq!(limiter.get_current_concurrent().await, 2);
        
        // 释放许可
        drop(permit1);
        sleep(Duration::from_millis(10)).await;
        
        assert_eq!(limiter.get_current_concurrent().await, 1);
    }

    #[tokio::test]
    async fn test_response_time_tracker() {
        let tracker = ResponseTimeTracker::new(10);
        
        // 记录响应时间
        tracker.record_response_time(Duration::from_millis(100)).await;
        tracker.record_response_time(Duration::from_millis(200)).await;
        
        // 检查平均响应时间
        let avg = tracker.get_avg_response_time().await;
        assert_eq!(avg, Duration::from_millis(150));
    }

    #[tokio::test]
    async fn test_performance_manager() {
        let config = PerformanceConfig::default();
        let manager = PerformanceManager::new(config);
        
        // 获取请求许可
        let _permit = manager.acquire_request_permit().await.unwrap();
        
        // 记录响应时间
        manager.record_response_time(Duration::from_millis(50)).await;
        
        // 记录成功
        manager.record_success().await;
        
        // 获取统计信息
        let stats = manager.get_stats().await;
        assert_eq!(stats.successful_requests, 1);
        assert_eq!(stats.total_requests, 1);
    }

    #[test]
    fn test_fast_hash() {
        let value = serde_json::json!({"name": "test", "age": 25});
        let hash1 = utils::fast_hash(&value);
        let hash2 = utils::fast_hash(&value);
        
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_optimized_json_serialization() {
        let value = serde_json::json!({"name": "test", "age": 25});
        let serialized = utils::optimized_json_serialize(&value).unwrap();
        let deserialized = utils::optimized_json_deserialize(&serialized).unwrap();
        
        assert_eq!(value, deserialized);
    }

    #[tokio::test]
    async fn test_optimized_batch_processing() {
        let items = vec![1, 2, 3, 4, 5];
        let results = utils::optimized_batch_processing(
            items,
            |batch| batch.into_iter().map(|x| x * 2).collect(),
            2,
        ).await;
        
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }
}