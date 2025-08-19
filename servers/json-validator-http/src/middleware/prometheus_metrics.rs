//! Prometheus监控指标中间件

use axum::{
    extract::Request,
    http::HeaderValue,
    middleware::Next,
    response::Response,
};
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, Opts, Registry, TextEncoder, Encoder,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 监控指标收集器
#[derive(Clone)]
pub struct MetricsCollector {
    /// 注册表
    registry: Arc<Registry>,
    /// HTTP请求总数
    http_requests_total: Counter,
    /// HTTP响应时间直方图
    http_response_time_seconds: Histogram,
    /// 活跃连接数
    active_connections: Gauge,
    /// JSON验证总数
    json_validations_total: Counter,
    /// JSON验证时间直方图
    json_validation_time_seconds: Histogram,
    /// 缓存命中率
    cache_hit_ratio: Gauge,
    /// 错误总数
    errors_total: Counter,
    /// 速率限制拒绝数
    rate_limit_rejected_total: Counter,
    /// 认证失败数
    auth_failures_total: Counter,
    /// 内存使用量
    memory_usage_bytes: Gauge,
    /// CPU使用率
    cpu_usage_percent: Gauge,
    /// 请求队列大小
    request_queue_size: Gauge,
    /// 系统启动时间
    system_start_time: Gauge,
    /// 最后配置重载时间
    last_config_reload: Gauge,
}

impl MetricsCollector {
    /// 创建新的监控指标收集器
    pub fn new() -> Result<Self, prometheus::Error> {
        let registry = Arc::new(Registry::new());

        // HTTP请求总数
        let http_requests_total = Counter::with_opts(Opts::new(
            "http_requests_total",
            "Total number of HTTP requests",
        ))?;
        registry.register(Box::new(http_requests_total.clone()))?;

        // HTTP响应时间直方图
        let http_response_time_seconds = Histogram::with_opts(HistogramOpts::new(
            "http_response_time_seconds",
            "HTTP response time in seconds",
        ).buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]))?;
        registry.register(Box::new(http_response_time_seconds.clone()))?;

        // 活跃连接数
        let active_connections = Gauge::with_opts(Opts::new(
            "active_connections",
            "Number of active connections",
        ))?;
        registry.register(Box::new(active_connections.clone()))?;

        // JSON验证总数
        let json_validations_total = Counter::with_opts(Opts::new(
            "json_validations_total",
            "Total number of JSON validations",
        ))?;
        registry.register(Box::new(json_validations_total.clone()))?;

        // JSON验证时间直方图
        let json_validation_time_seconds = Histogram::with_opts(HistogramOpts::new(
            "json_validation_time_seconds",
            "JSON validation time in seconds",
        ).buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0]))?;
        registry.register(Box::new(json_validation_time_seconds.clone()))?;

        // 缓存命中率
        let cache_hit_ratio = Gauge::with_opts(Opts::new(
            "cache_hit_ratio",
            "Cache hit ratio (0.0 to 1.0)",
        ))?;
        registry.register(Box::new(cache_hit_ratio.clone()))?;

        // 错误总数
        let errors_total = Counter::with_opts(Opts::new(
            "errors_total",
            "Total number of errors",
        ))?;
        registry.register(Box::new(errors_total.clone()))?;

        // 速率限制拒绝数
        let rate_limit_rejected_total = Counter::with_opts(Opts::new(
            "rate_limit_rejected_total",
            "Total number of rate limit rejections",
        ))?;
        registry.register(Box::new(rate_limit_rejected_total.clone()))?;

        // 认证失败数
        let auth_failures_total = Counter::with_opts(Opts::new(
            "auth_failures_total",
            "Total number of authentication failures",
        ))?;
        registry.register(Box::new(auth_failures_total.clone()))?;

        // 内存使用量
        let memory_usage_bytes = Gauge::with_opts(Opts::new(
            "memory_usage_bytes",
            "Memory usage in bytes",
        ))?;
        registry.register(Box::new(memory_usage_bytes.clone()))?;

        // CPU使用率
        let cpu_usage_percent = Gauge::with_opts(Opts::new(
            "cpu_usage_percent",
            "CPU usage percentage",
        ))?;
        registry.register(Box::new(cpu_usage_percent.clone()))?;

        // 请求队列大小
        let request_queue_size = Gauge::with_opts(Opts::new(
            "request_queue_size",
            "Current request queue size",
        ))?;
        registry.register(Box::new(request_queue_size.clone()))?;

        // 系统启动时间
        let system_start_time = Gauge::with_opts(Opts::new(
            "system_start_time_seconds",
            "System start time in seconds since epoch",
        ))?;
        registry.register(Box::new(system_start_time.clone()))?;

        // 最后配置重载时间
        let last_config_reload = Gauge::with_opts(Opts::new(
            "last_config_reload_seconds",
            "Last config reload time in seconds since epoch",
        ))?;
        registry.register(Box::new(last_config_reload.clone()))?;

        Ok(Self {
            registry,
            http_requests_total,
            http_response_time_seconds,
            active_connections,
            json_validations_total,
            json_validation_time_seconds,
            cache_hit_ratio,
            errors_total,
            rate_limit_rejected_total,
            auth_failures_total,
            memory_usage_bytes,
            cpu_usage_percent,
            request_queue_size,
            system_start_time,
            last_config_reload,
        })
    }

    /// 记录HTTP请求
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        let labels = &[
            ("method", method),
            ("path", path),
            ("status", &status.to_string()),
        ];
        
        self.http_requests_total.with_label_values(labels).inc();
        self.http_response_time_seconds
            .with_label_values(labels)
            .observe(duration.as_secs_f64());
    }

    /// 记录JSON验证
    pub fn record_json_validation(&self, validation_type: &str, success: bool, duration: Duration) {
        let labels = &[
            ("type", validation_type),
            ("success", &success.to_string()),
        ];
        
        self.json_validations_total.with_label_values(labels).inc();
        self.json_validation_time_seconds
            .with_label_values(labels)
            .observe(duration.as_secs_f64());
    }

    /// 更新活跃连接数
    pub fn update_active_connections(&self, count: i64) {
        self.active_connections.set(count);
    }

    /// 更新缓存命中率
    pub fn update_cache_hit_ratio(&self, ratio: f64) {
        self.cache_hit_ratio.set(ratio);
    }

    /// 记录错误
    pub fn record_error(&self, error_type: &str, error_message: &str) {
        let labels = &[
            ("type", error_type),
            ("message", error_message),
        ];
        
        self.errors_total.with_label_values(labels).inc();
    }

    /// 记录速率限制拒绝
    pub fn record_rate_limit_rejection(&self, client_ip: &str, limit_type: &str) {
        let labels = &[
            ("client_ip", client_ip),
            ("limit_type", limit_type),
        ];
        
        self.rate_limit_rejected_total.with_label_values(labels).inc();
    }

    /// 记录认证失败
    pub fn record_auth_failure(&self, auth_type: &str, reason: &str) {
        let labels = &[
            ("auth_type", auth_type),
            ("reason", reason),
        ];
        
        self.auth_failures_total.with_label_values(labels).inc();
    }

    /// 更新系统指标
    pub fn update_system_metrics(&self) {
        // 更新内存使用量
        if let Ok(memory_usage) = get_memory_usage() {
            self.memory_usage_bytes.set(memory_usage as f64);
        }

        // 更新CPU使用率
        if let Ok(cpu_usage) = get_cpu_usage() {
            self.cpu_usage_percent.set(cpu_usage);
        }
    }

    /// 更新请求队列大小
    pub fn update_request_queue_size(&self, size: i64) {
        self.request_queue_size.set(size);
    }

    /// 设置系统启动时间
    pub fn set_system_start_time(&self, timestamp: i64) {
        self.system_start_time.set(timestamp as f64);
    }

    /// 设置最后配置重载时间
    pub fn set_last_config_reload(&self, timestamp: i64) {
        self.last_config_reload.set(timestamp as f64);
    }

    /// 获取指标数据
    pub fn gather_metrics(&self) -> Result<String, prometheus::Error> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode_to_string(&metric_families)
    }

    /// 获取注册表引用
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// Prometheus指标中间件
pub struct PrometheusMetricsLayer {
    /// 指标收集器
    collector: Arc<MetricsCollector>,
}

impl PrometheusMetricsLayer {
    /// 创建新的Prometheus指标中间件
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self { collector }
    }
}

#[axum::async_trait]
impl<S> axum::middleware::Next<S> for PrometheusMetricsLayer
where
    S: Send + Sync,
{
    async fn run(self, req: Request, next: Next<S>) -> Result<Response, axum::Error> {
        let start_time = Instant::now();
        let method = req.method().to_string();
        let path = req.uri().path().to_string();

        // 更新活跃连接数
        self.collector.update_active_connections(1);

        // 处理请求
        let response = next.run(req).await;

        // 计算请求持续时间
        let duration = start_time.elapsed();
        let status = response.status().as_u16();

        // 记录指标
        self.collector.record_http_request(&method, &path, status, duration);

        // 更新活跃连接数
        self.collector.update_active_connections(-1);

        // 更新系统指标
        self.collector.update_system_metrics();

        debug!("Recorded metrics for {} {} - {} ({:.3}ms)", method, path, status, duration.as_millis());

        Ok(response)
    }
}

/// 指标端点处理器
pub async fn metrics_handler(collector: Arc<MetricsCollector>) -> Result<axum::response::Json<serde_json::Value>, axum::http::StatusCode> {
    match collector.gather_metrics() {
        Ok(metrics) => {
            // 解析Prometheus格式的指标为JSON
            let json_metrics = parse_prometheus_metrics(&metrics);
            Ok(axum::response::Json(json_metrics))
        }
        Err(e) => {
            error!("Failed to gather metrics: {}", e);
            Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// 解析Prometheus格式的指标为JSON
fn parse_prometheus_metrics(metrics: &str) -> serde_json::Value {
    let mut json_metrics = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "metrics": {}
    });

    for line in metrics.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }

        if let Some((metric_name, value)) = line.split_once(' ') {
            let metric_name = metric_name.trim();
            let value = value.trim();

            if let Some((metric_name, labels)) = metric_name.split_once('{') {
                // 带标签的指标
                let metric_name = metric_name.to_string();
                let labels_str = labels.trim_end_matches('}');
                let labels = parse_labels(labels_str);

                if !json_metrics["metrics"].get(&metric_name).is_some() {
                    json_metrics["metrics"][metric_name] = serde_json::json!({});
                }

                if let Ok(num_value) = value.parse::<f64>() {
                    json_metrics["metrics"][metric_name][serde_json::to_string(&labels).as_str()] = num_value;
                }
            } else {
                // 简单指标
                if let Ok(num_value) = value.parse::<f64>() {
                    json_metrics["metrics"][metric_name] = num_value;
                }
            }
        }
    }

    json_metrics
}

/// 解析标签字符串
fn parse_labels(labels_str: &str) -> serde_json::Value {
    let mut labels = serde_json::Map::new();
    
    for label_pair in labels_str.split(',') {
        if let Some((key, value)) = label_pair.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            labels.insert(key, serde_json::Value::String(value));
        }
    }
    
    serde_json::Value::Object(labels)
}

/// 获取内存使用量
fn get_memory_usage() -> Result<u64, String> {
    // 简化版本，实际应该使用系统特定的API
    #[cfg(target_os = "linux")]
    {
        use std::fs::File;
        use std::io::Read;
        
        if let Ok(mut file) = File::open("/proc/self/status") {
            let mut content = String::new();
            if file.read_to_string(&mut content).is_ok() {
                for line in content.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(value) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = value.parse::<u64>() {
                                return Ok(kb * 1024); // 转换为字节
                            }
                        }
                    }
                }
            }
        }
    }

    // 默认返回0
    Ok(0)
}

/// 获取CPU使用率
fn get_cpu_usage() -> f64 {
    // 简化版本，实际应该使用系统特定的API
    0.0
}

/// 健康检查指标
pub struct HealthMetrics {
    /// 指标收集器
    collector: Arc<MetricsCollector>,
    /// 健康检查阈值
    thresholds: HealthThresholds,
}

/// 健康检查阈值
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// 最大内存使用率
    pub max_memory_usage_percent: f64,
    /// 最大CPU使用率
    pub max_cpu_usage_percent: f64,
    /// 最大响应时间（毫秒）
    pub max_response_time_ms: u64,
    /// 最大错误率
    pub max_error_rate: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_memory_usage_percent: 90.0,
            max_cpu_usage_percent: 80.0,
            max_response_time_ms: 1000,
            max_error_rate: 0.05,
        }
    }
}

impl HealthMetrics {
    /// 创建新的健康检查指标
    pub fn new(collector: Arc<MetricsCollector>) -> Self {
        Self {
            collector,
            thresholds: HealthThresholds::default(),
        }
    }

    /// 设置健康检查阈值
    pub fn with_thresholds(mut self, thresholds: HealthThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// 执行健康检查
    pub async fn check_health(&self) -> HealthStatus {
        let mut is_healthy = true;
        let mut checks = Vec::new();

        // 检查内存使用率
        if let Ok(memory_usage) = get_memory_usage() {
            let memory_usage_percent = (memory_usage as f64 / (1024.0 * 1024.0 * 1024.0)) * 100.0; // GB
            let memory_healthy = memory_usage_percent <= self.thresholds.max_memory_usage_percent;
            checks.push(HealthCheck {
                name: "memory_usage".to_string(),
                status: if memory_healthy { "healthy" } else { "unhealthy" },
                value: serde_json::json!({
                    "usage_bytes": memory_usage,
                    "usage_percent": memory_usage_percent,
                    "threshold_percent": self.thresholds.max_memory_usage_percent
                }),
            });
            if !memory_healthy {
                is_healthy = false;
            }
        }

        // 检查CPU使用率
        let cpu_usage = get_cpu_usage();
        let cpu_healthy = cpu_usage <= self.thresholds.max_cpu_usage_percent;
        checks.push(HealthCheck {
            name: "cpu_usage".to_string(),
            status: if cpu_healthy { "healthy" } else { "unhealthy" },
            value: serde_json::json!({
                "usage_percent": cpu_usage,
                "threshold_percent": self.thresholds.max_cpu_usage_percent
            }),
        });
        if !cpu_healthy {
            is_healthy = false;
        }

        HealthStatus {
            healthy: is_healthy,
            timestamp: chrono::Utc::now().to_rfc3339(),
            checks,
        }
    }
}

/// 健康状态
#[derive(Debug, serde::Serialize)]
pub struct HealthStatus {
    /// 是否健康
    pub healthy: bool,
    /// 检查时间
    pub timestamp: String,
    /// 检查项
    pub checks: Vec<HealthCheck>,
}

/// 健康检查项
#[derive(Debug, serde::Serialize)]
pub struct HealthCheck {
    /// 检查项名称
    pub name: String,
    /// 检查状态
    pub status: String,
    /// 检查值
    pub value: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.is_ok());
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        let collector = Arc::new(MetricsCollector::new().unwrap());
        
        // 记录HTTP请求
        collector.record_http_request("GET", "/test", 200, Duration::from_millis(100));
        
        // 记录JSON验证
        collector.record_json_validation("schema", true, Duration::from_millis(50));
        
        // 记录错误
        collector.record_error("validation", "Invalid JSON");
        
        // 更新系统指标
        collector.update_system_metrics();
        
        // 获取指标数据
        let metrics = collector.gather_metrics();
        assert!(metrics.is_ok());
        
        let metrics_str = metrics.unwrap();
        assert!(metrics_str.contains("http_requests_total"));
        assert!(metrics_str.contains("json_validations_total"));
        assert!(metrics_str.contains("errors_total"));
    }

    #[test]
    fn test_parse_prometheus_metrics() {
        let metrics = r#"
# HELP http_requests_total Total number of HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",path="/test",status="200"} 42
http_requests_total{method="POST",path="/api",status="201"} 10
        "#;

        let json_metrics = parse_prometheus_metrics(metrics);
        assert!(json_metrics["metrics"]["http_requests_total"].is_object());
        assert_eq!(json_metrics["metrics"]["http_requests_total"]["{\"method\":\"GET\",\"path\":\"test\",\"status\":\"200\"}"], 42);
    }

    #[test]
    fn test_parse_labels() {
        let labels = r#"method="GET",path="/test",status="200""#;
        let json_labels = parse_labels(labels);
        
        assert_eq!(json_labels["method"], "GET");
        assert_eq!(json_labels["path"], "/test");
        assert_eq!(json_labels["status"], "200");
    }

    #[tokio::test]
    async fn test_health_metrics() {
        let collector = Arc::new(MetricsCollector::new().unwrap());
        let health_metrics = HealthMetrics::new(collector);
        
        let health_status = health_metrics.check_health().await;
        assert!(health_status.healthy);
        assert!(!health_status.checks.is_empty());
    }
}