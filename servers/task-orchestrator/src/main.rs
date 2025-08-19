use std::sync::Arc;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::{trace::TraceLayer, cors::CorsLayer, compression::CompressionLayer, timeout::TimeoutLayer, request_id::MakeRequestUuid};
use tower::ServiceBuilder;

use crate::config::{ConfigManager, AppConfig};
use crate::infrastructure::{TaskRepository, SqliteTaskRepository, SqliteLockManager};
use crate::services::{TaskService, TaskScheduler, TaskMonitor};
use crate::handlers::{create_routes, ApiState};
use crate::utils::{LogManager, StructuredLogger, MetricsCollector, HealthChecker, ConcurrencyController, RateLimiter};

mod config;
mod domain;
mod models;
mod infrastructure;
mod services;
mod handlers;
mod errors;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化配置
    let config_manager = ConfigManager::new()?;
    let config = config_manager.config().clone();

    // 初始化日志系统
    let log_manager = LogManager::new(config.logging.clone());
    log_manager.init()?;
    let logger = log_manager.structured_logger();

    logger.log_info("Starting Task Orchestrator MCP Server", None);
    logger.log_info(&format!("Environment: {}", config.environment), None);
    logger.log_info(&format!("Version: {}", config.version), None);

    // 创建数据库连接池
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(&config.database.url)
        .await?;

    // 创建任务仓库
    let task_repository: Arc<dyn TaskRepository> = Arc::new(
        SqliteTaskRepository::with_pool(pool.clone()).await?
    );

    // 创建锁管理器
    let lock_manager = Arc::new(
        SqliteLockManager::with_pool(pool.clone()).await
    );

    // 创建并发控制器
    let concurrency_controller = ConcurrencyController::new(
        lock_manager.clone(),
        config.task.max_concurrent_tasks as usize,
        std::time::Duration::from_secs(config.task.worker_timeout),
        std::time::Duration::from_secs(config.task.heartbeat_interval),
    );

    // 创建速率限制器
    let rate_limiter = RateLimiter::new(
        config.security.rate_limit_requests_per_minute,
        std::time::Duration::from_secs(60),
    );

    // 创建任务服务
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        config.task.max_retries,
        config.task.default_task_timeout,
    ));

    // 创建任务调度器
    let task_scheduler = TaskScheduler::new(
        task_service.clone(),
        config.task.task_cleanup_interval,
        config.task.heartbeat_interval,
    );

    // 创建任务监控器
    let task_monitor = TaskMonitor::new(
        task_service.clone(),
        config.monitoring.metrics_collection_interval,
    );

    // 创建指标收集器
    let metrics_collector = MetricsCollector::new()?;

    // 创建健康检查器
    let health_checker = HealthChecker::new();

    // 创建API状态
    let api_state = ApiState {
        task_service: task_service.clone(),
        logger: logger.clone(),
    };

    // 启动后台任务
    task_scheduler.start().await?;
    task_monitor.start().await?;
    concurrency_controller.start_cleanup_task().await?;
    rate_limiter.start_cleanup_task().await?;

    logger.log_info("Background tasks started", None);

    // 创建HTTP服务
    let app = create_routes(api_state);

    // 添加中间件
    let app = app.layer(
        ServiceBuilder::new()
            // 请求ID
            .layer(MakeRequestUuid::default())
            // 超时
            .layer(TimeoutLayer::new(std::time::Duration::from_secs(config.server.timeout)))
            // 压缩
            .layer(CompressionLayer::new())
            // CORS
            .layer(CorsLayer::permissive())
            // 追踪
            .layer(TraceLayer::new_for_http())
    );

    // 配置服务器地址
    let addr = SocketAddr::new(
        config.server.host.parse()?,
        config.server.port,
    );

    logger.log_info(&format!("Starting server on {}", addr), None);

    // 启动服务器
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // 优雅关闭处理
    let shutdown_signal = async {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .await
                .expect("Failed to install signal handler");
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        logger.log_info("Shutdown signal received", None);
    };

    // 启动服务器
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    logger.log_info("Server shutdown completed", None);

    Ok(())
}


impl StructuredLogger {
    pub fn log_info(&self, message: &str, context: Option<&str>) {
        if let Some(ctx) = context {
            tracing::info!(message = %message, context = %ctx);
        } else {
            tracing::info!(message = %message);
        }
    }

    pub fn log_warn(&self, message: &str, context: Option<&str>) {
        if let Some(ctx) = context {
            tracing::warn!(message = %message, context = %ctx);
        } else {
            tracing::warn!(message = %message);
        }
    }

    pub fn log_error(&self, message: &str, context: Option<&str>) {
        if let Some(ctx) = context {
            tracing::error!(message = %message, context = %ctx);
        } else {
            tracing::error!(message = %message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        let config_manager = ConfigManager::new().unwrap();
        let config = config_manager.config();
        
        assert!(!config.database.url.is_empty());
        assert!(config.server.port > 0);
        assert!(config.server.workers > 0);
    }

    #[tokio::test]
    async fn test_logging_initialization() {
        let config = AppConfig::from_env().unwrap();
        let log_manager = LogManager::new(config.logging);
        let result = log_manager.init();
        assert!(result.is_ok());
    }
}