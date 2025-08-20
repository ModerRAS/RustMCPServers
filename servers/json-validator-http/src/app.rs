//! 应用程序配置和路由

use axum::{
    routing::{get, post},
    Router,
    response::Json,
};
use std::sync::Arc;
use crate::handlers::{json_rpc_handler, health_check};
use crate::models::AppState;

/// 创建应用程序路由
pub fn create_app() -> Router {
    // 创建应用状态
    let state = AppState::new();
    
    // 创建路由
    Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_check))
        .route("/rpc", post(json_rpc_handler))
        .with_state(state)
}

/// 根路径处理器
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "JSON Validator HTTP MCP Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "HTTP protocol JSON validation MCP server",
        "endpoints": {
            "rpc": "/rpc - JSON-RPC 2.0 endpoint",
            "health": "/health - Health check endpoint"
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_app() {
        let app = create_app();
        
        // 测试根路径
        let request = Request::builder()
            .uri("/")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_app();
        
        // 测试健康检查端点
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}