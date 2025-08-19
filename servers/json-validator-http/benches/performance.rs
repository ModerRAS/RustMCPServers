//! 性能基准测试
//! 
//! 这个模块包含了性能基准测试，用于验证优化效果和性能指标。

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;
use std::time::Duration;
use tokio::runtime::Runtime;

use json_validator_http::{
    app::create_app,
    config::ServerConfig,
    models::{ValidationOptions, ValidationResult},
    performance::{PerformanceConfig, PerformanceManager},
    services::JsonValidatorService,
};

/// 基准测试辅助函数
fn create_test_config() -> ServerConfig {
    let mut config = ServerConfig::default();
    
    // 启用性能优化
    config.performance.concurrency.max_concurrent_requests = 1000;
    config.performance.memory.enable_memory_pool = true;
    config.performance.basic.enable_compression = true;
    config.performance.basic.enable_caching = true;
    config.cache.enabled = true;
    
    config
}

/// 基准测试：基本JSON验证
fn bench_basic_json_validation(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let service = rt.block_on(async {
        JsonValidatorService::new(config).await.unwrap()
    });
    
    let json_data = json!({
        "name": "test",
        "age": 25,
        "email": "test@example.com"
    });
    
    let options = ValidationOptions::default();
    
    c.bench_function("basic_json_validation", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                service.validate_json(black_box(&json_data), black_box(&options)).await
            })
        });
    });
}

/// 基准测试：Schema验证
fn bench_schema_validation(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let service = rt.block_on(async {
        JsonValidatorService::new(config).await.unwrap()
    });
    
    let json_data = json!({
        "name": "test",
        "age": 25,
        "email": "test@example.com"
    });
    
    let schema = json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "number"},
            "email": {"type": "string", "format": "email"}
        },
        "required": ["name", "age", "email"]
    });
    
    let options = ValidationOptions::default();
    
    c.bench_function("schema_validation", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                service.validate_json_with_schema(
                    black_box(&json_data),
                    black_box(&schema),
                    black_box(&options)
                ).await
            })
        });
    });
}

/// 基准测试：批量验证
fn bench_batch_validation(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let service = rt.block_on(async {
        JsonValidatorService::new(config).await.unwrap()
    });
    
    let items = (0..100).map(|i| {
        use json_validator_http::models::BatchValidationItem;
        BatchValidationItem {
            id: i.to_string(),
            json_data: json!({
                "id": i,
                "name": format!("test_{}", i),
                "value": i * 10
            }),
            schema: None,
        }
    }).collect::<Vec<_>>();
    
    let options = ValidationOptions::default();
    
    c.bench_function("batch_validation_100_items", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                service.validate_json_batch(black_box(&items), black_box(&options)).await
            })
        });
    });
}

/// 基准测试：内存池性能
fn bench_memory_pool_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let performance_config = PerformanceConfig::default();
    let performance_manager = PerformanceManager::new(performance_config);
    
    c.bench_function("memory_pool_operations", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                let s = performance_manager.get_pooled_string(black_box(100)).await;
                performance_manager.return_pooled_string(s).await;
            })
        });
    });
}

/// 基准测试：HTTP请求处理
fn bench_http_request_handling(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let app = rt.block_on(async {
        create_app(config).await.unwrap()
    });
    
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {"name": "test", "age": 25},
            "options": {}
        },
        "id": 1
    });
    
    c.bench_function("http_request_handling", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                use axum::body::Body;
                use http::Request;
                use tower::ServiceExt;
                
                let request = Request::builder()
                    .uri("/rpc")
                    .method("POST")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap();
                
                app.clone().oneshot(request).await
            })
        });
    });
}

/// 基准测试：并发请求处理
fn bench_concurrent_requests(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let app = rt.block_on(async {
        create_app(config).await.unwrap()
    });
    
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "validate_json",
        "params": {
            "json_data": {"name": "test", "age": 25},
            "options": {}
        },
        "id": 1
    });
    
    let mut group = c.benchmark_group("concurrent_requests");
    
    for concurrent_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(BenchmarkId::new("concurrent", concurrent_count), concurrent_count, |b, &count| {
            b.to_async(&rt).iter(|| {
                rt.block_on(async {
                    use axum::body::Body;
                    use http::Request;
                    use tower::ServiceExt;
                    use futures::future::join_all;
                    
                    let requests: Vec<_> = (0..count).map(|i| {
                        let app = app.clone();
                        let request_body = request_body.clone();
                        
                        async move {
                            let request = Request::builder()
                                .uri("/rpc")
                                .method("POST")
                                .header("content-type", "application/json")
                                .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                                .unwrap();
                            
                            app.oneshot(request).await
                        }
                    }).collect();
                    
                    join_all(requests).await
                })
            });
        });
    }
    
    group.finish();
}

/// 基准测试：缓存性能
fn bench_cache_performance(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let service = rt.block_on(async {
        JsonValidatorService::new(config).await.unwrap()
    });
    
    let json_data = json!({
        "name": "test",
        "age": 25,
        "email": "test@example.com"
    });
    
    let options = ValidationOptions {
        cache_key: Some("test_cache_key".to_string()),
        ..Default::default()
    };
    
    // 预热缓存
    rt.block_on(async {
        service.validate_json(&json_data, &options).await.unwrap();
    });
    
    c.bench_function("cache_hit_performance", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                service.validate_json(black_box(&json_data), black_box(&options)).await
            })
        });
    });
}

/// 基准测试：大JSON处理
fn bench_large_json_processing(c: &mut Criterion) {
    let config = create_test_config();
    let rt = Runtime::new().unwrap();
    
    let service = rt.block_on(async {
        JsonValidatorService::new(config).await.unwrap()
    });
    
    // 创建大型JSON数据
    let large_json: serde_json::Value = (0..1000).map(|i| {
        json!({
            "id": i,
            "name": format!("item_{}", i),
            "description": "This is a long description for item {}".repeat(10),
            "metadata": {
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z",
                "tags": vec!["tag1", "tag2", "tag3"],
                "attributes": {
                    "size": i * 100,
                    "weight": i * 0.5,
                    "active": true
                }
            }
        })
    }).collect();
    
    let options = ValidationOptions::default();
    
    c.bench_function("large_json_processing", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                service.validate_json(black_box(&large_json), black_box(&options)).await
            })
        });
    });
}

/// 基准测试：性能统计收集
fn bench_performance_stats_collection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    let performance_config = PerformanceConfig::default();
    let performance_manager = PerformanceManager::new(performance_config);
    
    c.bench_function("performance_stats_collection", |b| {
        b.to_async(&rt).iter(|| {
            rt.block_on(async {
                performance_manager.record_response_time(black_box(Duration::from_millis(10))).await;
                performance_manager.record_success().await;
                performance_manager.get_stats().await
            })
        });
    });
}

criterion_group!(
    benches,
    bench_basic_json_validation,
    bench_schema_validation,
    bench_batch_validation,
    bench_memory_pool_performance,
    bench_http_request_handling,
    bench_concurrent_requests,
    bench_cache_performance,
    bench_large_json_processing,
    bench_performance_stats_collection
);
criterion_main!(benches);