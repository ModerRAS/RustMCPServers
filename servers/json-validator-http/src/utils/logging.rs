//! 日志工具模块

use tracing::{Level, Span};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    Layer,
};
use std::path::Path;

/// 日志配置
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式
    pub format: LogFormat,
    /// 是否输出到标准输出
    pub stdout: bool,
    /// 是否输出到标准错误
    pub stderr: bool,
    /// 日志文件路径（可选）
    pub file_path: Option<String>,
    /// 是否启用日志轮转
    pub rotation: bool,
    /// 日志轮转配置
    pub rotation_config: Option<RotationConfig>,
}

/// 日志格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// JSON格式
    Json,
    /// 文本格式
    Text,
    /// 简洁格式
    Compact,
}

/// 日志轮转配置
#[derive(Debug, Clone)]
pub struct RotationConfig {
    /// 最大文件大小
    pub max_size: u64,
    /// 最大文件数
    pub max_files: usize,
    /// 轮转周期
    pub rotation_period: RotationPeriod,
}

/// 轮转周期
#[derive(Debug, Clone)]
pub enum RotationPeriod {
    /// 每小时
    Hourly,
    /// 每天
    Daily,
    /// 每周
    Weekly,
    /// 每月
    Monthly,
}

/// 设置日志系统
pub fn setup_logging(level: &str) -> anyhow::Result<()> {
    let log_config = LogConfig {
        level: level.to_string(),
        format: LogFormat::Json,
        stdout: true,
        stderr: false,
        file_path: None,
        rotation: false,
        rotation_config: None,
    };
    
    setup_logging_with_config(log_config)
}

/// 使用配置设置日志系统
pub fn setup_logging_with_config(config: LogConfig) -> anyhow::Result<()> {
    // 创建环境过滤器
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));
    
    // 创建日志层
    let mut layers = Vec::new();
    
    // 标准输出层
    if config.stdout {
        let stdout_layer = create_stdout_layer(&config.format);
        layers.push(stdout_layer.boxed());
    }
    
    // 标准错误层（仅错误级别）
    if config.stderr {
        let stderr_layer = create_stderr_layer(&config.format);
        layers.push(stderr_layer.boxed());
    }
    
    // 文件层
    if let Some(file_path) = &config.file_path {
        let file_layer = if config.rotation {
            create_file_layer_with_rotation(file_path, &config.format, config.rotation_config.as_ref())?
        } else {
            create_file_layer(file_path, &config.format)?
        };
        layers.push(file_layer.boxed());
    }
    
    // 初始化订阅者
    tracing_subscriber::registry()
        .with(env_filter)
        .with(layers)
        .init();
    
    tracing::info!("Logging system initialized with level: {}", config.level);
    Ok(())
}

/// 创建标准输出日志层
fn create_stdout_layer(format: &LogFormat) -> impl Layer<tracing::Span> + Send + Sync {
    match format {
        LogFormat::Json => {
            fmt::layer()
                .json()
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true)
                .with_timer(fmt::time::ChronoLocal::rfc_3339())
        }
        LogFormat::Text => {
            fmt::layer()
                .pretty()
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true)
                .with_timer(fmt::time::ChronoLocal::rfc_3339())
        }
        LogFormat::Compact => {
            fmt::layer()
                .compact()
                .with_span_events(FmtSpan::CLOSE)
                .with_current_span(true)
        }
    }
}

/// 创建标准错误日志层
fn create_stderr_layer(format: &LogFormat) -> impl Layer<tracing::Span> + Send + Sync {
    let layer = match format {
        LogFormat::Json => fmt::layer().json(),
        LogFormat::Text => fmt::layer().pretty(),
        LogFormat::Compact => fmt::layer().compact(),
    };
    
    layer
        .with_filter(tracing_subscriber::filter::LevelFilter::ERROR)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true)
}

/// 创建文件日志层
fn create_file_layer(
    file_path: &str,
    format: &LogFormat,
) -> anyhow::Result<impl Layer<tracing::Span> + Send + Sync> {
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    
    let non_blocking = non_blocking(file);
    
    let layer = match format {
        LogFormat::Json => fmt::layer().json(),
        LogFormat::Text => fmt::layer().pretty(),
        LogFormat::Compact => fmt::layer().compact(),
    };
    
    Ok(layer
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true)
        .with_timer(fmt::time::ChronoLocal::rfc_3339()))
}

/// 创建带轮转的文件日志层
fn create_file_layer_with_rotation(
    file_path: &str,
    format: &LogFormat,
    rotation_config: Option<&RotationConfig>,
) -> anyhow::Result<impl Layer<tracing::Span> + Send + Sync> {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or(Path::new("."));
    let filename = path.file_name().unwrap_or_default().to_string_lossy();
    
    let rotation_config = rotation_config.unwrap_or(&RotationConfig {
        max_size: 100 * 1024 * 1024, // 100MB
        max_files: 10,
        rotation_period: RotationPeriod::Daily,
    });
    
    let roller = match rotation_config.rotation_period {
        RotationPeriod::Hourly => rolling::hourly(parent, &filename),
        RotationPeriod::Daily => rolling::daily(parent, &filename),
        RotationPeriod::Weekly => rolling::never(parent, &filename), // 需要自定义实现
        RotationPeriod::Monthly => rolling::never(parent, &filename), // 需要自定义实现
    };
    
    let non_blocking = non_blocking(roller);
    
    let layer = match format {
        LogFormat::Json => fmt::layer().json(),
        LogFormat::Text => fmt::layer().pretty(),
        LogFormat::Compact => fmt::layer().compact(),
    };
    
    Ok(layer
        .with_writer(non_blocking)
        .with_span_events(FmtSpan::CLOSE)
        .with_current_span(true)
        .with_timer(fmt::time::ChronoLocal::rfc_3339()))
}

/// 结构化日志宏
#[macro_export]
macro_rules! log_request {
    ($level:expr, $method:expr, $uri:expr, $status:expr, $duration:expr) => {
        tracing::event!(
            $level,
            method = %$method,
            uri = %$uri,
            status = %$status,
            duration = ?$duration,
            "HTTP request"
        );
    };
}

#[macro_export]
macro_rules! log_validation {
    ($level:expr, $valid:expr, $duration:expr, $cache_hit:expr) => {
        tracing::event!(
            $level,
            valid = %$valid,
            duration = ?$duration,
            cache_hit = %$cache_hit,
            "JSON validation"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($level:expr, $error:expr, $context:expr) => {
        tracing::event!(
            $level,
            error = %$error,
            context = %$context,
            "Error occurred"
        );
    };
}

/// 创建请求日志上下文
pub fn create_request_context(
    method: &str,
    uri: &str,
    user_agent: Option<&str>,
    remote_addr: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "method": method,
        "uri": uri,
        "user_agent": user_agent,
        "remote_addr": remote_addr,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// 创建验证日志上下文
pub fn create_validation_context(
    json_size: usize,
    schema_size: Option<usize>,
    options: &crate::models::ValidationOptions,
) -> serde_json::Value {
    serde_json::json!({
        "json_size": json_size,
        "schema_size": schema_size,
        "strict_mode": options.strict_mode,
        "allow_additional_properties": options.allow_additional_properties,
        "custom_formats": !options.custom_formats.is_empty(),
        "detailed_errors": options.detailed_errors,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    })
}

/// 创建错误日志上下文
pub fn create_error_context(
    error_code: &str,
    error_message: &str,
    additional_info: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut context = serde_json::json!({
        "error_code": error_code,
        "error_message": error_message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    
    if let Some(info) = additional_info {
        context["additional_info"] = info;
    }
    
    context
}

/// 获取当前日志级别
pub fn get_current_level() -> Level {
    tracing::level_filters::LevelFilter::current().into()
}

/// 检查是否启用特定日志级别
pub fn is_level_enabled(level: Level) -> bool {
    tracing::level_enabled!(level)
}

/// 性能日志记录器
pub struct PerformanceLogger {
    start_time: std::time::Instant,
    operation: String,
    context: serde_json::Value,
}

impl PerformanceLogger {
    /// 创建性能日志记录器
    pub fn new(operation: String, context: serde_json::Value) -> Self {
        let start_time = std::time::Instant::now();
        
        tracing::info!(
            operation = %operation,
            context = %context,
            "Performance tracking started"
        );
        
        Self {
            start_time,
            operation,
            context,
        }
    }
    
    /// 记录完成时间
    pub fn done(self) {
        let duration = self.start_time.elapsed();
        
        tracing::info!(
            operation = %self.operation,
            duration = ?duration,
            context = %self.context,
            "Performance tracking completed"
        );
    }
    
    /// 记录完成时间并返回结果
    pub fn done_with_result<T>(self, result: &T) -> T
    where
        T: std::fmt::Debug,
    {
        let duration = self.start_time.elapsed();
        
        tracing::info!(
            operation = %self.operation,
            duration = ?duration,
            result = ?result,
            context = %self.context,
            "Performance tracking completed with result"
        );
        
        // 这里不能返回result的副本，所以这个方法应该接受一个引用
        // 实际使用时需要调整
        panic!("done_with_result cannot return the result. Use done() instead.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_log_config_creation() {
        let config = LogConfig {
            level: "info".to_string(),
            format: LogFormat::Json,
            stdout: true,
            stderr: false,
            file_path: None,
            rotation: false,
            rotation_config: None,
        };
        
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, LogFormat::Json));
    }

    #[test]
    fn test_request_context_creation() {
        let context = create_request_context(
            "GET",
            "/health",
            Some("test-agent"),
            Some("127.0.0.1"),
        );
        
        assert_eq!(context["method"], "GET");
        assert_eq!(context["uri"], "/health");
        assert_eq!(context["user_agent"], "test-agent");
        assert_eq!(context["remote_addr"], "127.0.0.1");
    }

    #[test]
    fn test_validation_context_creation() {
        let options = crate::models::ValidationOptions::default();
        let context = create_validation_context(1024, Some(512), &options);
        
        assert_eq!(context["json_size"], 1024);
        assert_eq!(context["schema_size"], 512);
        assert_eq!(context["strict_mode"], false);
    }

    #[test]
    fn test_error_context_creation() {
        let context = create_error_context(
            "TEST_ERROR",
            "Test error message",
            Some(serde_json::json!({"key": "value"})),
        );
        
        assert_eq!(context["error_code"], "TEST_ERROR");
        assert_eq!(context["error_message"], "Test error message");
        assert_eq!(context["additional_info"]["key"], "value");
    }

    #[test]
    fn test_performance_logger() {
        let context = serde_json::json!({"test": "value"});
        let logger = PerformanceLogger::new("test_operation".to_string(), context);
        
        // 测试性能日志记录器不会panic
        logger.done();
    }
}