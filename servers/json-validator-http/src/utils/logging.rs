//! 日志工具模块

use chrono::Utc;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{self},
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
};

/// 日志配置
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式
    pub format: LogFormat,
    /// 是否输出到标准输出
    pub stdout: bool,
}

/// 日志格式
#[derive(Debug, Clone)]
pub enum LogFormat {
    /// JSON格式
    Json,
    /// 文本格式
    Text,
}

/// 设置日志系统
pub fn setup_logging(level: &str) -> anyhow::Result<()> {
    let log_config = LogConfig {
        level: level.to_string(),
        format: LogFormat::Text,
        stdout: true,
    };
    
    setup_logging_with_config(log_config)
}

/// 使用配置设置日志系统
pub fn setup_logging_with_config(config: LogConfig) -> anyhow::Result<()> {
    // 简化的日志配置
    tracing_subscriber::fmt()
        .with_env_filter(&config.level)
        .init();
    
    println!("Logging system initialized with level: {}", config.level);
    Ok(())
}

/// 创建标准输出日志层
fn create_stdout_layer(format: &LogFormat) -> Box<dyn Layer<tracing_subscriber::Registry> + Send + Sync> {
    match format {
        LogFormat::Json => Box::new(fmt::layer().json()),
        LogFormat::Text => Box::new(fmt::layer().compact()),
    }
}

/// 创建请求上下文
pub fn create_request_context(method: &str, uri: &str, status: u16) -> serde_json::Value {
    serde_json::json!({
        "method": method,
        "uri": uri,
        "status": status,
        "timestamp": Utc::now().to_rfc3339()
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_config_creation() {
        let config = LogConfig {
            level: "info".to_string(),
            format: LogFormat::Text,
            stdout: true,
        };
        
        assert_eq!(config.level, "info");
        assert!(matches!(config.format, LogFormat::Text));
        assert!(config.stdout);
    }

    #[test]
    fn test_request_context_creation() {
        let context = create_request_context("GET", "/api/test", 200);
        
        assert_eq!(context["method"], "GET");
        assert_eq!(context["uri"], "/api/test");
        assert_eq!(context["status"], 200);
        assert!(context["timestamp"].is_string());
    }
}