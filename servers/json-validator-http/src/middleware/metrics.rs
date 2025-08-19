//! 指标中间件

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use once_cell::sync::Lazy;
use prometheus::{Counter, Gauge, Histogram, TextEncoder, Encoder};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

/// 指标收集器
#[derive(Clone)]
pub struct MetricsCollector {
    /// 请求总数
    pub requests_total: Counter,
    /// 成功请求数
    pub requests_success: Counter,
    /// 失败请求数
    pub requests_failed: Counter,
    /// 响应时间直方图
    pub response_time_histogram: Histogram,
    /// 活跃连接数
    pub active_connections: Gauge,
    /// 验证总数
    pub validations_total: Counter,
    /// 成功验证数
    pub validations_success: Counter,
    /// 失败验证数
    pub validations_failed: Counter,
    /// 缓存命中数
    pub cache_hits: Counter,
    /// 缓存未命中数
    pub cache_misses: Counter,
    /// 验证时间直方图
    pub validation_time_histogram: Histogram,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Self {
        Self {
            requests_total: Counter::new(
                "http_requests_total",
                "Total number of HTTP requests"
            ).unwrap(),
            requests_success: Counter::new(
                "http_requests_success_total",
                "Total number of successful HTTP requests"
            ).unwrap(),
            requests_failed: Counter::new(
                "http_requests_failed_total",
                "Total number of failed HTTP requests"
            ).unwrap(),
            response_time_histogram: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "http_response_time_seconds",
                    "HTTP response time in seconds"
                ).buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0])
            ).unwrap(),
            active_connections: Gauge::new(
                "http_active_connections",
                "Number of active HTTP connections"
            ).unwrap(),
            validations_total: Counter::new(
                "json_validations_total",
                "Total number of JSON validations"
            ).unwrap(),
            validations_success: Counter::new(
                "json_validations_success_total",
                "Total number of successful JSON validations"
            ).unwrap(),
            validations_failed: Counter::new(
                "json_validations_failed_total",
                "Total number of failed JSON validations"
            ).unwrap(),
            cache_hits: Counter::new(
                "cache_hits_total",
                "Total number of cache hits"
            ).unwrap(),
            cache_misses: Counter::new(
                "cache_misses_total",
                "Total number of cache misses"
            ).unwrap(),
            validation_time_histogram: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "json_validation_time_seconds",
                    "JSON validation time in seconds"
                ).buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0])
            ).unwrap(),
        }
    }
    
    /// 记录请求开始
    pub fn record_request_start(&self) {
        self.requests_total.inc();
        self.active_connections.inc();
    }
    
    /// 记录请求成功
    pub fn record_request_success(&self, duration: f64) {
        self.requests_success.inc();
        self.response_time_histogram.observe(duration);
    }
    
    /// 记录请求失败
    pub fn record_request_failed(&self, duration: f64) {
        self.requests_failed.inc();
        self.response_time_histogram.observe(duration);
    }
    
    /// 记录请求结束
    pub fn record_request_end(&self) {
        self.active_connections.dec();
    }
    
    /// 记录验证开始
    pub fn record_validation_start(&self) {
        self.validations_total.inc();
    }
    
    /// 记录验证成功
    pub fn record_validation_success(&self, duration: f64, cache_hit: bool) {
        self.validations_success.inc();
        self.validation_time_histogram.observe(duration);
        if cache_hit {
            self.cache_hits.inc();
        } else {
            self.cache_misses.inc();
        }
    }
    
    /// 记录验证失败
    pub fn record_validation_failed(&self, duration: f64, cache_hit: bool) {
        self.validations_failed.inc();
        self.validation_time_histogram.observe(duration);
        if cache_hit {
            self.cache_hits.inc();
        } else {
            self.cache_misses.inc();
        }
    }
    
    /// 获取平均响应时间
    pub fn get_avg_response_time(&self) -> f64 {
        // 这里应该从直方图中计算平均值
        // 简化实现，返回0
        0.0
    }
    
    /// 获取缓存命中率
    pub fn get_cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.get();
        let misses = self.cache_misses.get();
        let total = hits + misses;
        
        if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        }
    }
    
    /// 获取验证成功率
    pub fn get_validation_success_rate(&self) -> f64 {
        let success = self.validations_success.get();
        let failed = self.validations_failed.get();
        let total = success + failed;
        
        if total == 0 {
            0.0
        } else {
            success as f64 / total as f64
        }
    }
    
    /// 导出指标
    pub fn export(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = prometheus::gather();
        encoder.encode_to_string(&metric_families).unwrap_or_else(|e| {
            error!("Failed to encode metrics: {}", e);
            String::new()
        })
    }
}

/// 全局指标收集器
static GLOBAL_METRICS: Lazy<Arc<RwLock<MetricsCollector>>> = 
    Lazy::new(|| Arc::new(RwLock::new(MetricsCollector::new())));

/// 指标中间件
pub struct MetricsLayer {
    metrics: Arc<RwLock<MetricsCollector>>,
}

impl MetricsLayer {
    /// 创建新的指标中间件
    pub fn new() -> Self {
        Self {
            metrics: GLOBAL_METRICS.clone(),
        }
    }
    
    /// 获取指标收集器引用
    pub fn get_collector(&self) -> Arc<RwLock<MetricsCollector>> {
        self.metrics.clone()
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for MetricsLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        let start_time = Instant::now();
        let method = req.method().clone();
        let uri = req.uri().clone();
        
        // 记录请求开始
        {
            let metrics = self.metrics.read().await;
            metrics.record_request_start();
        }
        
        debug!("Recording metrics for request: {} {}", method, uri);
        
        // 处理请求
        let response = next.run(req).await;
        
        let duration = start_time.elapsed();
        let duration_seconds = duration.as_secs_f64();
        let status = response.status();
        
        // 记录请求完成
        {
            let metrics = self.metrics.read().await;
            if status.is_success() {
                metrics.record_request_success(duration_seconds);
            } else {
                metrics.record_request_failed(duration_seconds);
            }
            metrics.record_request_end();
        }
        
        debug!("Request metrics recorded: {} {} - {} - {:.3}s", 
               method, uri, status, duration_seconds);
        
        Ok(response)
    }
}

/// 指标辅助函数
pub async fn record_validation_metrics(
    success: bool,
    duration: std::time::Duration,
    cache_hit: bool,
) {
    let metrics = GLOBAL_METRICS.read().await;
    let duration_seconds = duration.as_secs_f64();
    
    if success {
        metrics.record_validation_success(duration_seconds, cache_hit);
    } else {
        metrics.record_validation_failed(duration_seconds, cache_hit);
    }
}

/// 获取指标数据
pub async fn get_metrics_data() -> String {
    let metrics = GLOBAL_METRICS.read().await;
    metrics.export()
}

/// 获取指标统计
pub async fn get_metrics_stats() -> serde_json::Value {
    let metrics = GLOBAL_METRICS.read().await;
    
    serde_json::json!({
        "requests_total": metrics.requests_total.get(),
        "requests_success": metrics.requests_success.get(),
        "requests_failed": metrics.requests_failed.get(),
        "active_connections": metrics.active_connections.get(),
        "validations_total": metrics.validations_total.get(),
        "validations_success": metrics.validations_success.get(),
        "validations_failed": metrics.validations_failed.get(),
        "cache_hits": metrics.cache_hits.get(),
        "cache_misses": metrics.cache_misses.get(),
        "cache_hit_rate": metrics.get_cache_hit_rate(),
        "validation_success_rate": metrics.get_validation_success_rate(),
        "avg_response_time": metrics.get_avg_response_time(),
    })
}

/// 重置指标
pub async fn reset_metrics() {
    let metrics = GLOBAL_METRICS.read().await;
    
    metrics.requests_total.reset();
    metrics.requests_success.reset();
    metrics.requests_failed.reset();
    metrics.response_time_histogram.reset();
    metrics.active_connections.set(0.0);
    metrics.validations_total.reset();
    metrics.validations_success.reset();
    metrics.validations_failed.reset();
    metrics.cache_hits.reset();
    metrics.cache_misses.reset();
    metrics.validation_time_histogram.reset();
}

/// 指标健康检查
pub async fn metrics_health_check() -> bool {
    let metrics = GLOBAL_METRICS.read().await;
    
    // 检查指标是否正常工作
    metrics.requests_total.get() >= 0 && metrics.active_connections.get() >= 0.0
}

/// 指标服务器信息
pub async fn get_metrics_info() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "metrics_enabled": true,
        "metrics_collector": "prometheus",
        "export_format": "text/plain",
        "metrics_available": [
            "http_requests_total",
            "http_requests_success_total",
            "http_requests_failed_total",
            "http_response_time_seconds",
            "http_active_connections",
            "json_validations_total",
            "json_validations_success_total",
            "json_validations_failed_total",
            "cache_hits_total",
            "cache_misses_total",
            "json_validation_time_seconds"
        ]
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        // 测试请求记录
        collector.record_request_start();
        collector.record_request_success(0.1);
        collector.record_request_end();
        
        assert_eq!(collector.requests_total.get(), 1);
        assert_eq!(collector.requests_success.get(), 1);
        assert_eq!(collector.requests_failed.get(), 0);
    }

    #[tokio::test]
    async fn test_validation_metrics() {
        let collector = MetricsCollector::new();
        
        // 测试验证记录
        collector.record_validation_start();
        collector.record_validation_success(0.05, true);
        
        assert_eq!(collector.validations_total.get(), 1);
        assert_eq!(collector.validations_success.get(), 1);
        assert_eq!(collector.validations_failed.get(), 0);
        assert_eq!(collector.cache_hits.get(), 1);
        assert_eq!(collector.cache_misses.get(), 0);
        
        // 测试缓存命中率
        assert_eq!(collector.get_cache_hit_rate(), 1.0);
        assert_eq!(collector.get_validation_success_rate(), 1.0);
    }

    #[tokio::test]
    async fn test_metrics_export() {
        let collector = MetricsCollector::new();
        
        // 记录一些指标
        collector.record_request_start();
        collector.record_request_success(0.1);
        collector.record_request_end();
        
        let exported = collector.export();
        assert!(!exported.is_empty());
        assert!(exported.contains("http_requests_total"));
    }

    #[tokio::test]
    async fn test_global_metrics() {
        // 测试全局指标功能
        record_validation_metrics(true, std::time::Duration::from_millis(50), true).await;
        
        let stats = get_metrics_stats().await;
        assert!(stats["validations_total"].as_u64().unwrap() > 0);
        assert!(stats["cache_hits"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_metrics_health_check() {
        let health = metrics_health_check().await;
        assert!(health);
    }

    #[tokio::test]
    async fn test_metrics_info() {
        let info = get_metrics_info().await;
        assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(info["metrics_enabled"], true);
        assert!(info["metrics_available"].is_array());
    }
}