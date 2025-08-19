use anyhow::Result;
use clap::Parser;
use config::Config;
use json_validator_http::app::create_app;
use json_validator_http::config::ServerConfig;
use json_validator_http::utils::logging::setup_logging;
use std::net::SocketAddr;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

/// HTTP协议JSON验证MCP服务器
#[derive(Parser)]
#[command(name = "json-validator-http")]
#[command(about = "HTTP protocol JSON validation MCP server")]
#[command(version, long_about = None)]
struct Args {
    /// 配置文件路径
    #[arg(short, long, default_value = "config/default.toml")]
    config: String,
    
    /// 监听地址
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    listen: SocketAddr,
    
    /// 日志级别
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// 启用开发模式
    #[arg(long, default_value = "false")]
    dev: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();
    
    // 设置日志系统
    setup_logging(&args.log_level)?;
    
    info!("Starting JSON Validator HTTP MCP server");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("Listen address: {}", args.listen);
    
    // 加载配置
    let config = load_config(&args.config)?;
    info!("Configuration loaded from: {}", args.config);
    
    // 创建应用
    let app = create_app(config.clone()).await?;
    
    // 添加追踪层
    let app = app.layer(
        TraceLayer::new_for_http()
            .make_span_with(|request: &axum::http::Request<_>| {
                tracing::info_span!(
                    "http_request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            })
            .on_request(|_request: &axum::http::Request<_>, _span: &tracing::Span| {
                tracing::info!("request received");
            })
            .on_response(|_response: &axum::http::Response<_>, latency: std::time::Duration, _span: &tracing::Span| {
                tracing::info!("response generated in {:?}", latency);
            }),
    );
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(args.listen).await?;
    info!("Server listening on {}", args.listen);
    
    // 设置优雅关闭
    let graceful_shutdown = async {
        shutdown_signal().await;
        info!("Shutdown signal received, starting graceful shutdown");
    };
    
    // 运行服务器
    axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown)
        .await?;
    
    info!("Server shutdown complete");
    Ok(())
}

/// 加载配置文件
fn load_config(config_path: &str) -> Result<ServerConfig> {
    let mut settings = Config::builder()
        .add_source(config::File::with_name(config_path).required(false))
        .add_source(config::Environment::with_prefix("JSON_VALIDATOR"))
        .build()?;
    
    // 设置默认值
    settings.set_default("server.host", "127.0.0.1")?;
    settings.set_default("server.port", 8080)?;
    settings.set_default("server.workers", 4)?;
    settings.set_default("server.max_connections", 1000)?;
    settings.set_default("server.timeout", 30)?;
    
    settings.set_default("cache.enabled", false)?;
    settings.set_default("cache.ttl", 3600)?;
    settings.set_default("cache.max_size", 1000)?;
    
    settings.set_default("security.enabled", false)?;
    settings.set_default("security.jwt_secret", "default-secret")?;
    settings.set_default("security.rate_limit", 100)?;
    settings.set_default("security.cors.enabled", true)?;
    
    settings.set_default("logging.level", "info")?;
    settings.set_default("logging.format", "json")?;
    
    settings.set_default("metrics.enabled", true)?;
    settings.set_default("metrics.port", 9090)?;
    
    let config: ServerConfig = settings.try_deserialize()?;
    Ok(config)
}

/// 等待关闭信号
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };
    
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
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
}