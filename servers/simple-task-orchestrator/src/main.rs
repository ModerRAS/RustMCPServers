use std::sync::Arc;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::{trace::TraceLayer, cors::CorsLayer};

use crate::config::ConfigManager;
use crate::infrastructure::{InMemoryTaskRepository, SimpleLockManager};
use crate::services::{TaskService, TaskScheduler, TaskMonitor, TaskExecutionService};
use crate::handlers::{create_routes, ApiState};
use crate::utils::RateLimiter;

mod config;
mod domain;
mod infrastructure;
mod services;
mod handlers;
mod errors;
mod utils;
mod execution;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化配置
    let config_manager = ConfigManager::new()?;
    let config = config_manager.config().clone();

    // 初始化日志
    init_logging(&config.logging);

    println!("🚀 Starting Simple Task Orchestrator MCP Server");
    println!("📋 Configuration loaded successfully");
    println!("🌐 Server will listen on {}:{}", config.server.host, config.server.port);

    // 创建内存仓库
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    
    // 创建锁管理器
    let lock_manager = Arc::new(SimpleLockManager::new());
    
    // 创建任务服务
    let task_service = Arc::new(TaskService::new(
        task_repository.clone(),
        lock_manager,
        config.task.max_retries,
        config.task.timeout,
    ));

    // 创建任务调度器
    let task_scheduler = TaskScheduler::new(
        task_service.clone(),
        config.task.cleanup_interval,
        config.task.heartbeat_interval,
    );

    // 创建任务监控器
    let task_monitor = TaskMonitor::new(
        task_service.clone(),
        config.monitoring.metrics_interval,
    );

    // 创建速率限制器
    let _rate_limiter = Arc::new(RateLimiter::new(config.security.rate_limit));

    // 创建执行服务
    let execution_service = Arc::new(TaskExecutionService::new(task_repository.clone()));
    
    // 创建API状态
    let api_state = ApiState {
        task_service: task_service.clone(),
        execution_service: execution_service.clone(),
    };

    // 启动后台任务
    task_scheduler.start().await?;
    task_monitor.start().await?;

    println!("🔄 Background tasks started");

    // 创建HTTP服务
    let app = create_routes().with_state(api_state)
        // 添加中间件
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // 配置服务器地址
    let addr = SocketAddr::new(
        config.server.host.parse()?,
        config.server.port,
    );

    println!("🎯 Starting server on {}", addr);

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
            let _ = signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("Failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }

        println!("🛑 Shutdown signal received");
    };

    // 启动服务器
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    println!("✅ Server shutdown completed");

    Ok(())
}

/// 初始化日志系统
fn init_logging(config: &crate::config::LoggingConfig) {
    use tracing_subscriber::{fmt, EnvFilter, prelude::*};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    match config.format {
        crate::config::LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer.compact())
                .init();
        }
        crate::config::LogFormat::Pretty => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer.pretty())
                .init();
        }
    }

    tracing::info!("Logging initialized with level: {}", config.level);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_config_loading() {
        let config_manager = ConfigManager::new().unwrap();
        let config = config_manager.config();
        
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
        assert!(config.task.max_retries > 0);
    }

    #[tokio::test]
    async fn test_server_initialization() {
        let config_manager = ConfigManager::new().unwrap();
        let config = config_manager.config();
        
        let task_repository = Arc::new(InMemoryTaskRepository::new());
        let lock_manager = Arc::new(SimpleLockManager::new());
        
        let _task_service = Arc::new(TaskService::new(
            task_repository,
            lock_manager,
            config.task.max_retries,
            config.task.timeout,
        ));
        
        // 测试配置加载成功
        assert!(config.task.max_retries > 0);
        assert!(config.task.timeout > 0);
    }
}