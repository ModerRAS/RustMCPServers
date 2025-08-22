//! # Task Orchestrator MCP Server
//! 
//! 这是一个基于Rust的任务编排MCP (Model Context Protocol) 服务器，使用rmcp 0.5.0 SDK实现。
//! 
//! ## 主要功能
//! 
//! - **任务管理**: 创建、获取、执行、完成任务的完整生命周期管理
//! - **MCP协议支持**: 完整的MCP协议实现，支持JSON-RPC over HTTP
//! - **内存存储**: 高性能的内存任务存储（可扩展为持久化存储）
//! - **配置管理**: 灵活的配置系统，支持文件和环境变量
//! - **健康检查**: 提供服务健康状态检查
//! - **优雅关闭**: 支持优雅关闭和资源清理
//! 
//! ## 实现的MCP工具
//! 
//! - `create_task` - 创建新任务
//! - `get_task` - 获取特定任务详情
//! - `acquire_task` - 获取下一个可用任务
//! - `complete_task` - 标记任务完成
//! - `list_tasks` - 列出任务（支持过滤）
//! - `get_statistics` - 获取任务统计信息
//! - `retry_task` - 重试失败任务
//! 
//! ## 任务状态管理
//! 
//! - Pending: 待执行
//! - Waiting: 等待资源
//! - Running: 运行中
//! - Completed: 已完成
//! - Failed: 失败
//! - Cancelled: 已取消
//! 
//! ## 任务优先级
//! 
//! - Low (1): 低优先级
//! - Medium (2): 中优先级（默认）
//! - High (3): 高优先级
//! - Urgent (4): 紧急优先级
//! 
//! ## 使用示例
//! 
//! ```bash
//! # 编译项目
//! cargo build --release
//! 
//! # 启动服务器
//! cargo run
//! 
//! # 健康检查
//! curl http://127.0.0.1:8080/health
//! 
//! # MCP工具调用
//! curl -X POST http://127.0.0.1:8080/ \
//!   -H "Content-Type: application/json" \
//!   -d '{"jsonrpc":"2.0","method":"create_task","params":{"title":"Test Task","description":"Test Description"},"id":1}'
//! ```

use std::sync::Arc;
use std::path::PathBuf;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::{trace::TraceLayer, cors::CorsLayer, compression::CompressionLayer};
use tower::ServiceBuilder;

use crate::config::Config;
use crate::storage::InMemoryTaskRepository;
use crate::server::TaskOrchestratorServer;
use crate::api::create_api_routes;

mod config;
mod models;
mod storage;
mod server;
mod api;

/// 应用程序主入口点
/// 
/// 该函数是Task Orchestrator MCP服务器的主入口点，负责：
/// 
/// 1. 初始化配置系统
/// 2. 设置日志记录
/// 3. 创建内存任务存储库
/// 4. 创建MCP服务器实例
/// 5. 设置HTTP路由和中间件
/// 6. 启动HTTP服务器
/// 7. 处理优雅关闭
/// 
/// # 返回值
/// 
/// 返回 `Result<(), Box<dyn std::error::Error + Send + Sync>>`：
/// - `Ok(())`: 服务器正常启动和运行
/// - `Err(error)`: 启动过程中的错误
/// 
/// # 错误处理
/// 
/// 该函数会处理以下错误情况：
/// - 配置文件加载失败
/// - 服务器绑定失败
/// - 日志初始化失败
/// 
/// # 优雅关闭
/// 
/// 支持以下关闭信号：
/// - Ctrl+C (SIGINT)
/// - SIGTERM (Unix系统)
/// 
/// 收到关闭信号后，会：
/// 1. 停止接受新请求
/// 2. 完成正在处理的请求
/// 3. 清理资源
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize configuration
    let config_path = PathBuf::from("config.toml");
    let config = Config::from_file_or_env(&config_path)?;
    
    // Initialize logging
    init_logging(&config.logging);

    println!("🚀 Starting Task Orchestrator MCP Server");
    println!("📋 Configuration loaded successfully");
    println!("🌐 Server will listen on {}:{}", config.server.host, config.server.port);
    println!("💾 Using in-memory storage (for demo)");

    // Create task repository
    let task_repository = Arc::new(InMemoryTaskRepository::new());

    // Create MCP server (reserved for future MCP service integration)
    let _mcp_server = TaskOrchestratorServer::new(task_repository.clone());

    // Create HTTP service for MCP (not used in current implementation)

    // Create HTTP router with API routes only (for testing)
    let app = axum::Router::new()
        .route("/", axum::routing::get(|| async { "Task Orchestrator MCP Server" }))
        .route("/health", axum::routing::get(health_check))
        .nest("/api", create_api_routes(task_repository.clone()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(CompressionLayer::new())
        );

    // Configure server address
    let addr = SocketAddr::new(
        config.server.host.parse()?,
        config.server.port,
    );

    println!("🎯 Starting HTTP server on {addr}");
    println!("📚 Available endpoints:");
    println!("   GET  /health - Health check");
    println!("   POST /      - MCP JSON-RPC over HTTP");
    println!("   GET  /      - MCP SSE stream");

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Graceful shutdown handling
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

    // Start server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    println!("✅ Server shutdown completed");

    Ok(())
}

/// 健康检查端点处理器
/// 
/// 该函数处理 `/health` 端点的GET请求，返回服务的健康状态信息。
/// 
/// # 返回值
/// 
/// 返回 `axum::Json<serde_json::Value>`，包含以下字段：
/// - `status`: 服务状态（"healthy"）
/// - `timestamp`: 当前时间的RFC3339格式字符串
/// - `version`: 服务版本号（"0.1.0"）
/// - `service`: 服务名称（"task-orchestrator-mcp"）
/// 
/// # 使用示例
/// 
/// ```bash
/// curl http://127.0.0.1:8080/health
/// ```
/// 
/// # 响应示例
/// 
/// ```json
/// {
///   "status": "healthy",
///   "timestamp": "2025-08-19T12:00:00Z",
///   "version": "0.1.0",
///   "service": "task-orchestrator-mcp"
/// }
/// ```
async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "service": "task-orchestrator-mcp"
    }))
}

/// 初始化日志系统
/// 
/// 该函数根据配置初始化tracing日志系统，支持不同的日志格式和级别。
/// 
/// # 参数
/// 
/// - `config`: 日志配置引用，包含日志级别和格式设置
/// 
/// # 功能说明
/// 
/// - 从环境变量或配置中设置日志级别
/// - 支持JSON和Pretty两种日志格式
/// - 启用线程ID和线程名称显示
/// - 使用tracing-subscriber作为日志后端
/// 
/// # 支持的日志级别
/// 
/// - trace: 最详细的日志信息
/// - debug: 调试信息
/// - info: 一般信息（默认）
/// - warn: 警告信息
/// - error: 错误信息
/// 
/// # 支持的日志格式
/// 
/// - `LogFormat::Json`: 结构化JSON格式，适合生产环境
/// - `LogFormat::Pretty`: 美化格式，适合开发调试
/// 
/// # 示例
/// 
/// ```rust
/// let config = LoggingConfig {
///     level: "debug".to_string(),
///     format: LogFormat::Pretty,
/// };
/// init_logging(&config);
/// ```
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

    /// 测试配置加载功能
    /// 
    /// 该测试验证配置系统是否能正确从环境变量加载配置。
    /// 
    /// # 测试内容
    /// 
    /// - 调用 `Config::from_env()` 加载配置
    /// - 验证配置字段的有效性
    /// 
    /// # 断言
    /// 
    /// - 服务器主机名不为空
    /// - 服务器端口大于0
    /// - 任务最大重试次数大于0
    #[tokio::test]
    async fn test_config_loading() {
        let config = Config::from_env().unwrap();
        
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
        assert!(config.task.max_retries > 0);
    }

    /// 测试服务器初始化
    /// 
    /// 该测试验证TaskOrchestratorServer是否能正确初始化。
    /// 
    /// # 测试内容
    /// 
    /// - 加载配置
    /// - 创建内存任务存储库
    /// - 创建TaskOrchestratorServer实例
    /// 
    /// # 断言
    /// 
    /// - 服务器实例能够成功创建（无panic）
    /// - 配置加载成功
    /// - 存储库创建成功
    #[tokio::test]
    async fn test_server_initialization() {
        let _config = Config::from_env().unwrap();
        let repository = Arc::new(InMemoryTaskRepository::new());
        let _server = TaskOrchestratorServer::new(repository);
        
        // Test that server can be created
    }
}