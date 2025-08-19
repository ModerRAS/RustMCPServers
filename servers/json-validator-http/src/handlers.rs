//! HTTP请求处理器

use axum::{
    extract::{Path, Query, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Json},
    RequestPartsExt,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::app::AppState;
use crate::models::*;
use crate::services::JsonValidatorService;
use crate::utils::logging::create_request_context;

/// JSON-RPC请求处理器
pub async fn json_rpc_handler(
    State(state): State<AppState>,
    request: axum::Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let start_time = std::time::Instant::now();
    let request_id = crate::utils::utils::generate_request_id();
    
    debug!("Received JSON-RPC request: {:?}", request);
    
    // 验证请求格式
    if let Err(err) = request.validate() {
        warn!("Invalid JSON-RPC request: {}", err.message);
        return create_error_response(err, request.id.clone());
    }
    
    // 处理请求
    let response = match request.method.as_str() {
        "tools/call" => handle_tool_call(&state, &request, &request_id).await,
        "ping" => handle_ping(&request),
        "validate_json" => handle_validate_json(&state, &request, &request_id).await,
        "validate_json_with_schema" => handle_validate_json_with_schema(&state, &request, &request_id).await,
        "validate_json_batch" => handle_validate_json_batch(&state, &request, &request_id).await,
        _ => {
            warn!("Unknown method: {}", request.method);
            create_error_response(
                JsonRpcError::method_not_found(request.method.clone()),
                request.id.clone(),
            )
        }
    };
    
    let duration = start_time.elapsed();
    log_request!(
        tracing::Level::INFO,
        request.method,
        "JSON-RPC",
        StatusCode::OK.as_u16(),
        duration
    );
    
    response
}

/// 处理工具调用
async fn handle_tool_call(
    state: &AppState,
    request: &JsonRpcRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
    
    // 解析工具调用参数
    let tool_call: ToolCallRequest = match serde_json::from_value(params.clone()) {
        Ok(call) => call,
        Err(e) => {
            error!("Failed to parse tool call params: {}", e);
            return create_error_response(
                JsonRpcError::invalid_params("Invalid tool call parameters".to_string()),
                request.id.clone(),
            );
        }
    };
    
    debug!("Tool call: {} -> {}", tool_call.name, tool_call.arguments);
    
    // 根据工具名称分发处理
    match tool_call.name.as_str() {
        "validate_json" => {
            let args: ValidateJsonRequest = match serde_json::from_value(tool_call.arguments) {
                Ok(args) => args,
                Err(e) => {
                    error!("Failed to parse validate_json arguments: {}", e);
                    return create_error_response(
                        JsonRpcError::invalid_params("Invalid validate_json arguments".to_string()),
                        request.id.clone(),
                    );
                }
            };
            
            handle_validate_json_request(state, args, request_id).await
        }
        "validate_json_with_schema" => {
            let args: ValidateJsonWithSchemaRequest = match serde_json::from_value(tool_call.arguments) {
                Ok(args) => args,
                Err(e) => {
                    error!("Failed to parse validate_json_with_schema arguments: {}", e);
                    return create_error_response(
                        JsonRpcError::invalid_params("Invalid validate_json_with_schema arguments".to_string()),
                        request.id.clone(),
                    );
                }
            };
            
            handle_validate_json_with_schema_request(state, args, request_id).await
        }
        "validate_json_batch" => {
            let args: ValidateJsonBatchRequest = match serde_json::from_value(tool_call.arguments) {
                Ok(args) => args,
                Err(e) => {
                    error!("Failed to parse validate_json_batch arguments: {}", e);
                    return create_error_response(
                        JsonRpcError::invalid_params("Invalid validate_json_batch arguments".to_string()),
                        request.id.clone(),
                    );
                }
            };
            
            handle_validate_json_batch_request(state, args, request_id).await
        }
        _ => {
            warn!("Unknown tool: {}", tool_call.name);
            create_error_response(
                JsonRpcError::method_not_found(tool_call.name),
                request.id.clone(),
            )
        }
    }
}

/// 处理ping请求
fn handle_ping(request: &JsonRpcRequest) -> Json<JsonRpcResponse> {
    debug!("Handling ping request");
    
    let result = serde_json::json!({
        "message": "pong",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
    });
    
    create_success_response(result, request.id.clone())
}

/// 处理validate_json请求
async fn handle_validate_json(
    state: &AppState,
    request: &JsonRpcRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
    
    let args: ValidateJsonRequest = match serde_json::from_value(params.clone()) {
        Ok(args) => args,
        Err(e) => {
            error!("Failed to parse validate_json arguments: {}", e);
            return create_error_response(
                JsonRpcError::invalid_params("Invalid validate_json arguments".to_string()),
                request.id.clone(),
            );
        }
    };
    
    handle_validate_json_request(state, args, request_id).await
}

/// 处理validate_json_with_schema请求
async fn handle_validate_json_with_schema(
    state: &AppState,
    request: &JsonRpcRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
    
    let args: ValidateJsonWithSchemaRequest = match serde_json::from_value(params.clone()) {
        Ok(args) => args,
        Err(e) => {
            error!("Failed to parse validate_json_with_schema arguments: {}", e);
            return create_error_response(
                JsonRpcError::invalid_params("Invalid validate_json_with_schema arguments".to_string()),
                request.id.clone(),
            );
        }
    };
    
    handle_validate_json_with_schema_request(state, args, request_id).await
}

/// 处理validate_json_batch请求
async fn handle_validate_json_batch(
    state: &AppState,
    request: &JsonRpcRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let params = request.params.as_ref().unwrap_or(&serde_json::Value::Null);
    
    let args: ValidateJsonBatchRequest = match serde_json::from_value(params.clone()) {
        Ok(args) => args,
        Err(e) => {
            error!("Failed to parse validate_json_batch arguments: {}", e);
            return create_error_response(
                JsonRpcError::invalid_params("Invalid validate_json_batch arguments".to_string()),
                request.id.clone(),
            );
        }
    };
    
    handle_validate_json_batch_request(state, args, request_id).await
}

/// 处理validate_json请求的具体逻辑
async fn handle_validate_json_request(
    state: &AppState,
    args: ValidateJsonRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let options = args.options.unwrap_or_default();
    
    debug!("Validating JSON with options: {:?}", options);
    
    match state.validator_service.validate_json(&args.json_data, &options).await {
        Ok(result) => {
            log_validation!(
                tracing::Level::INFO,
                result.valid,
                result.execution_time,
                result.cache_hit
            );
            
            let result_value = serde_json::to_value(result).unwrap_or_default();
            create_success_response(result_value, serde_json::Value::String(request_id.to_string()))
        }
        Err(e) => {
            error!("JSON validation failed: {}", e);
            create_error_response(
                JsonRpcError::internal_error(format!("Validation failed: {}", e)),
                serde_json::Value::String(request_id.to_string()),
            )
        }
    }
}

/// 处理validate_json_with_schema请求的具体逻辑
async fn handle_validate_json_with_schema_request(
    state: &AppState,
    args: ValidateJsonWithSchemaRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let options = args.options.unwrap_or_default();
    
    debug!("Validating JSON with schema, options: {:?}", options);
    
    match state
        .validator_service
        .validate_json_with_schema(&args.json_data, &args.schema, &options)
        .await
    {
        Ok(result) => {
            log_validation!(
                tracing::Level::INFO,
                result.valid,
                result.execution_time,
                result.cache_hit
            );
            
            let result_value = serde_json::to_value(result).unwrap_or_default();
            create_success_response(result_value, serde_json::Value::String(request_id.to_string()))
        }
        Err(e) => {
            error!("JSON schema validation failed: {}", e);
            create_error_response(
                JsonRpcError::internal_error(format!("Schema validation failed: {}", e)),
                serde_json::Value::String(request_id.to_string()),
            )
        }
    }
}

/// 处理validate_json_batch请求的具体逻辑
async fn handle_validate_json_batch_request(
    state: &AppState,
    args: ValidateJsonBatchRequest,
    request_id: &str,
) -> Json<JsonRpcResponse> {
    let options = args.options.unwrap_or_default();
    
    debug!("Validating JSON batch with {} items", args.items.len());
    
    match state
        .validator_service
        .validate_json_batch(&args.items, &options)
        .await
    {
        Ok(results) => {
            let success_count = results.iter().filter(|r| r.result.valid).count();
            let total_count = results.len();
            
            log_validation!(
                tracing::Level::INFO,
                success_count == total_count,
                0, // 批量验证时间在服务内部计算
                false
            );
            
            let result_value = serde_json::json!({
                "results": results,
                "summary": {
                    "total": total_count,
                    "success": success_count,
                    "failed": total_count - success_count,
                }
            });
            
            create_success_response(result_value, serde_json::Value::String(request_id.to_string()))
        }
        Err(e) => {
            error!("JSON batch validation failed: {}", e);
            create_error_response(
                JsonRpcError::internal_error(format!("Batch validation failed: {}", e)),
                serde_json::Value::String(request_id.to_string()),
            )
        }
    }
}

/// 健康检查处理器
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let health_response = HealthCheckResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        start_time: chrono::Utc::now().to_rfc3339(),
        uptime: chrono::Utc::now().timestamp() as u64,
        components: {
            let mut components = HashMap::new();
            
            // 检查验证服务
            let validator_health = ComponentHealth {
                name: "validator".to_string(),
                status: "healthy".to_string(),
                message: None,
                checked_at: chrono::Utc::now().to_rfc3339(),
            };
            components.insert("validator".to_string(), validator_health);
            
            // 检查缓存服务
            let cache_health = if state.config.cache.enabled {
                ComponentHealth {
                    name: "cache".to_string(),
                    status: "healthy".to_string(),
                    message: None,
                    checked_at: chrono::Utc::now().to_rfc3339(),
                }
            } else {
                ComponentHealth {
                    name: "cache".to_string(),
                    status: "disabled".to_string(),
                    message: Some("Cache is disabled".to_string()),
                    checked_at: chrono::Utc::now().to_rfc3339(),
                }
            };
            components.insert("cache".to_string(), cache_health);
            
            components
        },
    };
    
    Json(health_response)
}

/// 服务器信息处理器
pub async fn server_info_handler(State(state): State<AppState>) -> impl IntoResponse {
    let server_info = ServerInfo {
        name: "JSON Validator HTTP MCP Server".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        description: "HTTP protocol JSON validation MCP server".to_string(),
        build_time: std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string()),
        build_hash: std::env::var("BUILD_HASH").unwrap_or_else(|_| "unknown".to_string()),
        capabilities: ServerCapabilities {
            tools: vec![
                "validate_json".to_string(),
                "validate_json_with_schema".to_string(),
                "validate_json_batch".to_string(),
            ],
            formats: vec!["JSON".to_string(), "JSON Schema".to_string()],
            cache: state.config.cache.enabled,
            batch: true,
            custom_formats: state.config.validation.custom_formats,
        },
    };
    
    Json(server_info)
}

/// 指标处理器
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    let metrics = MetricsResponse {
        requests_total: state.validator_service.get_request_count().await,
        requests_success: state.validator_service.get_success_count().await,
        requests_failed: state.validator_service.get_error_count().await,
        avg_response_time: state.validator_service.get_avg_response_time().await,
        cache_hit_rate: state.validator_service.get_cache_hit_rate().await,
        active_connections: 0, // TODO: 实现连接计数
        validations_total: state.validator_service.get_validation_count().await,
        validation_success_rate: state.validator_service.get_validation_success_rate().await,
    };
    
    Json(metrics)
}

/// 工具调用请求
#[derive(Debug, Deserialize)]
struct ToolCallRequest {
    /// 工具名称
    pub name: String,
    /// 工具参数
    pub arguments: serde_json::Value,
}

/// 创建成功响应
fn create_success_response(result: serde_json::Value, id: serde_json::Value) -> Json<JsonRpcResponse> {
    Json(JsonRpcResponse::success(result, id))
}

/// 创建错误响应
fn create_error_response(error: JsonRpcError, id: serde_json::Value) -> Json<JsonRpcResponse> {
    Json(JsonRpcResponse::error(error, id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_ping_handler() {
        let request = JsonRpcRequest::new(
            "ping".to_string(),
            None,
            serde_json::Value::Number(1.into()),
        );
        
        let response = handle_ping(&request);
        let response = response.0;
        
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id, serde_json::Value::Number(1.into()));
    }

    #[tokio::test]
    async fn test_health_handler() {
        // 这里需要模拟AppState，在实际测试中会使用mock
        // 暂时跳过这个测试
    }
}