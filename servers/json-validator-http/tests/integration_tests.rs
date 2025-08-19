//! 集成测试
//! 
//! 这个模块包含了HTTP协议JSON验证MCP服务器的集成测试，
//! 覆盖了主要功能和性能测试。

use anyhow::Result;
use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::sleep;

use json_validator_http::{
    app::create_app,
    config::ServerConfig,
    models::{ValidationOptions, ValidationResult},
    services::JsonValidatorService,
};

/// 测试助手函数
async fn create_test_server() -> Result<axum::Router> {
    let config = ServerConfig::default();
    create_app(config).await
}

/// 测试助手：发送HTTP请求
async fn send_request(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    headers: &[(&str, &str)],
) -> Result<axum::response::Response> {
    use axum::body::Body;
    use http::Request;
    use tower::ServiceExt;

    let mut request_builder = Request::builder()
        .method(method)
        .uri(path);

    // 添加头部
    for (key, value) in headers {
        request_builder = request_builder.header(*key, *value);
    }

    let request = if let Some(body) = body {
        request_builder
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body)?))
            .unwrap()
    } else {
        request_builder.body(Body::empty()).unwrap()
    };

    Ok(app.clone().oneshot(request).await?)
}

#[tokio::test]
async fn test_health_check() {
    let app = create_test_server().await.unwrap();

    let response = send_request(&app, "GET", "/health", None, &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["status"], "healthy");
    assert!(response_json["timestamp"].is_string());
}

#[tokio::test]
async fn test_server_info() {
    let app = create_test_server().await.unwrap();

    let response = send_request(&app, "GET", "/info", None, &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["name"], "JSON Validator HTTP MCP Server");
    assert!(response_json["version"].is_string());
    assert!(response_json["features"]["performance_optimizations"].as_bool().unwrap());
}

#[tokio::test]
async fn test_performance_stats() {
    let app = create_test_server().await.unwrap();

    let response = send_request(&app, "GET", "/performance", None, &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    // 检查性能统计结构
    assert!(response_json["server_stats"].is_object());
    assert!(response_json["response_time_stats"].is_object());
    assert!(response_json["system_stats"].is_object());
    assert!(response_json["validation_stats"].is_object());
    assert!(response_json["performance_config"].is_object());
}

#[tokio::test]
async fn test_basic_json_validation() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {
                "name": "test",
                "age": 25,
                "email": "test@example.com"
            },
            "options": {}
        },
        "id": 1
    });

    let response = send_request(&app, "POST", "/rpc", Some(request_body), &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"]["valid"].as_bool().unwrap());
}

#[tokio::test]
async fn test_schema_validation() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json_with_schema",
        "params": {
            "json_data": {
                "name": "test",
                "age": 25
            },
            "schema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string"},
                    "age": {"type": "number"}
                },
                "required": ["name", "age"]
            },
            "options": {}
        },
        "id": 1
    });

    let response = send_request(&app, "POST", "/rpc", Some(request_body), &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"]["valid"].as_bool().unwrap());
}

#[tokio::test]
async fn test_batch_validation() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json_batch",
        "params": {
            "items": [
                {
                    "id": "1",
                    "json_data": {"name": "test1", "age": 25},
                    "schema": null
                },
                {
                    "id": "2",
                    "json_data": {"name": "test2", "age": 30},
                    "schema": null
                }
            ],
            "options": {}
        },
        "id": 1
    });

    let response = send_request(&app, "POST", "/rpc", Some(request_body), &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"].is_array());
    assert_eq!(response_json["result"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_invalid_json() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {
                "name": "test",
                "age": "not_a_number"
            },
            "options": {
                "strict_mode": true
            }
        },
        "id": 1
    });

    let response = send_request(&app, "POST", "/rpc", Some(request_body), &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(!response_json["result"]["valid"].as_bool().unwrap());
    assert!(response_json["result"]["errors"].is_array());
}

#[tokio::test]
async fn test_api_key_authentication() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {"name": "test", "age": 25},
            "options": {}
        },
        "id": 1
    });

    // 测试没有API密钥的情况
    let response = send_request(&app, "POST", "/rpc", Some(request_body.clone()), &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::UNAUTHORIZED);

    // 测试有效的API密钥
    let response = send_request(&app, "POST", "/rpc", Some(request_body), &[("Authorization", "Bearer admin_key")])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);
}

#[tokio::test]
async fn test_rate_limiting() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {"name": "test", "age": 25},
            "options": {}
        },
        "id": 1
    });

    let mut success_count = 0;
    let mut rate_limited_count = 0;

    // 发送多个请求以测试限流
    for i in 0..20 {
        let response = send_request(&app, "POST", "/rpc", Some(request_body.clone()), &[("Authorization", "Bearer user_key")])
            .await
            .unwrap();

        if response.status() == http::StatusCode::OK {
            success_count += 1;
        } else if response.status() == http::StatusCode::TOO_MANY_REQUESTS {
            rate_limited_count += 1;
        }

        // 短暂延迟
        sleep(Duration::from_millis(10)).await;
    }

    // 验证限流是否生效
    assert!(success_count > 0);
    assert!(rate_limited_count > 0);
}

#[tokio::test]
async fn test_concurrent_requests() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {"name": "test", "age": 25},
            "options": {}
        },
        "id": 1
    });

    // 创建多个并发请求
    let mut handles = vec![];
    for i in 0..50 {
        let app = app.clone();
        let request_body = request_body.clone();
        
        let handle = tokio::spawn(async move {
            send_request(&app, "POST", "/rpc", Some(request_body), &[("Authorization", "Bearer admin_key")])
                .await
                .unwrap()
        });
        handles.push(handle);
    }

    // 等待所有请求完成
    let mut success_count = 0;
    for handle in handles {
        let response = handle.await.unwrap();
        if response.status() == http::StatusCode::OK {
            success_count += 1;
        }
    }

    // 验证所有请求都成功
    assert_eq!(success_count, 50);
}

#[tokio::test]
async fn test_large_json_processing() {
    let app = create_test_server().await.unwrap();

    // 创建大型JSON数据
    let large_json: Value = (0..1000).map(|i| {
        json!({
            "id": i,
            "name": format!("item_{}", i),
            "description": "This is a long description for item {}".repeat(10),
            "metadata": {
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z",
                "tags": ["tag1", "tag2", "tag3"],
                "attributes": {
                    "size": i * 100,
                    "weight": i * 0.5,
                    "active": true
                }
            }
        })
    }).collect();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": large_json,
            "options": {}
        },
        "id": 1
    });

    let response = send_request(&app, "POST", "/rpc", Some(request_body), &[("Authorization", "Bearer admin_key")])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["result"]["valid"].as_bool().unwrap());
}

#[tokio::test]
async fn test_caching_functionality() {
    let app = create_test_server().await.unwrap();

    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {"name": "test", "age": 25},
            "options": {
                "cache_key": "test_cache_key"
            }
        },
        "id": 1
    });

    // 第一次请求
    let response1 = send_request(&app, "POST", "/rpc", Some(request_body.clone()), &[("Authorization", "Bearer admin_key")])
        .await
        .unwrap();

    assert_eq!(response1.status(), http::StatusCode::OK);

    let body1 = hyper::body::to_bytes(response1.into_body()).await.unwrap();
    let response_json1: Value = serde_json::from_slice(&body1).unwrap();

    // 第二次请求（应该命中缓存）
    let response2 = send_request(&app, "POST", "/rpc", Some(request_body), &[("Authorization", "Bearer admin_key")])
        .await
        .unwrap();

    assert_eq!(response2.status(), http::StatusCode::OK);

    let body2 = hyper::body::to_bytes(response2.into_body()).await.unwrap();
    let response_json2: Value = serde_json::from_slice(&body2).unwrap();

    // 验证结果相同
    assert_eq!(response_json1["result"]["valid"], response_json2["result"]["valid"]);
    
    // 第二次请求应该有缓存命中
    assert!(response_json2["result"]["cache_hit"].as_bool().unwrap_or(false));
}

#[tokio::test]
async fn test_error_handling() {
    let app = create_test_server().await.unwrap();

    // 测试无效的JSON-RPC请求
    let invalid_request = json!({
        "jsonrpc": "2.0",
        "method": "nonexistent_method",
        "params": {},
        "id": 1
    });

    let response = send_request(&app, "POST", "/rpc", Some(invalid_request), &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let response_json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json["jsonrpc"], "2.0");
    assert_eq!(response_json["id"], 1);
    assert!(response_json["error"].is_object());
    assert_eq!(response_json["error"]["code"], -32601); // Method not found

    // 测试无效的JSON格式
    let response = send_request(&app, "POST", "/rpc", None, &[("content-type", "application/json")])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_metrics_endpoint() {
    let app = create_test_server().await.unwrap();

    // 发送一些请求以生成指标
    for i in 0..5 {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "validate_json",
            "params": {
                "json_data": {"name": "test", "age": 25},
                "options": {}
            },
            "id": i
        });

        let _response = send_request(&app, "POST", "/rpc", Some(request_body), &[("Authorization", "Bearer admin_key")])
            .await
            .unwrap();
    }

    // 获取指标
    let response = send_request(&app, "GET", "/metrics", None, &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let metrics_text = String::from_utf8(body.to_vec()).unwrap();

    // 验证指标格式
    assert!(metrics_text.contains("http_requests_total"));
    assert!(metrics_text.contains("http_request_duration_seconds"));
    assert!(metrics_text.contains("json_validator_validations_total"));
}

#[tokio::test]
async fn test_cors_functionality() {
    let app = create_test_server().await.unwrap();

    // 测试OPTIONS请求
    let response = send_request(&app, "OPTIONS", "/rpc", None, &[
        ("Origin", "http://localhost:3000"),
        ("Access-Control-Request-Method", "POST"),
        ("Access-Control-Request-Headers", "content-type"),
    ])
    .await
    .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    // 验证CORS头部
    assert_eq!(response.headers().get("Access-Control-Allow-Origin").unwrap(), "*");
    assert_eq!(response.headers().get("Access-Control-Allow-Methods").unwrap(), "POST");
}

#[tokio::test]
async fn test_service_initialization() {
    let config = ServerConfig::default();
    let service = JsonValidatorService::new(config).await.unwrap();

    // 测试基本功能
    let json_data = json!({"name": "test", "age": 25});
    let options = ValidationOptions::default();
    
    let result = service.validate_json(&json_data, &options).await.unwrap();
    assert!(result.valid);

    // 测试统计功能
    let stats = service.get_stats().await;
    assert_eq!(stats.requests_total, 1);
    assert_eq!(stats.validations_total, 1);
    assert_eq!(stats.requests_success, 1);
}

#[tokio::test]
async fn test_configuration_validation() {
    let mut config = ServerConfig::default();
    
    // 测试有效配置
    assert!(config.validate().is_ok());

    // 测试无效配置
    config.server.workers = 0;
    assert!(config.validate().is_err());

    // 测试生产环境安全检查
    let mut config = ServerConfig::default();
    config.deployment.environment = "production";
    config.security.jwt_secret = "your-secret-key-here-change-in-production";
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_performance_optimizations() {
    let app = create_test_server().await.unwrap();

    let start_time = std::time::Instant::now();
    
    // 发送多个请求以测试性能
    for i in 0..100 {
        let request_body = json!({
            "jsonrpc": "2.0",
            "method": "validate_json",
            "params": {
                "json_data": {"name": "test", "age": 25},
                "options": {}
            },
            "id": i
        });

        let response = send_request(&app, "POST", "/rpc", Some(request_body), &[("Authorization", "Bearer admin_key")])
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);
    }

    let duration = start_time.elapsed();
    
    // 验证性能目标（100个请求应该在5秒内完成）
    assert!(duration.as_secs() < 5);
    
    // 获取性能统计
    let response = send_request(&app, "GET", "/performance", None, &[])
        .await
        .unwrap();

    assert_eq!(response.status(), http::StatusCode::OK);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let performance_stats: Value = serde_json::from_slice(&body).unwrap();

    // 验证性能指标
    assert!(performance_stats["server_stats"]["total_requests"].as_u64().unwrap() >= 100);
    assert!(performance_stats["server_stats"]["successful_requests"].as_u64().unwrap() >= 100);
}