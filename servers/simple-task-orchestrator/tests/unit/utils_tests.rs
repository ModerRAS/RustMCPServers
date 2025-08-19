#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_new() {
        let limiter = RateLimiter::new(5);
        assert_eq!(limiter.requests_per_minute, 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_within_limit() {
        let limiter = RateLimiter::new(5);
        let key = "test_user";

        // 前5个请求应该成功
        for i in 0..5 {
            let result = limiter.check_rate_limit(key).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_exceed_limit() {
        let limiter = RateLimiter::new(3);
        let key = "test_user";

        // 前3个请求应该成功
        for i in 0..3 {
            let result = limiter.check_rate_limit(key).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }

        // 第4个请求应该被限制
        let result = limiter.check_rate_limit(key).await;
        assert!(result.is_err(), "4th request should be rate limited");
        assert_eq!(result.unwrap_err(), "Rate limit exceeded");
    }

    #[tokio::test]
    async fn test_rate_limiter_reset_after_window() {
        let limiter = RateLimiter::new(1);
        let key = "test_user";

        // 第一个请求应该成功
        let result1 = limiter.check_rate_limit(key).await;
        assert!(result1.is_ok());

        // 第二个请求应该被限制
        let result2 = limiter.check_rate_limit(key).await;
        assert!(result2.is_err());

        // 等待时间窗口过期
        sleep(Duration::from_millis(100)).await;

        // 第三个请求应该成功（因为时间窗口已重置）
        let result3 = limiter.check_rate_limit(key).await;
        assert!(result3.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_multiple_users() {
        let limiter = RateLimiter::new(2);
        let user1 = "user1";
        let user2 = "user2";

        // 用户1的请求
        assert!(limiter.check_rate_limit(user1).await.is_ok());
        assert!(limiter.check_rate_limit(user1).await.is_ok());
        assert!(limiter.check_rate_limit(user1).await.is_err());

        // 用户2的请求应该不受影响
        assert!(limiter.check_rate_limit(user2).await.is_ok());
        assert!(limiter.check_rate_limit(user2).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_get_remaining_requests() {
        let limiter = RateLimiter::new(5);
        let key = "test_user";

        // 初始应该有5个剩余请求
        assert_eq!(limiter.get_remaining_requests(key).await, 5);

        // 使用2个请求
        assert!(limiter.check_rate_limit(key).await.is_ok());
        assert!(limiter.check_rate_limit(key).await.is_ok());

        // 应该有3个剩余请求
        assert_eq!(limiter.get_remaining_requests(key).await, 3);
    }

    #[tokio::test]
    async fn test_rate_limiter_clone() {
        let limiter1 = RateLimiter::new(3);
        let limiter2 = limiter1.clone();

        let key = "test_user";

        // 使用limiter1进行2个请求
        assert!(limiter1.check_rate_limit(key).await.is_ok());
        assert!(limiter1.check_rate_limit(key).await.is_ok());

        // limiter2应该反映相同的状态
        assert_eq!(limiter2.get_remaining_requests(key).await, 1);
        assert!(limiter2.check_rate_limit(key).await.is_ok());
        assert!(limiter2.check_rate_limit(key).await.is_err());
    }

    #[tokio::test]
    async fn test_health_checker_new() {
        let checker = HealthChecker::new();
        assert_eq!(checker.checks.len(), 0);
    }

    #[tokio::test]
    async fn test_health_checker_add_check() {
        let mut checker = HealthChecker::new();
        
        let mock_check = Box::new(MockHealthCheck::new(true));
        checker.add_check("test_check".to_string(), mock_check);

        assert_eq!(checker.checks.len(), 1);
        assert!(checker.checks.contains_key("test_check"));
    }

    #[tokio::test]
    async fn test_health_checker_check_all() {
        let mut checker = HealthChecker::new();
        
        checker.add_check("check1".to_string(), Box::new(MockHealthCheck::new(true)));
        checker.add_check("check2".to_string(), Box::new(MockHealthCheck::new(false)));

        let results = checker.check_all().await;
        
        assert_eq!(results.len(), 2);
        assert_eq!(results.get("check1"), Some(&true));
        assert_eq!(results.get("check2"), Some(&false));
    }

    #[tokio::test]
    async fn test_health_checker_is_healthy() {
        let mut checker = HealthChecker::new();
        
        // 空检查器应该是健康的
        assert!(checker.is_healthy().await);

        // 添加健康的检查
        checker.add_check("check1".to_string(), Box::new(MockHealthCheck::new(true)));
        assert!(checker.is_healthy().await);

        // 添加不健康的检查
        checker.add_check("check2".to_string(), Box::new(MockHealthCheck::new(false)));
        assert!(!checker.is_healthy().await);
    }

    #[tokio::test]
    async fn test_memory_health_check_new() {
        let check = MemoryHealthCheck::new(1024);
        assert_eq!(check.threshold_mb, 1024);
    }

    #[tokio::test]
    async fn test_memory_health_check_check() {
        let check = MemoryHealthCheck::new(1024);
        // 简化实现总是返回true
        assert!(check.check().await);
    }

    #[tokio::test]
    async fn test_metrics_collector_new() {
        let collector = MetricsCollector::new();
        let metrics = collector.metrics.read().await;
        assert_eq!(metrics.len(), 0);
    }

    #[tokio::test]
    async fn test_metrics_collector_increment() {
        let collector = MetricsCollector::new();
        
        collector.increment("test_counter").await;
        collector.increment("test_counter").await;

        assert_eq!(collector.get("test_counter").await, Some(2));
    }

    #[tokio::test]
    async fn test_metrics_collector_set() {
        let collector = MetricsCollector::new();
        
        collector.set("test_gauge", 42).await;
        assert_eq!(collector.get("test_gauge").await, Some(42));

        collector.set("test_gauge", 100).await;
        assert_eq!(collector.get("test_gauge").await, Some(100));
    }

    #[tokio::test]
    async fn test_metrics_collector_get_nonexistent() {
        let collector = MetricsCollector::new();
        assert_eq!(collector.get("nonexistent").await, None);
    }

    #[tokio::test]
    async fn test_metrics_collector_get_all() {
        let collector = MetricsCollector::new();
        
        collector.increment("counter1").await;
        collector.set("gauge1", 10).await;
        collector.increment("counter1").await;
        collector.set("gauge2", 20).await;

        let all_metrics = collector.get_all().await;
        
        assert_eq!(all_metrics.len(), 3);
        assert_eq!(all_metrics.get("counter1"), Some(&2));
        assert_eq!(all_metrics.get("gauge1"), Some(&10));
        assert_eq!(all_metrics.get("gauge2"), Some(&20));
    }

    #[tokio::test]
    async fn test_metrics_collector_clone() {
        let collector1 = MetricsCollector::new();
        
        collector1.increment("test_metric").await;
        
        let collector2 = collector1.clone();
        collector2.increment("test_metric").await;

        assert_eq!(collector1.get("test_metric").await, Some(2));
        assert_eq!(collector2.get("test_metric").await, Some(2));
    }

    #[test]
    fn test_utils_format_duration() {
        assert_eq!(utils::format_duration(0), "0s");
        assert_eq!(utils::format_duration(30), "30s");
        assert_eq!(utils::format_duration(60), "1m 0s");
        assert_eq!(utils::format_duration(90), "1m 30s");
        assert_eq!(utils::format_duration(3600), "1h 0m 0s");
        assert_eq!(utils::format_duration(3661), "1h 1m 1s");
        assert_eq!(utils::format_duration(7320), "2h 2m 0s");
    }

    #[test]
    fn test_utils_format_file_size() {
        assert_eq!(utils::format_file_size(0), "0 B");
        assert_eq!(utils::format_file_size(1), "1 B");
        assert_eq!(utils::format_file_size(1023), "1023 B");
        assert_eq!(utils::format_file_size(1024), "1.0 KB");
        assert_eq!(utils::format_file_size(1536), "1.5 KB");
        assert_eq!(utils::format_file_size(1048576), "1.0 MB");
        assert_eq!(utils::format_file_size(1073741824), "1.0 GB");
        assert_eq!(utils::format_file_size(1099511627776), "1.0 TB");
    }

    #[test]
    fn test_utils_timestamp() {
        let ts = utils::timestamp();
        assert!(ts.timestamp() > 0);
    }

    #[test]
    fn test_utils_random_string() {
        let s1 = utils::random_string(10);
        let s2 = utils::random_string(10);
        
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2); // 很大概率不相等
        
        // 测试不同长度
        assert_eq!(utils::random_string(5).len(), 5);
        assert_eq!(utils::random_string(20).len(), 20);
    }

    #[test]
    fn test_utils_is_valid_email() {
        // 有效邮箱
        assert!(utils::is_valid_email("test@example.com"));
        assert!(utils::is_valid_email("user.name@domain.org"));
        assert!(utils::is_valid_email("test+tag@example.co.uk"));
        
        // 无效邮箱
        assert!(!utils::is_valid_email("invalid-email"));
        assert!(!utils::is_valid_email("@example.com"));
        assert!(!utils::is_valid_email("test@"));
        assert!(!utils::is_valid_email("test.com"));
        assert!(!utils::is_valid_email(""));
    }

    #[test]
    fn test_utils_is_valid_url() {
        // 有效URL
        assert!(utils::is_valid_url("https://example.com"));
        assert!(utils::is_valid_url("http://example.com"));
        assert!(utils::is_valid_url("https://sub.domain.com/path"));
        assert!(utils::is_valid_url("http://localhost:8080"));
        
        // 无效URL
        assert!(!utils::is_valid_url("ftp://example.com"));
        assert!(!utils::is_valid_url("example.com"));
        assert!(!utils::is_valid_url("https://"));
        assert!(!utils::is_valid_url(""));
    }

    #[test]
    fn test_utils_truncate_string() {
        // 不需要截断
        assert_eq!(utils::truncate_string("Hello", 10), "Hello");
        assert_eq!(utils::truncate_string("Hello", 5), "Hello");
        
        // 需要截断
        assert_eq!(utils::truncate_string("Hello World", 5), "He...");
        assert_eq!(utils::truncate_string("Long text here", 8), "Long te...");
        
        // 边界情况
        assert_eq!(utils::truncate_string("", 10), "");
        assert_eq!(utils::truncate_string("Hi", 1), "...");
        assert_eq!(utils::truncate_string("Hello", 3), "...");
    }

    // Mock HealthCheck for testing
    struct MockHealthCheck {
        healthy: bool,
    }

    impl MockHealthCheck {
        fn new(healthy: bool) -> Self {
            Self { healthy }
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for MockHealthCheck {
        async fn check(&self) -> bool {
            self.healthy
        }
    }
}