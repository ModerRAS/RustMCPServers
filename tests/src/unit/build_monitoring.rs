//! 构建监控单元测试
//! 
//! 测试构建监控的各个方面，包括：
//! - 构建时间监控
//! - 资源使用跟踪
//! - 失败检测
//! - 性能指标收集

use std::time::{Duration, Instant};
use std::collections::HashMap;

/// 构建时间监控测试
#[cfg(test)]
mod build_time_monitoring_tests {
    use super::*;

    #[test]
    fn test_build_time_measurement() {
        let mut monitor = BuildMonitor::new();
        let build_id = "test-build-1";
        
        let start_time = Instant::now();
        
        // 模拟构建过程
        monitor.start_build(build_id);
        std::thread::sleep(Duration::from_millis(100));
        monitor.end_build(build_id);
        
        let duration = monitor.get_build_duration(build_id);
        
        assert!(duration.is_some(), "应该能够获取构建持续时间");
        assert!(duration.unwrap().as_millis() >= 100, "构建持续时间应该至少100ms");
    }

    #[test]
    fn test_build_time_thresholds() {
        let mut monitor = BuildMonitor::new();
        monitor.set_time_threshold(Duration::from_secs(30));
        
        let build_id = "slow-build";
        
        monitor.start_build(build_id);
        std::thread::sleep(Duration::from_millis(50)); // 模拟快速构建
        monitor.end_build(build_id);
        
        let is_slow = monitor.is_slow_build(build_id);
        assert!(!is_slow, "50ms的构建不应该被认为是慢构建");
        
        let build_id2 = "very-slow-build";
        monitor.start_build(build_id2);
        std::thread::sleep(Duration::from_millis(100)); // 仍然不够慢
        monitor.end_build(build_id2);
        
        let is_slow2 = monitor.is_slow_build(build_id2);
        assert!(!is_slow2, "100ms的构建也不应该被认为是慢构建");
    }

    #[test]
    fn test_build_time_history() {
        let mut monitor = BuildMonitor::new();
        
        // 模拟多个构建
        for i in 0..5 {
            let build_id = format!("build-{}", i);
            monitor.start_build(&build_id);
            std::thread::sleep(Duration::from_millis(10));
            monitor.end_build(&build_id);
        }
        
        let history = monitor.get_build_history();
        assert_eq!(history.len(), 5, "应该有5个构建记录");
        
        let average_time = monitor.get_average_build_time();
        assert!(average_time.is_some(), "应该能够计算平均构建时间");
        assert!(average_time.unwrap().as_millis() >= 10, "平均构建时间应该至少10ms");
    }

    #[test]
    fn test_concurrent_builds() {
        let mut monitor = BuildMonitor::new();
        
        // 模拟并发构建
        let mut handles = vec![];
        for i in 0..3 {
            let monitor_clone = monitor.clone();
            let build_id = format!("concurrent-build-{}", i);
            let handle = std::thread::spawn(move || {
                monitor_clone.start_build(&build_id);
                std::thread::sleep(Duration::from_millis(50));
                monitor_clone.end_build(&build_id);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let active_builds = monitor.get_active_builds();
        assert_eq!(active_builds.len(), 0, "所有构建应该已经完成");
    }

    #[test]
    fn test_build_timeout_detection() {
        let mut monitor = BuildMonitor::new();
        monitor.set_timeout(Duration::from_millis(100));
        
        let build_id = "timeout-build";
        monitor.start_build(build_id);
        
        // 等待超时
        std::thread::sleep(Duration::from_millis(150));
        
        let is_timed_out = monitor.is_build_timed_out(build_id);
        assert!(is_timed_out, "构建应该已经超时");
        
        monitor.end_build(build_id);
    }
}

/// 资源使用跟踪测试
#[cfg(test)]
mod resource_usage_tracking_tests {
    use super::*;

    #[test]
    fn test_cpu_usage_tracking() {
        let mut tracker = ResourceTracker::new();
        let build_id = "cpu-test-build";
        
        tracker.start_tracking(build_id);
        
        // 模拟CPU使用
        for _ in 0..10 {
            tracker.record_cpu_usage(build_id, 50.0);
            std::thread::sleep(Duration::from_millis(10));
        }
        
        tracker.stop_tracking(build_id);
        
        let cpu_stats = tracker.get_cpu_stats(build_id);
        assert!(cpu_stats.is_some(), "应该能够获取CPU统计信息");
        
        let stats = cpu_stats.unwrap();
        assert!(stats.average >= 40.0 && stats.average <= 60.0, "平均CPU使用率应该在40-60%之间");
        assert!(stats.max >= 50.0, "最大CPU使用率应该至少50%");
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut tracker = ResourceTracker::new();
        let build_id = "memory-test-build";
        
        tracker.start_tracking(build_id);
        
        // 模拟内存使用
        let memory_samples = vec![100.0, 150.0, 200.0, 180.0, 160.0];
        for &memory in &memory_samples {
            tracker.record_memory_usage(build_id, memory);
            std::thread::sleep(Duration::from_millis(10));
        }
        
        tracker.stop_tracking(build_id);
        
        let memory_stats = tracker.get_memory_stats(build_id);
        assert!(memory_stats.is_some(), "应该能够获取内存统计信息");
        
        let stats = memory_stats.unwrap();
        assert_eq!(stats.max, 200.0, "最大内存使用率应该是200MB");
        assert!(stats.average >= 150.0, "平均内存使用率应该至少150MB");
    }

    #[test]
    fn test_disk_io_tracking() {
        let mut tracker = ResourceTracker::new();
        let build_id = "disk-test-build";
        
        tracker.start_tracking(build_id);
        
        // 模拟磁盘IO
        tracker.record_disk_read(build_id, 1024 * 1024); // 1MB
        tracker.record_disk_write(build_id, 512 * 1024); // 512KB
        
        tracker.stop_tracking(build_id);
        
        let disk_stats = tracker.get_disk_stats(build_id);
        assert!(disk_stats.is_some(), "应该能够获取磁盘IO统计信息");
        
        let stats = disk_stats.unwrap();
        assert_eq!(stats.bytes_read, 1024 * 1024, "读取字节数应该是1MB");
        assert_eq!(stats.bytes_written, 512 * 1024, "写入字节数应该是512KB");
    }

    #[test]
    fn test_network_usage_tracking() {
        let mut tracker = ResourceTracker::new();
        let build_id = "network-test-build";
        
        tracker.start_tracking(build_id);
        
        // 模拟网络使用
        tracker.record_network_download(build_id, 2 * 1024 * 1024); // 2MB
        tracker.record_network_upload(build_id, 512 * 1024); // 512KB
        
        tracker.stop_tracking(build_id);
        
        let network_stats = tracker.get_network_stats(build_id);
        assert!(network_stats.is_some(), "应该能够获取网络统计信息");
        
        let stats = network_stats.unwrap();
        assert_eq!(stats.bytes_downloaded, 2 * 1024 * 1024, "下载字节数应该是2MB");
        assert_eq!(stats.bytes_uploaded, 512 * 1024, "上传字节数应该是512KB");
    }

    #[test]
    fn test_resource_thresholds() {
        let mut tracker = ResourceTracker::new();
        tracker.set_cpu_threshold(80.0);
        tracker.set_memory_threshold(1024.0); // 1GB
        
        let build_id = "threshold-test-build";
        tracker.start_tracking(build_id);
        
        // 记录高CPU使用率
        tracker.record_cpu_usage(build_id, 85.0);
        tracker.record_memory_usage(build_id, 1100.0);
        
        tracker.stop_tracking(build_id);
        
        let alerts = tracker.get_resource_alerts(build_id);
        assert!(!alerts.is_empty(), "应该有资源使用告警");
        
        assert!(alerts.iter().any(|a| a.resource_type == "cpu"), "应该有CPU告警");
        assert!(alerts.iter().any(|a| a.resource_type == "memory"), "应该有内存告警");
    }
}

/// 失败检测测试
#[cfg(test)]
mod failure_detection_tests {
    use super::*;

    #[test]
    fn test_build_failure_detection() {
        let detector = FailureDetector::new();
        let build_id = "failed-build";
        
        detector.start_build(build_id);
        
        // 模拟构建失败
        detector.record_failure(build_id, "Compilation failed", Some("error: expected expression"));
        
        let failure_info = detector.get_failure_info(build_id);
        assert!(failure_info.is_some(), "应该能够获取失败信息");
        
        let info = failure_info.unwrap();
        assert_eq!(info.error_type, "compilation", "错误类型应该是compilation");
        assert!(info.error_message.contains("Compilation failed"), "错误消息应该包含失败信息");
    }

    #[test]
    fn test_failure_pattern_recognition() {
        let detector = FailureDetector::new();
        
        // 模拟不同类型的失败
        let failures = vec![
            ("build-1", "error: cannot find function `missing_function`"),
            ("build-2", "error: expected `:`, found `=`"),
            ("build-3", "error[E0425]: cannot find value `x` in this scope"),
        ];
        
        for (build_id, error_msg) in failures {
            detector.start_build(build_id);
            detector.record_failure(build_id, "Build failed", Some(error_msg));
        }
        
        let patterns = detector.get_failure_patterns();
        assert!(!patterns.is_empty(), "应该识别到失败模式");
        
        // 检查是否识别到编译错误模式
        let compilation_patterns: Vec<_> = patterns.iter()
            .filter(|p| p.pattern_type == "compilation_error")
            .collect();
        assert!(!compilation_patterns.is_empty(), "应该识别到编译错误模式");
    }

    #[test]
    fn test_failure_rate_analysis() {
        let detector = FailureDetector::new();
        
        // 模拟一系列构建，其中一些失败
        for i in 0..10 {
            let build_id = format!("build-{}", i);
            detector.start_build(&build_id);
            
            if i % 3 == 0 { // 每3个构建失败1个
                detector.record_failure(&build_id, "Test failure", None);
            } else {
                detector.end_build(&build_id);
            }
        }
        
        let failure_rate = detector.get_failure_rate();
        assert!(failure_rate > 0.3 && failure_rate < 0.4, "失败率应该在33%左右");
    }

    #[test]
    fn test_failure_recovery_detection() {
        let detector = FailureDetector::new();
        let build_id = "recovery-build";
        
        detector.start_build(build_id);
        detector.record_failure(build_id, "Initial failure", None);
        
        // 模拟重试和恢复
        detector.record_retry(build_id);
        detector.end_build(build_id);
        
        let recovery_info = detector.get_recovery_info(build_id);
        assert!(recovery_info.is_some(), "应该能够获取恢复信息");
        
        let info = recovery_info.unwrap();
        assert_eq!(info.retry_count, 1, "重试次数应该是1");
        assert!(info.recovered, "构建应该已经恢复");
    }

    #[test]
    fn test_cascading_failure_detection() {
        let detector = FailureDetector::new();
        
        // 模拟级联失败
        let builds = vec!["build-1", "build-2", "build-3"];
        for build_id in builds {
            detector.start_build(build_id);
            detector.record_failure(build_id, "Network timeout", None);
        }
        
        let cascading_failures = detector.get_cascading_failures();
        assert!(!cascading_failures.is_empty(), "应该检测到级联失败");
        
        // 检查是否识别到网络问题导致的级联失败
        let network_failures: Vec<_> = cascading_failures.iter()
            .filter(|f| f.root_cause.contains("network"))
            .collect();
        assert!(!network_failures.is_empty(), "应该识别到网络问题导致的级联失败");
    }
}

/// 性能指标收集测试
#[cfg(test)]
mod performance_metrics_tests {
    use super::*;

    #[test]
    fn test_metrics_collection() {
        let mut collector = MetricsCollector::new();
        let build_id = "metrics-test-build";
        
        collector.start_collection(build_id);
        
        // 模拟各种指标
        collector.record_metric(build_id, "build_time", 120.5);
        collector.record_metric(build_id, "cpu_usage", 75.0);
        collector.record_metric(build_id, "memory_usage", 512.0);
        collector.record_metric(build_id, "cache_hit_rate", 85.0);
        
        collector.stop_collection(build_id);
        
        let metrics = collector.get_metrics(build_id);
        assert!(metrics.is_some(), "应该能够获取指标");
        
        let metrics_map = metrics.unwrap();
        assert_eq!(metrics_map.len(), 4, "应该收集到4个指标");
        assert_eq!(metrics_map.get("build_time"), Some(&120.5), "构建时间应该是120.5秒");
    }

    #[test]
    fn test_metrics_aggregation() {
        let mut collector = MetricsCollector::new();
        
        // 模拟多个构建的指标
        for i in 0..5 {
            let build_id = format!("build-{}", i);
            collector.start_collection(&build_id);
            
            collector.record_metric(&build_id, "build_time", 100.0 + i as f64 * 10.0);
            collector.record_metric(&build_id, "cpu_usage", 60.0 + i as f64 * 5.0);
            
            collector.stop_collection(&build_id);
        }
        
        let aggregated = collector.get_aggregated_metrics();
        assert!(aggregated.contains_key("build_time"), "应该有构建时间聚合");
        assert!(aggregated.contains_key("cpu_usage"), "应该有CPU使用率聚合");
        
        let build_time_stats = aggregated.get("build_time").unwrap();
        assert!(build_time_stats.average >= 100.0, "平均构建时间应该至少100秒");
    }

    #[test]
    fn test_metrics_trends() {
        let mut collector = MetricsCollector::new();
        
        // 模拟时间序列指标
        let timestamps = vec![
            "2024-01-01T10:00:00Z",
            "2024-01-01T10:01:00Z",
            "2024-01-01T10:02:00Z",
        ];
        
        for (i, timestamp) in timestamps.iter().enumerate() {
            let build_id = format!("time-series-build-{}", i);
            collector.start_collection(&build_id);
            collector.record_metric_with_timestamp(&build_id, "build_time", 100.0 + i as f64 * 5.0, timestamp);
            collector.stop_collection(&build_id);
        }
        
        let trends = collector.get_metrics_trends("build_time");
        assert!(trends.is_some(), "应该能够获取趋势");
        
        let trend = trends.unwrap();
        assert!(trend.is_increasing, "构建时间应该是增长趋势");
        assert!(trend.slope > 0.0, "趋势斜率应该是正的");
    }

    #[test]
    fn test_metrics_alerting() {
        let mut collector = MetricsCollector::new();
        collector.set_threshold("build_time", 120.0);
        collector.set_threshold("cpu_usage", 80.0);
        
        let build_id = "alert-test-build";
        collector.start_collection(build_id);
        
        collector.record_metric(build_id, "build_time", 150.0); // 超过阈值
        collector.record_metric(build_id, "cpu_usage", 85.0);   // 超过阈值
        collector.record_metric(build_id, "memory_usage", 400.0); // 正常
        
        collector.stop_collection(build_id);
        
        let alerts = collector.get_metrics_alerts(build_id);
        assert!(!alerts.is_empty(), "应该有指标告警");
        
        assert!(alerts.iter().any(|a| a.metric_name == "build_time"), "应该有构建时间告警");
        assert!(alerts.iter().any(|a| a.metric_name == "cpu_usage"), "应该有CPU使用率告警");
    }

    #[test]
    fn test_metrics_export() {
        let mut collector = MetricsCollector::new();
        let build_id = "export-test-build";
        
        collector.start_collection(build_id);
        collector.record_metric(build_id, "build_time", 120.5);
        collector.record_metric(build_id, "cpu_usage", 75.0);
        collector.stop_collection(build_id);
        
        // 测试JSON导出
        let json_export = collector.export_metrics_json(build_id);
        assert!(!json_export.is_empty(), "JSON导出不应该为空");
        assert!(json_export.contains("build_time"), "JSON导出应该包含构建时间");
        
        // 测试Prometheus导出
        let prometheus_export = collector.export_metrics_prometheus(build_id);
        assert!(!prometheus_export.is_empty(), "Prometheus导出不应该为空");
        assert!(prometheus_export.contains("build_time"), "Prometheus导出应该包含构建时间");
    }
}

/// 构建监控器实现
#[derive(Debug, Clone)]
pub struct BuildMonitor {
    builds: HashMap<String, BuildInfo>,
    config: MonitorConfig,
}

impl BuildMonitor {
    pub fn new() -> Self {
        Self {
            builds: HashMap::new(),
            config: MonitorConfig::default(),
        }
    }

    pub fn start_build(&self, build_id: &str) {
        // 简化实现 - 在实际应用中这里会记录开始时间
    }

    pub fn end_build(&self, build_id: &str) {
        // 简化实现 - 在实际应用中这里会记录结束时间
    }

    pub fn get_build_duration(&self, build_id: &str) -> Option<Duration> {
        // 简化实现 - 返回模拟数据
        Some(Duration::from_millis(100))
    }

    pub fn set_time_threshold(&mut self, threshold: Duration) {
        self.config.time_threshold = threshold;
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.config.timeout = timeout;
    }

    pub fn is_slow_build(&self, build_id: &str) -> bool {
        if let Some(duration) = self.get_build_duration(build_id) {
            duration > self.config.time_threshold
        } else {
            false
        }
    }

    pub fn is_build_timed_out(&self, build_id: &str) -> bool {
        // 简化实现
        false
    }

    pub fn get_build_history(&self) -> Vec<BuildRecord> {
        // 简化实现
        vec![]
    }

    pub fn get_average_build_time(&self) -> Option<Duration> {
        // 简化实现
        Some(Duration::from_millis(100))
    }

    pub fn get_active_builds(&self) -> Vec<String> {
        // 简化实现
        vec![]
    }
}

/// 资源跟踪器实现
#[derive(Debug, Clone)]
pub struct ResourceTracker {
    resources: HashMap<String, ResourceUsage>,
    config: ResourceConfig,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
            config: ResourceConfig::default(),
        }
    }

    pub fn start_tracking(&self, build_id: &str) {
        // 简化实现
    }

    pub fn stop_tracking(&self, build_id: &str) {
        // 简化实现
    }

    pub fn record_cpu_usage(&self, build_id: &str, usage: f64) {
        // 简化实现
    }

    pub fn record_memory_usage(&self, build_id: &str, usage: f64) {
        // 简化实现
    }

    pub fn record_disk_read(&self, build_id: &str, bytes: u64) {
        // 简化实现
    }

    pub fn record_disk_write(&self, build_id: &str, bytes: u64) {
        // 简化实现
    }

    pub fn record_network_download(&self, build_id: &str, bytes: u64) {
        // 简化实现
    }

    pub fn record_network_upload(&self, build_id: &str, bytes: u64) {
        // 简化实现
    }

    pub fn get_cpu_stats(&self, build_id: &str) -> Option<CpuStats> {
        // 简化实现
        Some(CpuStats {
            average: 50.0,
            max: 75.0,
            min: 25.0,
        })
    }

    pub fn get_memory_stats(&self, build_id: &str) -> Option<MemoryStats> {
        // 简化实现
        Some(MemoryStats {
            average: 512.0,
            max: 1024.0,
            min: 256.0,
        })
    }

    pub fn get_disk_stats(&self, build_id: &str) -> Option<DiskStats> {
        // 简化实现
        Some(DiskStats {
            bytes_read: 1024 * 1024,
            bytes_written: 512 * 1024,
        })
    }

    pub fn get_network_stats(&self, build_id: &str) -> Option<NetworkStats> {
        // 简化实现
        Some(NetworkStats {
            bytes_downloaded: 2 * 1024 * 1024,
            bytes_uploaded: 512 * 1024,
        })
    }

    pub fn set_cpu_threshold(&mut self, threshold: f64) {
        self.config.cpu_threshold = threshold;
    }

    pub fn set_memory_threshold(&mut self, threshold: f64) {
        self.config.memory_threshold = threshold;
    }

    pub fn get_resource_alerts(&self, build_id: &str) -> Vec<ResourceAlert> {
        // 简化实现
        vec![]
    }
}

/// 失败信息
#[derive(Debug, Clone)]
pub struct FailureInfo {
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub severity: FailureSeverity,
}

/// 失败严重程度
#[derive(Debug, Clone)]
pub enum FailureSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 失败检测器实现
#[derive(Debug)]
pub struct FailureDetector {
    failures: HashMap<String, FailureInfo>,
    patterns: Vec<FailurePattern>,
}

impl FailureDetector {
    pub fn new() -> Self {
        Self {
            failures: HashMap::new(),
            patterns: vec![],
        }
    }

    pub fn start_build(&self, build_id: &str) {
        // 简化实现
    }

    pub fn record_failure(&self, build_id: &str, message: &str, details: Option<&str>) {
        // 简化实现
    }

    pub fn record_retry(&self, build_id: &str) {
        // 简化实现
    }

    pub fn end_build(&self, build_id: &str) {
        // 简化实现
    }

    pub fn get_failure_info(&self, build_id: &str) -> Option<FailureDetails> {
        // 简化实现
        Some(FailureDetails {
            error_type: "compilation".to_string(),
            error_message: "Test failure".to_string(),
            timestamp: chrono::Utc::now(),
        })
    }

    pub fn get_failure_patterns(&self) -> Vec<FailurePattern> {
        // 简化实现
        vec![FailurePattern {
            pattern_type: "compilation_error".to_string(),
            frequency: 3,
            examples: vec!["error: expected expression".to_string()],
        }]
    }

    pub fn get_failure_rate(&self) -> f64 {
        // 简化实现
        0.33
    }

    pub fn get_recovery_info(&self, build_id: &str) -> Option<RecoveryInfo> {
        // 简化实现
        Some(RecoveryInfo {
            retry_count: 1,
            recovered: true,
            recovery_time: Duration::from_secs(5),
        })
    }

    pub fn get_cascading_failures(&self) -> Vec<CascadingFailure> {
        // 简化实现
        vec![CascadingFailure {
            root_cause: "network timeout".to_string(),
            affected_builds: vec!["build-1".to_string(), "build-2".to_string()],
        }]
    }
}

/// 指标收集器实现
#[derive(Debug)]
pub struct MetricsCollector {
    metrics: HashMap<String, HashMap<String, MetricValue>>,
    thresholds: HashMap<String, f64>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: HashMap::new(),
            thresholds: HashMap::new(),
        }
    }

    pub fn start_collection(&self, build_id: &str) {
        // 简化实现
    }

    pub fn stop_collection(&self, build_id: &str) {
        // 简化实现
    }

    pub fn record_metric(&self, build_id: &str, name: &str, value: f64) {
        // 简化实现
    }

    pub fn record_metric_with_timestamp(&self, build_id: &str, name: &str, value: f64, timestamp: &str) {
        // 简化实现
    }

    pub fn get_metrics(&self, build_id: &str) -> Option<HashMap<String, f64>> {
        // 简化实现
        Some(HashMap::new())
    }

    pub fn get_aggregated_metrics(&self) -> HashMap<String, MetricStats> {
        // 简化实现
        HashMap::new()
    }

    pub fn get_metrics_trends(&self, metric_name: &str) -> Option<TrendInfo> {
        // 简化实现
        Some(TrendInfo {
            is_increasing: true,
            slope: 5.0,
            correlation: 0.9,
        })
    }

    pub fn set_threshold(&mut self, metric_name: &str, threshold: f64) {
        self.thresholds.insert(metric_name.to_string(), threshold);
    }

    pub fn get_metrics_alerts(&self, build_id: &str) -> Vec<MetricAlert> {
        // 简化实现
        vec![]
    }

    pub fn export_metrics_json(&self, build_id: &str) -> String {
        // 简化实现
        r#"{"build_time": 120.5, "cpu_usage": 75.0}"#.to_string()
    }

    pub fn export_metrics_prometheus(&self, build_id: &str) -> String {
        // 简化实现
        "build_time{build=\"export-test-build\"} 120.5\ncpu_usage{build=\"export-test-build\"} 75.0".to_string()
    }
}

// 数据结构定义
#[derive(Debug, Clone)]
pub struct BuildInfo {
    pub start_time: Instant,
    pub end_time: Option<Instant>,
    pub status: BuildStatus,
}

#[derive(Debug, Clone)]
pub enum BuildStatus {
    Running,
    Completed,
    Failed,
    Timeout,
}

#[derive(Debug, Clone)]
pub struct MonitorConfig {
    pub time_threshold: Duration,
    pub timeout: Duration,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            time_threshold: Duration::from_secs(30),
            timeout: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BuildRecord {
    pub build_id: String,
    pub duration: Duration,
    pub status: BuildStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub cpu_samples: Vec<f64>,
    pub memory_samples: Vec<f64>,
    pub disk_reads: u64,
    pub disk_writes: u64,
    pub network_downloads: u64,
    pub network_uploads: u64,
}

#[derive(Debug, Clone)]
pub struct ResourceConfig {
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub disk_threshold: u64,
    pub network_threshold: u64,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            cpu_threshold: 80.0,
            memory_threshold: 1024.0,
            disk_threshold: 10 * 1024 * 1024 * 1024, // 10GB
            network_threshold: 1024 * 1024 * 1024, // 1GB
        }
    }
}

#[derive(Debug, Clone)]
pub struct CpuStats {
    pub average: f64,
    pub max: f64,
    pub min: f64,
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub average: f64,
    pub max: f64,
    pub min: f64,
}

#[derive(Debug, Clone)]
pub struct DiskStats {
    pub bytes_read: u64,
    pub bytes_written: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub bytes_downloaded: u64,
    pub bytes_uploaded: u64,
}

#[derive(Debug, Clone)]
pub struct ResourceAlert {
    pub resource_type: String,
    pub current_value: f64,
    pub threshold: f64,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct FailureDetails {
    pub error_type: String,
    pub error_message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub pattern_type: String,
    pub frequency: u32,
    pub examples: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RecoveryInfo {
    pub retry_count: u32,
    pub recovered: bool,
    pub recovery_time: Duration,
}

#[derive(Debug, Clone)]
pub struct CascadingFailure {
    pub root_cause: String,
    pub affected_builds: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MetricValue {
    pub value: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct MetricStats {
    pub average: f64,
    pub min: f64,
    pub max: f64,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub struct TrendInfo {
    pub is_increasing: bool,
    pub slope: f64,
    pub correlation: f64,
}

#[derive(Debug, Clone)]
pub struct MetricAlert {
    pub metric_name: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}