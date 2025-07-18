mod auth;
mod auth_routes;
mod duckduckgo;
mod mcp_handler;
mod mcp_types;

use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(name = "duckduckgo-mcp-server")]
#[command(about = "DuckDuckGo MCP Server with HTTP transport")]
#[command(version = "0.1.0")]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host to bind to
    #[arg(short, long, default_value = "127.0.0.1")]
    host: String,

    /// Secret key for JWT token generation
    #[arg(long, default_value = "your-secret-key-change-this")]
    secret_key: String,

    /// Require authentication for all requests
    #[arg(long, default_value = "false")]
    require_auth: bool,

    /// Static API tokens (comma-separated)
    #[arg(long)]
    static_tokens: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

    info!("Starting DuckDuckGo MCP Server");
    info!("Configuration: {:?}", args);

    // Initialize authentication state
    let auth_state = Arc::new(auth::AuthState::new(
        args.secret_key.clone(),
        args.require_auth,
    ));

    // Add static tokens if provided
    if let Some(tokens) = args.static_tokens {
        for token in tokens.split(',') {
            let token = token.trim().to_string();
            if !token.is_empty() {
                auth_state.add_static_token(token).await;
            }
        }
    }

    // Initialize MCP state
    let mcp_state = Arc::new(mcp_handler::McpState::new(auth_state.clone()));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

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
        .route("/auth/tokens/remove", post(auth_routes::remove_static_token_handler))
        // Health check
        .route("/health", get(auth_routes::health_check))
        // Apply authentication middleware
        .layer(middleware::from_fn_with_state(
            auth_state.clone(),
            auth::auth_middleware,
        ))
        .layer(cors)
        .with_state(mcp_state);

    let addr = format!("{}:{}", args.host, args.port);
    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let auth_state = Arc::new(auth::AuthState::new(
            "test-secret".to_string(),
            false,
        ));
        let mcp_state = Arc::new(mcp_handler::McpState::new(auth_state));

        let app = Router::new()
            .route("/health", get(auth_routes::health_check))
            .with_state(mcp_state);

        let response = app
            .oneshot(Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_mcp_endpoints() {
        let auth_state = Arc::new(auth::AuthState::new(
            "test-secret".to_string(),
            false,
        ));
        let mcp_state = Arc::new(mcp_handler::McpState::new(auth_state));

        let app = Router::new()
            .route("/mcp/ping", post(mcp_handler::handle_ping))
            .with_state(mcp_state);

        let response = app
            .oneshot(Request::builder()
                .method("POST")
                .uri("/mcp/ping")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#))
                .unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
