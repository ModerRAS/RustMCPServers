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

async fn health_check() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": "0.1.0",
        "service": "task-orchestrator-mcp"
    }))
}

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
        let config = Config::from_env().unwrap();
        
        assert!(!config.server.host.is_empty());
        assert!(config.server.port > 0);
        assert!(config.task.max_retries > 0);
    }

    #[tokio::test]
    async fn test_server_initialization() {
        let _config = Config::from_env().unwrap();
        let repository = Arc::new(InMemoryTaskRepository::new());
        let _server = TaskOrchestratorServer::new(repository);
        
        // Test that server can be created
    }
}