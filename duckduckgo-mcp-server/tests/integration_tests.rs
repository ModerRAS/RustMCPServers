use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use duckduckgo_mcp_server::{config::ServerConfig, mcp_handler::McpState};
use serde_json::json;
use tower::util::ServiceExt;

async fn create_test_app() -> axum::Router {
    let config = ServerConfig::default();
    let state = std::sync::Arc::new(McpState::new(config).await);

    use duckduckgo_mcp_server::mcp_handler::{
        handle_call_tool, handle_initialize, handle_list_tools, handle_ping,
    };

    axum::Router::new()
        .route("/mcp/initialize", axum::routing::post(handle_initialize))
        .route("/mcp/tools/list", axum::routing::post(handle_list_tools))
        .route("/mcp/tools/call", axum::routing::post(handle_call_tool))
        .route("/mcp/ping", axum::routing::post(handle_ping))
        .with_state(state)
}

#[tokio::test]
async fn test_mcp_initialize() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/initialize")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "initialize",
                        "params": {
                            "protocolVersion": "2024-11-05",
                            "capabilities": {},
                            "clientInfo": {
                                "name": "test-client",
                                "version": "1.0.0"
                            }
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"].is_object());
    assert_eq!(
        response["result"]["serverInfo"]["name"],
        "duckduckgo-mcp-server"
    );
}

#[tokio::test]
async fn test_mcp_tools_list() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/tools/list")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/list"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["tools"].is_array());

    let tools = response["result"]["tools"].as_array().unwrap();
    assert!(tools.len() >= 2); // Should have search and search_news

    let tool_names: Vec<String> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();

    assert!(tool_names.contains(&"search".to_string()));
    assert!(tool_names.contains(&"search_news".to_string()));
}

#[tokio::test]
async fn test_mcp_ping() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/ping")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "ping"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert_eq!(response["result"]["pong"], true);
}

#[tokio::test]
async fn test_mcp_search_tool_call() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/call",
                        "params": {
                            "name": "search",
                            "arguments": {
                                "query": "rust programming language",
                                "max_results": 3
                            }
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["content"].is_array());
}

#[tokio::test]
async fn test_mcp_invalid_tool_call() {
    let app = create_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/mcp/tools/call")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "jsonrpc": "2.0",
                        "id": 1,
                        "method": "tools/call",
                        "params": {
                            "name": "invalid_tool",
                            "arguments": {}
                        }
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["error"].is_object());
    assert!(response["error"]["message"]
        .as_str()
        .unwrap()
        .contains("Unknown tool"));
}

#[tokio::test]
async fn test_health_check_endpoint() {
    let config = ServerConfig::default();
    let state = std::sync::Arc::new(McpState::new(config).await);

    let app = axum::Router::new()
        .route(
            "/health",
            axum::routing::get(|| async {
                axum::response::Json(serde_json::json!({"status": "healthy"}))
            }),
        )
        .with_state(state);

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

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["status"], "healthy");
    assert_eq!(response["service"], "duckduckgo-mcp-server");
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let config = ServerConfig::default();
    let state = std::sync::Arc::new(McpState::new(config).await);

    let app = axum::Router::new()
        .route(
            "/metrics",
            axum::routing::get(|| async {
                axum::response::Json(serde_json::json!({"service": "test"}))
            }),
        )
        .with_state(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/metrics")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response["service"], "duckduckgo-mcp-server");
    assert_eq!(response["uptime"], "running");
}
