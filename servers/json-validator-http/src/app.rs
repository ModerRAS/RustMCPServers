//! 应用层 - HTTP路由和应用配置

use axum::{
    extract::State,
    http::{Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{
    cors::CorsLayer,
    compression::CompressionLayer,
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower::ServiceBuilder;
use tracing::info;

use crate::config::ServerConfig;
use crate::handlers::{
    health_handler, json_rpc_handler, metrics_handler, server_info_handler,
};
use crate::middleware::{
    auth::AuthLayer, logging::LoggingLayer, metrics::MetricsLayer,
    rate_limit::RateLimitLayer,
};
use crate::services::JsonValidatorService;
use crate::performance::PerformanceManager;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    /// JSON验证服务
    pub validator_service: Arc<JsonValidatorService>,
    /// 服务器配置
    pub config: Arc<ServerConfig>,
    /// 性能管理器
    pub performance_manager: Arc<PerformanceManager>,
}

/// 创建HTTP应用
pub async fn create_app(config: ServerConfig) -> anyhow::Result<Router> {
    // 验证配置
    config.validate()?;
    
    // 创建JSON验证服务
    let validator_service = Arc::new(JsonValidatorService::new(config.clone()).await?);
    
    // 创建性能管理器
    let performance_config = config.get_performance_config();
    let performance_manager = Arc::new(PerformanceManager::new(performance_config));
    
    // 创建应用状态
    let app_state = AppState {
        validator_service,
        config: Arc::new(config),
        performance_manager,
    };
    
    // 基础路由
    let app = Router::new()
        // JSON-RPC端点
        .route("/rpc", post(json_rpc_handler))
        // 健康检查端点
        .route("/health", get(health_handler))
        // 服务器信息端点
        .route("/info", get(server_info_handler))
        // 指标端点
        .route("/metrics", get(metrics_handler))
        // 性能统计端点
        .route("/performance", get(performance_stats_handler))
        // 根路径
        .route("/", get(root_handler))
        .with_state(app_state.clone());
    
    // 添加中间件层
    let app = apply_middleware(app, &app_state.config, &app_state.performance_manager).await;
    
    info!("Application created successfully with performance optimizations");
    Ok(app)
}

/// 应用中间件
async fn apply_middleware(
    app: Router,
    config: &ServerConfig,
    performance_manager: &Arc<PerformanceManager>,
) -> Router {
    let mut service_builder = ServiceBuilder::new();
    
    // 1. 请求限制层
    if config.server.max_request_size > 0 {
        service_builder = service_builder.layer(RequestBodyLimitLayer::new(
            config.server.max_request_size,
        ));
    }
    
    // 2. 超时层
    if config.server.timeout > 0 {
        service_builder = service_builder.layer(TimeoutLayer::new(
            std::time::Duration::from_secs(config.server.timeout),
        ));
    }
    
    // 3. 压缩层
    if config.server.compression {
        service_builder = service_builder.layer(CompressionLayer::new());
    }
    
    // 4. CORS层
    if config.security.cors.enabled {
        let cors = CorsLayer::new()
            .allow_methods([
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::DELETE,
                Method::OPTIONS,
            ])
            .allow_headers(tower_http::cors::Any)
            .allow_origin(if config.security.cors.allow_origins.contains(&"*".to_string()) {
                tower_http::cors::Any
            } else {
                config.security.cors.allow_origins.iter().map(|s| s.as_str()).collect()
            });
        
        service_builder = service_builder.layer(cors);
    }
    
    // 5. 日志层
    service_builder = service_builder.layer(LoggingLayer::new());
    
    // 6. 指标层
    if config.metrics.enabled {
        service_builder = service_builder.layer(MetricsLayer::new());
    }
    
    // 7. 限流层
    if config.security.enabled && config.security.rate_limit > 0 {
        service_builder = service_builder.layer(RateLimitLayer::new(
            config.security.rate_limit,
            std::time::Duration::from_secs(60),
        ));
    }
    
    // 8. 认证层
    if config.security.enabled {
        service_builder = service_builder.layer(AuthLayer::new(
            config.security.jwt_secret.clone(),
        ));
    }
    
    // 9. 追踪层
    service_builder = service_builder.layer(
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
    
    app.layer(service_builder)
}

/// 根路径处理器
async fn root_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "name": "JSON Validator HTTP MCP Server",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "HTTP protocol JSON validation MCP server with performance optimizations",
        "endpoints": {
            "rpc": "/rpc - JSON-RPC 2.0 endpoint",
            "health": "/health - Health check endpoint",
            "info": "/info - Server information",
            "metrics": "/metrics - Metrics endpoint",
            "performance": "/performance - Performance statistics"
        },
        "features": {
            "performance_optimizations": true,
            "memory_pool": true,
            "connection_pooling": true,
            "caching": true,
            "compression": true
        },
        "documentation": "https://github.com/RustMCPServers/RustMCPServers"
    }))
}

/// 性能统计处理器
async fn performance_stats_handler(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let stats = state.performance_manager.get_stats().await;
    
    // 获取服务统计信息
    let service_stats = state.validator_service.get_stats().await;
    
    let performance_data = serde_json::json!({
        "server_stats": {
            "current_concurrent_requests": stats.current_concurrent_requests,
            "peak_concurrent_requests": stats.peak_concurrent_requests,
            "total_requests": stats.total_requests,
            "successful_requests": stats.successful_requests,
            "failed_requests": stats.failed_requests,
            "success_rate": if stats.total_requests > 0 {
                stats.successful_requests as f64 / stats.total_requests as f64
            } else {
                0.0
            }
        },
        "response_time_stats": {
            "avg_response_time_ms": stats.avg_response_time_ms,
            "p95_response_time_ms": stats.p95_response_time_ms,
            "p99_response_time_ms": stats.p99_response_time_ms
        },
        "system_stats": {
            "memory_usage_mb": stats.memory_usage_mb,
            "cpu_usage_percent": stats.cpu_usage_percent,
            "cache_hit_rate_percent": stats.cache_hit_rate_percent
        },
        "validation_stats": {
            "total_validations": service_stats.validations_total,
            "successful_validations": service_stats.validations_success,
            "failed_validations": service_stats.validations_failed,
            "validation_success_rate": if service_stats.validations_total > 0 {
                service_stats.validations_success as f64 / service_stats.validations_total as f64
            } else {
                0.0
            },
            "cache_hits": service_stats.cache_hits,
            "cache_misses": service_stats.cache_misses,
            "cache_hit_rate": if service_stats.cache_hits + service_stats.cache_misses > 0 {
                service_stats.cache_hits as f64 / (service_stats.cache_hits + service_stats.cache_misses) as f64
            } else {
                0.0
            }
        },
        "performance_config": {
            "max_concurrent_requests": state.config.performance.concurrency.max_concurrent_requests,
            "connection_pool_size": state.config.performance.basic.connection_pool_size,
            "memory_limit_mb": state.config.performance.memory.memory_limit_mb,
            "enable_memory_pool": state.config.performance.memory.enable_memory_pool,
            "enable_compression": state.config.performance.basic.enable_compression
        }
    });
    
    Ok(Json(performance_data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_app() {
        let config = ServerConfig::default();
        let app = create_app(config).await.unwrap();
        
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
    async fn test_performance_endpoint() {
        let config = ServerConfig::default();
        let app = create_app(config).await.unwrap();
        
        // 测试性能统计端点
        let request = Request::builder()
            .uri("/performance")
            .method("GET")
            .body(Body::empty())
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rpc_endpoint_exists() {
        let config = ServerConfig::default();
        let app = create_app(config).await.unwrap();
        
        // 测试RPC端点
        let request = Request::builder()
            .uri("/rpc")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"jsonrpc":"2.0","method":"ping","params":{},"id":1}"#))
            .unwrap();
        
        let response = app.oneshot(request).await.unwrap();
        assert!(response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_performance_integration() {
        let config = ServerConfig::default();
        let app = create_app(config).await.unwrap();
        
        // 测试多个并发请求
        let mut handles = vec![];
        
        for i in 0..10 {
            let app = app.clone();
            let handle = tokio::spawn(async move {
                let request = Request::builder()
                    .uri("/performance")
                    .method("GET")
                    .body(Body::empty())
                    .unwrap();
                
                app.oneshot(request).await.unwrap()
            });
            handles.push(handle);
        }
        
        // 等待所有请求完成
        for handle in handles {
            let response = handle.await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }
    }
}