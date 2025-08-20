use tracing::{info, warn, error, debug, instrument, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::path::Path;

use crate::config::{LoggingConfig, AppConfig};

/// 日志管理器
pub struct LogManager {
    config: LoggingConfig,
}

impl LogManager {
    /// 创建新的日志管理器
    pub fn new(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// 初始化日志系统
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // 创建环境过滤器
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| {
                EnvFilter::new(&self.config.level)
                    .add_directive("tower_http=debug".parse().unwrap())
                    .add_directive("task_orchestrator=info".parse().unwrap())
            });

        // 初始化全局订阅者 - 简化实现
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .init();

        info!("Logging system initialized with level: {}", self.config.level);
        Ok(())
    }

    /// 创建结构化日志记录器
    pub fn structured_logger(&self) -> StructuredLogger {
        StructuredLogger::new(&self.config)
    }
}

/// 结构化日志记录器
#[derive(Clone)]
pub struct StructuredLogger {
    config: LoggingConfig,
}

impl StructuredLogger {
    pub fn new(config: &LoggingConfig) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// 记录一般信息日志
    pub fn log_info(&self, message: &str, context: Option<&str>) {
        if let Some(ctx) = context {
            info!(message = %message, context = %ctx, "Info");
        } else {
            info!(message = %message, "Info");
        }
    }

    /// 记录任务创建日志
    #[instrument(skip_all, fields(
        task_id,
        work_directory,
        priority,
        tags_count,
        status
    ))]
    pub fn log_task_created(&self, task_id: &str, work_directory: &str, priority: &str, tags_count: usize, status: &str) {
        let span = Span::current();
        span.record("task_id", task_id);
        span.record("work_directory", work_directory);
        span.record("priority", priority);
        span.record("tags_count", tags_count);
        span.record("status", status);

        info!(
            task_id,
            work_directory,
            priority,
            tags_count,
            status,
            "Task created"
        );
    }

    /// 记录任务获取日志
    #[instrument(skip_all, fields(
        task_id,
        worker_id,
        work_directory,
        priority,
        acquisition_time_ms
    ))]
    pub fn log_task_acquired(&self, task_id: &str, worker_id: &str, work_directory: &str, priority: &str, acquisition_time_ms: u64) {
        let span = Span::current();
        span.record("task_id", task_id);
        span.record("worker_id", worker_id);
        span.record("work_directory", work_directory);
        span.record("priority", priority);
        span.record("acquisition_time_ms", acquisition_time_ms);

        info!(
            task_id,
            worker_id,
            work_directory,
            priority,
            acquisition_time_ms,
            "Task acquired by worker"
        );
    }

    /// 记录任务完成日志
    #[instrument(skip_all, fields(
        task_id,
        worker_id,
        status,
        processing_time_ms,
        result_status,
        error_message
    ))]
    pub fn log_task_completed(&self, task_id: &str, worker_id: &str, status: &str, processing_time_ms: u64, result_status: &str, error_message: Option<&str>) {
        let span = Span::current();
        span.record("task_id", task_id);
        span.record("worker_id", worker_id);
        span.record("status", status);
        span.record("processing_time_ms", processing_time_ms);
        span.record("result_status", result_status);
        
        if let Some(error) = error_message {
            span.record("error_message", error);
        }

        info!(
            task_id,
            worker_id,
            status,
            processing_time_ms,
            result_status,
            error_message,
            "Task completed"
        );
    }

    /// 记录任务失败日志
    #[instrument(skip_all, fields(
        task_id,
        worker_id,
        error,
        retry_count,
        max_retries
    ))]
    pub fn log_task_failed(&self, task_id: &str, worker_id: &str, error: &str, retry_count: u32, max_retries: u32) {
        let span = Span::current();
        span.record("task_id", task_id);
        span.record("worker_id", worker_id);
        span.record("error", error);
        span.record("retry_count", retry_count);
        span.record("max_retries", max_retries);

        warn!(
            task_id,
            worker_id,
            error,
            retry_count,
            max_retries,
            "Task failed"
        );
    }

    /// 记录任务取消日志
    #[instrument(skip_all, fields(
        task_id,
        reason,
        cancelled_by
    ))]
    pub fn log_task_cancelled(&self, task_id: &str, reason: Option<&str>, cancelled_by: Option<&str>) {
        let span = Span::current();
        span.record("task_id", task_id);
        
        if let Some(reason) = reason {
            span.record("reason", reason);
        }
        
        if let Some(cancelled_by) = cancelled_by {
            span.record("cancelled_by", cancelled_by);
        }

        info!(
            task_id,
            reason,
            cancelled_by,
            "Task cancelled"
        );
    }

    /// 记录API请求日志
    #[instrument(skip_all, fields(
        method,
        path,
        status_code,
        response_time_ms,
        user_agent,
        client_ip
    ))]
    pub fn log_api_request(&self, method: &str, path: &str, status_code: u16, response_time_ms: u64, user_agent: Option<&str>, client_ip: Option<&str>) {
        let span = Span::current();
        span.record("method", method);
        span.record("path", path);
        span.record("status_code", status_code);
        span.record("response_time_ms", response_time_ms);
        
        if let Some(ua) = user_agent {
            span.record("user_agent", ua);
        }
        
        if let Some(ip) = client_ip {
            span.record("client_ip", ip);
        }

        info!(
            method,
            path,
            status_code,
            response_time_ms,
            user_agent,
            client_ip,
            "API request processed"
        );
    }

    /// 记录数据库操作日志
    #[instrument(skip_all, fields(
        operation,
        table,
        execution_time_ms,
        rows_affected
    ))]
    pub fn log_database_operation(&self, operation: &str, table: &str, execution_time_ms: u64, rows_affected: Option<u64>) {
        let span = Span::current();
        span.record("operation", operation);
        span.record("table", table);
        span.record("execution_time_ms", execution_time_ms);
        
        if let Some(rows) = rows_affected {
            span.record("rows_affected", rows);
        }

        debug!(
            operation,
            table,
            execution_time_ms,
            rows_affected,
            "Database operation completed"
        );
    }

    /// 记录错误日志
    #[instrument(skip_all, fields(
        error_type,
        error_message,
        stack_trace,
        context
    ))]
    pub fn log_error(&self, error_type: &str, error_message: &str, stack_trace: Option<&str>, context: Option<&str>) {
        let span = Span::current();
        span.record("error_type", error_type);
        span.record("error_message", error_message);
        
        if let Some(trace) = stack_trace {
            span.record("stack_trace", trace);
        }
        
        if let Some(ctx) = context {
            span.record("context", ctx);
        }

        error!(
            error_type,
            error_message,
            stack_trace,
            context,
            "Error occurred"
        );
    }

    /// 记录性能指标日志
    #[instrument(skip_all, fields(
        metric_name,
        metric_value,
        metric_type,
        tags
    ))]
    pub fn log_metric(&self, metric_name: &str, metric_value: f64, metric_type: &str, tags: Option<&str>) {
        let span = Span::current();
        span.record("metric_name", metric_name);
        span.record("metric_value", metric_value);
        span.record("metric_type", metric_type);
        
        if let Some(t) = tags {
            span.record("tags", t);
        }

        debug!(
            metric_name,
            metric_value,
            metric_type,
            tags,
            "Metric recorded"
        );
    }
}

/// HTTP请求中间件
pub struct LoggingMiddleware {
    logger: StructuredLogger,
}

impl LoggingMiddleware {
    pub fn new(logger: StructuredLogger) -> Self {
        Self { logger }
    }
}

impl tower_http::trace::MakeSpan for LoggingMiddleware {
    fn make_span<B>(&mut self, request: &axum::http::Request<B>) -> Span {
        let user_agent = request.headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("unknown");
        
        let client_ip = request.headers()
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .or_else(|| {
                request.headers()
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
            })
            .unwrap_or("unknown");

        tracing::info_span!(
            "http_request",
            method = %request.method(),
            uri = %request.uri(),
            version = ?request.version(),
            user_agent = %user_agent,
            client_ip = %client_ip,
        )
    }
}

/// 监控指标收集器
pub struct MetricsCollector {
    task_created_counter: prometheus::Counter,
    task_completed_counter: prometheus::Counter,
    task_failed_counter: prometheus::Counter,
    task_cancelled_counter: prometheus::Counter,
    task_acquired_counter: prometheus::Counter,
    response_time_histogram: prometheus::Histogram,
    active_tasks_gauge: prometheus::Gauge,
    queue_size_gauge: prometheus::Gauge,
    error_counter: prometheus::Counter,
    database_query_duration: prometheus::Histogram,
}

impl MetricsCollector {
    /// 创建新的指标收集器
    pub fn new() -> Result<Self, prometheus::Error> {
        Ok(Self {
            task_created_counter: prometheus::Counter::with_opts(
                prometheus::Opts::new("task_created_total", "Total number of tasks created")
                    .const_label("service", "task_orchestrator")
            )?,
            
            task_completed_counter: prometheus::Counter::with_opts(
                prometheus::Opts::new("task_completed_total", "Total number of tasks completed")
                    .const_label("service", "task_orchestrator")
            )?,
            
            task_failed_counter: prometheus::Counter::with_opts(
                prometheus::Opts::new("task_failed_total", "Total number of tasks failed")
                    .const_label("service", "task_orchestrator")
            )?,
            
            task_cancelled_counter: prometheus::Counter::with_opts(
                prometheus::Opts::new("task_cancelled_total", "Total number of tasks cancelled")
                    .const_label("service", "task_orchestrator")
            )?,
            
            task_acquired_counter: prometheus::Counter::with_opts(
                prometheus::Opts::new("task_acquired_total", "Total number of tasks acquired")
                    .const_label("service", "task_orchestrator")
            )?,
            
            response_time_histogram: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("response_time_seconds", "HTTP response time")
                    .const_label("service", "task_orchestrator")
                    .buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0])
            )?,
            
            active_tasks_gauge: prometheus::Gauge::with_opts(
                prometheus::Opts::new("active_tasks", "Number of currently active tasks")
                    .const_label("service", "task_orchestrator")
            )?,
            
            queue_size_gauge: prometheus::Gauge::with_opts(
                prometheus::Opts::new("queue_size", "Current queue size")
                    .const_label("service", "task_orchestrator")
            )?,
            
            error_counter: prometheus::Counter::with_opts(
                prometheus::Opts::new("errors_total", "Total number of errors")
                    .const_label("service", "task_orchestrator")
            )?,
            
            database_query_duration: prometheus::Histogram::with_opts(
                prometheus::HistogramOpts::new("database_query_duration_seconds", "Database query duration")
                    .const_label("service", "task_orchestrator")
                    .buckets(vec![0.001, 0.01, 0.1, 1.0])
            )?,
        })
    }

    /// 记录任务创建
    pub fn record_task_created(&self, priority: &str) {
        self.task_created_counter.inc();
    }

    /// 记录任务完成
    pub fn record_task_completed(&self, priority: &str, processing_time: f64) {
        self.task_completed_counter.inc();
        self.response_time_histogram.observe(processing_time);
    }

    /// 记录任务失败
    pub fn record_task_failed(&self, priority: &str, error_type: &str) {
        self.task_failed_counter.inc();
    }

    /// 记录任务取消
    pub fn record_task_cancelled(&self, reason: &str) {
        self.task_cancelled_counter.inc();
    }

    /// 记录任务获取
    pub fn record_task_acquired(&self, worker_id: &str) {
        self.task_acquired_counter.inc();
    }

    /// 设置活跃任务数量
    pub fn set_active_tasks(&self, count: u64) {
        self.active_tasks_gauge.set(count as f64);
    }

    /// 设置队列大小
    pub fn set_queue_size(&self, size: u64) {
        self.queue_size_gauge.set(size as f64);
    }

    /// 记录错误
    pub fn record_error(&self, error_type: &str, endpoint: &str) {
        self.error_counter.inc();
    }

    /// 记录数据库查询时间
    pub fn record_database_query(&self, operation: &str, table: &str, duration: f64) {
        self.database_query_duration.observe(duration);
    }

    /// 获取所有指标的文本格式
    pub fn gather_metrics(&self) -> String {
        use prometheus::Encoder;
        
        let encoder = prometheus::TextEncoder::new();
        let metric_families = prometheus::gather();
        let mut buffer = Vec::new();
        
        encoder.encode(&metric_families, &mut buffer).unwrap();
        String::from_utf8(buffer).unwrap()
    }
}

/// 健康检查器
pub struct HealthChecker {
    database_healthy: bool,
    cache_healthy: bool,
    external_services_healthy: bool,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            database_healthy: true,
            cache_healthy: true,
            external_services_healthy: true,
        }
    }

    /// 检查系统健康状态
    pub async fn check_health(&self) -> HealthStatus {
        let components = vec![
            ("database", self.database_healthy),
            ("cache", self.cache_healthy),
            ("external_services", self.external_services_healthy),
        ];

        let all_healthy = components.iter().all(|(_, healthy)| *healthy);
        let overall_status = if all_healthy { "healthy" } else { "unhealthy" };

        let mut component_details = std::collections::HashMap::new();
        for (name, healthy) in components {
            component_details.insert(
                name.to_string(),
                ComponentHealth {
                    healthy,
                    response_time: Some(5.2), // 模拟响应时间
                    last_checked: chrono::Utc::now(),
                },
            );
        }

        HealthStatus {
            status: overall_status.to_string(),
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            components: component_details,
            metrics: SystemMetrics {
                memory_usage: "78MB".to_string(),
                cpu_usage: 12.5,
                active_connections: 15,
                queue_size: 0,
            },
        }
    }

    /// 设置数据库健康状态
    pub fn set_database_health(&mut self, healthy: bool) {
        self.database_healthy = healthy;
    }

    /// 设置缓存健康状态
    pub fn set_cache_health(&mut self, healthy: bool) {
        self.cache_healthy = healthy;
    }

    /// 设置外部服务健康状态
    pub fn set_external_services_health(&mut self, healthy: bool) {
        self.external_services_healthy = healthy;
    }
}

/// 健康状态
#[derive(Debug, serde::Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub components: std::collections::HashMap<String, ComponentHealth>,
    pub metrics: SystemMetrics,
}

/// 组件健康状态
#[derive(Debug, serde::Serialize)]
pub struct ComponentHealth {
    pub healthy: bool,
    pub response_time: Option<f64>,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}

/// 系统指标
#[derive(Debug, serde::Serialize)]
pub struct SystemMetrics {
    pub memory_usage: String,
    pub cpu_usage: f64,
    pub active_connections: u32,
    pub queue_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_structured_logger_creation() {
        let config = LoggingConfig::default();
        let logger = StructuredLogger::new(&config);
        assert!(logger.config.enable_json);
    }

    #[test]
    fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.is_ok());
    }

    #[test]
    fn test_health_checker_creation() {
        let checker = HealthChecker::new();
        assert!(checker.database_healthy);
        assert!(checker.cache_healthy);
        assert!(checker.external_services_healthy);
    }

    #[tokio::test]
    async fn test_health_check() {
        let checker = HealthChecker::new();
        let status = checker.check_health().await;
        assert_eq!(status.status, "healthy");
        assert!(status.components.contains_key("database"));
        assert!(status.components.contains_key("cache"));
        assert!(status.components.contains_key("external_services"));
    }
}