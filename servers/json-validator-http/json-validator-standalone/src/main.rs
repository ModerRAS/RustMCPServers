use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Debug, Clone)]
struct AppState {
    // 简单的状态管理
    config: ServerConfig,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    max_connections: usize,
    timeout: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8082,
            max_connections: 1000,
            timeout: 30,
        }
    }
}

// JSON-RPC 请求结构
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

// JSON-RPC 响应结构
#[derive(Debug, Serialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    result: Option<T>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

// 验证结果
#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    execution_time: u64,
}

#[derive(Debug, Serialize)]
struct ValidationError {
    instance_path: String,
    schema_path: String,
    message: String,
}

// 健康检查响应
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
    timestamp: String,
}

// 服务器信息响应
#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
    description: String,
    capabilities: Vec<String>,
}

// Ping响应
#[derive(Debug, Serialize)]
struct PingResponse {
    message: String,
    timestamp: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 设置日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting JSON Validator HTTP Server");

    // 创建应用状态
    let state = AppState {
        config: ServerConfig::default(),
    };

    // 创建路由
    let app = create_app(Arc::new(state));

    // 启动服务器
    let addr = SocketAddr::from(([127, 0, 0, 1], 8082));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server listening on {}", addr);

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

fn create_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/info", get(server_info))
        .route("/rpc", post(handle_rpc))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 0,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

async fn server_info() -> Json<ServerInfo> {
    Json(ServerInfo {
        name: "JSON Validator HTTP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "HTTP protocol JSON validation MCP server".to_string(),
        capabilities: vec![
            "validate_json".to_string(),
            "validate_json_with_schema".to_string(),
            "validate_json_batch".to_string(),
            "ping".to_string(),
        ],
    })
}

async fn handle_rpc(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<JsonRpcRequest>,
) -> Result<Json<JsonRpcResponse<Value>>, StatusCode> {
    // 验证JSON-RPC版本
    if request.jsonrpc != "2.0" {
        return Ok(Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid JSON-RPC version".to_string(),
                data: None,
            }),
            id: request.id,
        }));
    }

    // 处理不同的方法
    let result = match request.method.as_str() {
        "ping" => handle_ping().await,
        "validate_json" => handle_validate_json(request.params).await,
        "validate_json_with_schema" => handle_validate_json_with_schema(request.params).await,
        "validate_json_batch" => handle_validate_json_batch(request.params).await,
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(result_value) => Ok(Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result_value),
            error: None,
            id: request.id,
        })),
        Err(error) => Ok(Json(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id: request.id,
        })),
    }
}

async fn handle_ping() -> Result<Value, JsonRpcError> {
    Ok(json!({
        "message": "pong",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn handle_validate_json(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    // 简单的JSON格式验证
    let json_data = params.get("json_data").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing json_data parameter".to_string(),
        data: None,
    })?;

    let start_time = std::time::Instant::now();
    let mut errors = Vec::new();

    // 基本JSON格式检查
    if let Some(obj) = json_data.as_object() {
        // 检查基本结构
        if obj.contains_key("name") && !obj["name"].is_string() {
            errors.push(ValidationError {
                instance_path: "/name".to_string(),
                schema_path: "/properties/name/type".to_string(),
                message: "name must be a string".to_string(),
            });
        }
        
        if obj.contains_key("age") && !obj["age"].is_number() {
            errors.push(ValidationError {
                instance_path: "/age".to_string(),
                schema_path: "/properties/age/type".to_string(),
                message: "age must be a number".to_string(),
            });
        }
    }

    let execution_time = start_time.elapsed().as_millis() as u64;

    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "execution_time": execution_time
    }))
}

async fn handle_validate_json_with_schema(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let json_data = params.get("json_data").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing json_data parameter".to_string(),
        data: None,
    })?;

    let schema = params.get("schema").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing schema parameter".to_string(),
        data: None,
    })?;

    let start_time = std::time::Instant::now();
    let mut errors = Vec::new();

    // 简化的schema验证
    if let Some(schema_obj) = schema.as_object() {
        if schema_obj.get("type").is_some() {
            if let Some(data_obj) = json_data.as_object() {
                for (key, value) in data_obj {
                    if let Some(properties) = schema_obj.get("properties") {
                        if let Some(props) = properties.as_object() {
                            if let Some(prop_schema) = props.get(key) {
                                if let Some(type_constraint) = prop_schema.get("type") {
                                    if type_constraint == "string" && !value.is_string() {
                                        errors.push(ValidationError {
                                            instance_path: format!("/{}", key),
                                            schema_path: format!("/properties/{}/type", key),
                                            message: format!("{} must be a string", key),
                                        });
                                    } else if type_constraint == "number" && !value.is_number() {
                                        errors.push(ValidationError {
                                            instance_path: format!("/{}", key),
                                            schema_path: format!("/properties/{}/type", key),
                                            message: format!("{} must be a number", key),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let execution_time = start_time.elapsed().as_millis() as u64;

    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "execution_time": execution_time
    }))
}

async fn handle_validate_json_batch(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;

    let items = params.get("items").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing items parameter".to_string(),
        data: None,
    })?;

    let items_array = items.as_array().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "items must be an array".to_string(),
        data: None,
    })?;

    let mut results = Vec::new();

    for (index, item) in items_array.iter().enumerate() {
        let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or(&index.to_string()).to_string();
        let json_data = item.get("json_data");
        
        if let Some(data) = json_data {
            let result = handle_validate_json(Some(json!({"json_data": data}))).await;
            match result {
                Ok(validation_result) => {
                    results.push(json!({
                        "id": item_id,
                        "result": validation_result
                    }));
                }
                Err(_) => {
                    results.push(json!({
                        "id": item_id,
                        "error": {
                            "code": -32603,
                            "message": "Internal error"
                        }
                    }));
                }
            }
        } else {
            results.push(json!({
                "id": item_id,
                "error": {
                    "code": -32602,
                    "message": "Missing json_data"
                }
            }));
        }
    }

    Ok(json!({
        "results": results,
        "total": results.len()
    }))
}

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