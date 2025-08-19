use std::sync::Arc;
use rmcp::{ServiceExt, transport::stdio};

use simple_task_orchestrator::config::ConfigManager;
use simple_task_orchestrator::infrastructure::{InMemoryTaskRepository, SimpleLockManager};
use simple_task_orchestrator::services::{TaskService, TaskScheduler, TaskMonitor, TaskExecutionService};
use simple_task_orchestrator::mcp_server::TaskOrchestratorServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 初始化配置
    let config_manager = ConfigManager::new()?;
    let config = config_manager.config().clone();

    // 初始化日志
    init_logging(&config.logging);

    println!("🚀 Starting Simple Task Orchestrator MCP Server");
    println!("📋 Configuration loaded successfully");
    println!("🌐 Server running in stdio mode");

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

    
    // 创建执行服务
    let execution_service = Arc::new(TaskExecutionService::new(task_repository.clone()));
    
    // 创建MCP服务器
    let mcp_server = TaskOrchestratorServer::new(
        task_service.clone(),
        execution_service.clone(),
    );

    // 启动后台任务
    task_scheduler.start().await?;
    task_monitor.start().await?;

    println!("🔄 Background tasks started");

    println!("🎯 Starting MCP server in stdio mode");

    // 启动MCP服务器
    let server = mcp_server.serve(stdio()).await.inspect_err(|e| {
        eprintln!("Server error: {:?}", e);
    })?;
    
    // 等待服务器完成
    server.waiting().await?;

    println!("✅ Server shutdown completed");

    Ok(())
}

/// 初始化日志系统
fn init_logging(config: &simple_task_orchestrator::config::LoggingConfig) {
    use tracing_subscriber::{fmt, EnvFilter, prelude::*};

    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true);

    match config.format {
        simple_task_orchestrator::config::LogFormat::Json => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer.compact())
                .init();
        }
        simple_task_orchestrator::config::LogFormat::Pretty => {
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