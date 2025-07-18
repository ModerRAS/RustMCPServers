mod auth;
mod auth_routes;
mod client;
mod config;
mod duckduckgo;
mod mcp_handler;
mod mcp_types;

use anyhow::Result;
use axum::{
    http::StatusCode,
    middleware,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, FmtSubscriber};

use crate::config::ServerConfig;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let config = ServerConfig::from_env();

    // Initialize tracing with configurable log level
    let log_level = match config.log_level.as_str() {
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true),
        )
        .with(tracing_subscriber::EnvFilter::from_default_env().add_directive(log_level.into()))
        .init();

    info!(
        "Starting DuckDuckGo MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );
    info!(
        "Configuration loaded: host={}, port={}",
        config.host, config.port
    );

    let mcp_state = Arc::new(mcp_handler::McpState::new(config.clone()).await);

    // Configure CORS
    // Configure CORS
    let cors = if config.cors_origins.contains(&"*".to_string()) {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(
                config
                    .cors_origins
                    .iter()
                    .map(|origin| origin.parse().unwrap())
                    .collect::<Vec<_>>(),
            )
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // Build the application
    let app = Router::new()
        // MCP protocol endpoints
        .route("/mcp", post(mcp_handler::handle_initialize))
        .route("/mcp/initialize", post(mcp_handler::handle_initialize))
        .route("/mcp/tools/list", post(mcp_handler::handle_list_tools))
        .route("/mcp/tools/call", post(mcp_handler::handle_call_tool))
        .route("/mcp/ping", post(mcp_handler::handle_ping))
        // Authentication endpoints
        .route("/auth/login", post(auth_routes::login_handler))
        .route("/auth/validate", post(auth_routes::validate_token_handler))
        .route("/auth/tokens", post(auth_routes::add_static_token_handler))
        .route(
            "/auth/tokens/remove",
            post(auth_routes::remove_static_token_handler),
        )
        // Health check and metrics
        .route("/health", get(health_check))
        .route("/metrics", get(metrics))
        // Apply middleware
        .layer(middleware::from_fn_with_state(
            mcp_state.clone(),
            auth::auth_middleware,
        ))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(mcp_state);

    let addr = format!("{}:{}", config.host, config.port);
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let response = serde_json::json!({
        "status": "healthy",
        "service": "duckduckgo-mcp-server",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    Ok(axum::Json(response))
}

async fn metrics() -> Result<axum::Json<serde_json::Value>, StatusCode> {
    let response = serde_json::json!({
        "service": "duckduckgo-mcp-server",
        "uptime": "running",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    Ok(axum::Json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let config = ServerConfig::default();
        let mcp_state = Arc::new(mcp_handler::McpState::new(config).await);

        let app = Router::new()
            .route("/health", get(health_check))
            .with_state(mcp_state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mcp_endpoints() {
        let config = ServerConfig::default();
        let mcp_state = Arc::new(mcp_handler::McpState::new(config).await);

        let app = Router::new()
            .route("/mcp/ping", post(mcp_handler::handle_ping))
            .with_state(mcp_state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/mcp/ping")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
