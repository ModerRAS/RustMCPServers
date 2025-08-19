//! 性能优化模块单元测试
//! 
//! 这个模块包含了性能优化相关功能的单元测试。

use std::time::Duration;
use tokio::time::sleep;

use json_validator_http::performance::*;

#[tokio::test]
async fn test_memory_pool() {
    let pool = MemoryPool::new(10);

    // 测试获取字符串
    let s1 = pool.get_string(100).await;
    assert_eq!(s1.capacity(), 100);

    // 测试归还字符串
    pool.return_string(s1).await;

    // 检查使用量
    assert_eq!(pool.get_usage().await, 0);

    // 测试多次获取和归还
    for i in 0..20 {
        let s = pool.get_string(50).await;
        assert_eq!(s.capacity(), 50);
        pool.return_string(s).await;
    }

    // 使用量应该为0（因为所有字符串都被归还）
    assert_eq!(pool.get_usage().await, 0);
}

#[tokio::test]
async fn test_memory_pool_capacity_limit() {
    let pool = MemoryPool::new(5);

    let mut strings = vec![];

    // 获取超过池大小的字符串
    for i in 0..10 {
        let s = pool.get_string(100).await;
        strings.push(s);
    }

    // 使用量应该为10（因为池已满，新的字符串不会被池化）
    assert_eq!(pool.get_usage().await, 10);

    // 归还所有字符串
    for s in strings {
        pool.return_string(s).await;
    }

    // 使用量应该为5（池的大小限制）
    assert_eq!(pool.get_usage().await, 5);
}

#[tokio::test]
async fn test_request_limiter() {
    let limiter = RequestLimiter::new(2);

    // 获取许可
    let permit1 = limiter.acquire().await;
    let permit2 = limiter.acquire().await;

    // 检查并发数
    assert_eq!(limiter.get_current_concurrent().await, 2);
    assert_eq!(limiter.get_peak_concurrent().await, 2);

    // 释放一个许可
    drop(permit1);
    sleep(Duration::from_millis(10)).await;

    assert_eq!(limiter.get_current_concurrent().await, 1);
    assert_eq!(limiter.get_peak_concurrent().await, 2);
}

#[tokio::test]
async fn test_request_limiter_concurrent() {
    let limiter = RequestLimiter::new(3);

    let mut handles = vec![];

    // 并发获取许可
    for i in 0..10 {
        let limiter = limiter.clone();
        let handle = tokio::spawn(async move {
            let permit = limiter.acquire().await;
            let concurrent = permit.get_current_concurrent().await;
            sleep(Duration::from_millis(10)).await;
            concurrent
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let results: Vec<usize> = futures::future::join_all(handles)
        .await
        .into_iter()
        .map(Result::unwrap)
        .collect();

    // 验证并发限制
    for concurrent in results {
        assert!(concurrent <= 3);
    }

    // 峰值并发数应该达到限制
    assert_eq!(limiter.get_peak_concurrent().await, 3);
}

#[tokio::test]
async fn test_response_time_tracker() {
    let tracker = ResponseTimeTracker::new(10);

    // 记录响应时间
    tracker.record_response_time(Duration::from_millis(100)).await;
    tracker.record_response_time(Duration::from_millis(200)).await;
    tracker.record_response_time(Duration::from_millis(300)).await;

    // 检查平均响应时间
    let avg = tracker.get_avg_response_time().await;
    assert_eq!(avg, Duration::from_millis(200));

    // 检查P95响应时间
    let p95 = tracker.get_p95_response_time().await;
    assert_eq!(p95, Duration::from_millis(300));

    // 检查P99响应时间
    let p99 = tracker.get_p99_response_time().await;
    assert_eq!(p99, Duration::from_millis(300));
}

#[tokio::test]
async fn test_response_time_tracker_history_limit() {
    let tracker = ResponseTimeTracker::new(3);

    // 记录超过历史限制的响应时间
    for i in 0..10 {
        tracker.record_response_time(Duration::from_millis(i * 10)).await;
    }

    // 只应该保留最近的3个记录
    let avg = tracker.get_avg_response_time().await;
    assert_eq!(avg, Duration::from_millis(70)); // (70 + 80 + 90) / 3
}

#[tokio::test]
async fn test_performance_manager() {
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config);

    // 获取请求许可
    let permit = manager.acquire_request_permit().await.unwrap();
    assert_eq!(permit.get_current_concurrent().await, 1);

    // 记录响应时间
    manager.record_response_time(Duration::from_millis(50)).await;

    // 记录成功
    manager.record_success().await;

    // 获取统计信息
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.successful_requests, 1);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.avg_response_time_ms, 50.0);

    drop(permit);
}

#[tokio::test]
async fn test_performance_manager_concurrent() {
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config);

    let mut handles = vec![];

    // 并发执行操作
    for i in 0..20 {
        let manager = manager.clone();
        let handle = tokio::spawn(async move {
            let permit = manager.acquire_request_permit().await.unwrap();
            manager.record_response_time(Duration::from_millis(10)).await;
            manager.record_success().await;
            drop(permit);
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    futures::future::join_all(handles).await;

    // 检查统计信息
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 20);
    assert_eq!(stats.successful_requests, 20);
    assert_eq!(stats.failed_requests, 0);
    assert_eq!(stats.avg_response_time_ms, 10.0);
}

#[tokio::test]
async fn test_connection_pool() {
    let pool = ConnectionPool::new(3, || Ok("connection".to_string()));

    // 获取连接
    let conn1 = pool.get_connection().await.unwrap();
    let conn2 = pool.get_connection().await.unwrap();
    let conn3 = pool.get_connection().await.unwrap();

    // 检查池大小
    assert_eq!(pool.size().await, 0);

    // 归还连接
    pool.return_connection(conn1).await;
    pool.return_connection(conn2).await;
    pool.return_connection(conn3).await;

    // 检查池大小
    assert_eq!(pool.size().await, 3);

    // 再次获取连接
    let conn4 = pool.get_connection().await.unwrap();
    assert_eq!(conn4, "connection");
    assert_eq!(pool.size().await, 2);
}

#[tokio::test]
async fn test_performance_utils_fast_hash() {
    use json_validator_http::performance::utils;

    let value = serde_json::json!({"name": "test", "age": 25});
    let hash1 = utils::fast_hash(&value);
    let hash2 = utils::fast_hash(&value);

    // 相同的值应该产生相同的哈希
    assert_eq!(hash1, hash2);

    // 不同的值应该产生不同的哈希
    let different_value = serde_json::json!({"name": "different", "age": 30});
    let hash3 = utils::fast_hash(&different_value);
    assert_ne!(hash1, hash3);
}

#[tokio::test]
async fn test_performance_utils_json_serialization() {
    use json_validator_http::performance::utils;

    let value = serde_json::json!({"name": "test", "age": 25});
    
    // 测试序列化
    let serialized = utils::optimized_json_serialize(&value).unwrap();
    assert!(serialized.len() > 0);

    // 测试反序列化
    let deserialized = utils::optimized_json_deserialize(&serialized).unwrap();
    assert_eq!(value, deserialized);
}

#[tokio::test]
async fn test_performance_utils_batch_processing() {
    use json_validator_http::performance::utils;

    let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    let results = utils::optimized_batch_processing(
        items,
        |batch| batch.into_iter().map(|x| x * 2).collect(),
        3,
    ).await;

    assert_eq!(results, vec![2, 4, 6, 8, 10, 12, 14, 16, 18, 20]);
}

#[tokio::test]
async fn test_performance_manager_memory_tracking() {
    let config = PerformanceConfig {
        memory_limit_mb: 1,
        ..Default::default()
    };
    let manager = PerformanceManager::new(config);

    // 更新系统统计
    manager.update_system_stats().await;

    let stats = manager.get_stats().await;
    assert!(stats.memory_usage_mb >= 0.0);
}

#[tokio::test]
async fn test_performance_manager_cleanup() {
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config);

    // 执行一些操作
    let permit = manager.acquire_request_permit().await.unwrap();
    manager.record_response_time(Duration::from_millis(10)).await;
    drop(permit);

    // 清理资源
    manager.cleanup().await;

    // 验证清理后的状态
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 1);
}

#[tokio::test]
async fn test_memory_pool_string_reuse() {
    let pool = MemoryPool::new(5);

    // 获取字符串
    let s1 = pool.get_string(100).await;
    let original_capacity = s1.capacity();
    let ptr = s1.as_ptr();

    // 归还字符串
    pool.return_string(s1).await;

    // 再次获取字符串
    let s2 = pool.get_string(100).await;
    
    // 验证字符串被重用（容量应该相同）
    assert_eq!(s2.capacity(), original_capacity);
    
    // 归还字符串
    pool.return_string(s2).await;
}

#[tokio::test]
async fn test_request_limiter_permit_tracking() {
    let limiter = RequestLimiter::new(2);

    // 获取许可
    let permit1 = limiter.acquire().await;
    let permit2 = limiter.acquire().await;

    // 检查当前并发数
    assert_eq!(permit1.get_current_concurrent().await, 2);
    assert_eq!(permit2.get_current_concurrent().await, 2);

    // 释放一个许可
    drop(permit1);
    sleep(Duration::from_millis(10)).await;

    // 检查剩余许可的当前并发数
    assert_eq!(permit2.get_current_concurrent().await, 1);
}

#[tokio::test]
async fn test_performance_config_validation() {
    let config = PerformanceConfig::default();

    // 验证默认配置
    assert!(config.max_concurrent_requests > 0);
    assert!(config.connection_pool_size > 0);
    assert!(config.cache_size > 0);
    assert!(config.memory_limit_mb > 0);
    assert!(config.request_timeout > Duration::from_secs(0));
}

#[tokio::test]
async fn test_response_time_tracker_empty() {
    let tracker = ResponseTimeTracker::new(10);

    // 空跟踪器应该返回0
    let avg = tracker.get_avg_response_time().await;
    assert_eq!(avg, Duration::from_millis(0));

    let p95 = tracker.get_p95_response_time().await;
    assert_eq!(p95, Duration::from_millis(0));

    let p99 = tracker.get_p99_response_time().await;
    assert_eq!(p99, Duration::from_millis(0));
}

#[tokio::test]
async fn test_performance_utils_memory_efficient_conversion() {
    use json_validator_http::performance::utils;

    let original = vec![1, 2, 3, 4, 5];
    let converted: Vec<i32> = utils::memory_efficient_conversion(original).unwrap();
    
    assert_eq!(converted, vec![1, 2, 3, 4, 5]);
}

#[tokio::test]
async fn test_performance_manager_error_handling() {
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config);

    // 记录失败请求
    manager.record_failure().await;

    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 1);
    assert_eq!(stats.failed_requests, 1);
    assert_eq!(stats.successful_requests, 0);
}

#[tokio::test]
async fn test_performance_manager_concurrent_stats() {
    let config = PerformanceConfig::default();
    let manager = PerformanceManager::new(config);

    let mut handles = vec![];

    // 并发记录统计
    for i in 0..100 {
        let manager = manager.clone();
        let handle = tokio::spawn(async move {
            if i % 2 == 0 {
                manager.record_success().await;
            } else {
                manager.record_failure().await;
            }
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    futures::future::join_all(handles).await;

    // 检查统计信息
    let stats = manager.get_stats().await;
    assert_eq!(stats.total_requests, 100);
    assert_eq!(stats.successful_requests, 50);
    assert_eq!(stats.failed_requests, 50);
}