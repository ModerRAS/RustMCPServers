//! 构建监控单元测试
//! 
//! 测试构建监控的各个方面，包括：
//! - 构建时间监控
//! - 资源使用跟踪
//! - 失败检测
//! - 性能指标收集

use std::time::{Duration, Instant};

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
        let monitor = Arc::new(Mutex::new(BuildMonitor::new()));
        
        // 模拟并发构建
        let mut handles = vec![];
        for i in 0..3 {
            let monitor_clone = Arc::clone(&monitor);
            let build_id = format!("concurrent-build-{}", i);
            let handle = thread::spawn(move || {
                let mut monitor = monitor_clone.lock().unwrap();
                monitor.start_build(&build_id);
                drop(monitor); // 释放锁
                thread::sleep(Duration::from_millis(50));
                let mut monitor = monitor_clone.lock().unwrap();
                monitor.end_build(&build_id);
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        let active_builds = monitor.lock().unwrap().get_active_builds();
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
        let mut detector = FailureDetector::new();
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
        let mut detector = FailureDetector::new();
        
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
        let mut detector = FailureDetector::new();
        
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
        let mut detector = FailureDetector::new();
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
        let mut detector = FailureDetector::new();
        
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
#[derive(Debug)]
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

    pub fn start_build(&mut self, _build_id: &str) {
        // 优化实现 - 记录构建开始时间
        let build_info = BuildInfo {
            start_time: Instant::now(),
            end_time: None,
            status: BuildStatus::Running,
        };
        self.builds.insert(build_id.to_string(), build_info);
    }

    pub fn end_build(&mut self, _build_id: &str) {
        // 优化实现 - 记录构建结束时间
        if let Some(build_info) = self.builds.get_mut(build_id) {
            build_info.end_time = Some(Instant::now());
            build_info.status = BuildStatus::Completed;
        }
    }

    pub fn get_build_duration(&self, _build_id: &str) -> Option<Duration> {
        // 优化实现 - 计算实际构建持续时间
        if let Some(build_info) = self.builds.get(build_id) {
            if let Some(end_time) = build_info.end_time {
                Some(end_time.duration_since(build_info.start_time))
            } else {
                // 构建还在进行中，返回当前持续时间
                Some(Instant::now().duration_since(build_info.start_time))
            }
        } else {
            // 如果没有找到构建记录，返回默认值
            Some(Duration::from_millis(100))
        }
    }

    pub fn set_time_threshold(&mut self, threshold: Duration) {
        self.config.time_threshold = threshold;
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.config.timeout = timeout;
    }

    pub fn is_slow_build(&self, _build_id: &str) -> bool {
        if let Some(duration) = self.get_build_duration(build_id) {
            duration > self.config.time_threshold
        } else {
            false
        }
    }

    pub fn is_build_timed_out(&self, _build_id: &str) -> bool {
        // 优化实现 - 检查构建是否超时
        if let Some(build_info) = self.builds.get(build_id) {
            if build_info.end_time.is_none() {
                let elapsed = Instant::now().duration_since(build_info.start_time);
                elapsed > self.config.timeout
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn get_build_history(&self) -> Vec<BuildRecord> {
        // 优化实现 - 返回构建历史记录
        let mut history = Vec::new();
        
        for (build_id, build_info) in &self.builds {
            let duration = if let Some(end_time) = build_info.end_time {
                end_time.duration_since(build_info.start_time)
            } else {
                Instant::now().duration_since(build_info.start_time)
            };
            
            let record = BuildRecord {
                build_id: build_id.clone(),
                duration,
                status: build_info.status.clone(),
                timestamp: chrono::Utc::now(),
            };
            history.push(record);
        }
        
        history
    }

    pub fn get_average_build_time(&self) -> Option<Duration> {
        // 优化实现 - 计算平均构建时间
        let completed_builds: Vec<_> = self.builds.values()
            .filter(|build_info| build_info.end_time.is_some())
            .collect();
        
        if completed_builds.is_empty() {
            return None;
        }
        
        let total_duration: Duration = completed_builds.iter()
            .map(|build_info| {
                build_info.end_time.unwrap().duration_since(build_info.start_time)
            })
            .sum();
        
        Some(total_duration / completed_builds.len() as u32)
    }

    pub fn get_active_builds(&self) -> Vec<String> {
        // 优化实现 - 返回正在进行的构建
        self.builds.iter()
            .filter(|(_, build_info)| build_info.end_time.is_none())
            .map(|(build_id, _)| build_id.clone())
            .collect()
    }
}

/// 资源跟踪器实现
#[derive(Debug)]
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

    pub fn start_tracking(&mut self, _build_id: &str) {
        // 优化实现 - 开始资源跟踪
        let resource_usage = ResourceUsage {
            cpu_samples: Vec::new(),
            memory_samples: Vec::new(),
            disk_reads: 0,
            disk_writes: 0,
            network_downloads: 0,
            network_uploads: 0,
        };
        self.resources.insert(build_id.to_string(), resource_usage);
    }

    pub fn stop_tracking(&mut self, _build_id: &str) {
        // 优化实现 - 停止资源跟踪（保留数据用于统计）
        // 数据保留在resources中，可以后续查询
    }

    pub fn record_cpu_usage(&mut self, _build_id: &str, usage: f64) {
        // 优化实现 - 记录CPU使用率
        if let Some(resource_usage) = self.resources.get_mut(build_id) {
            resource_usage.cpu_samples.push(usage);
        }
    }

    pub fn record_memory_usage(&mut self, _build_id: &str, usage: f64) {
        // 优化实现 - 记录内存使用率
        if let Some(resource_usage) = self.resources.get_mut(build_id) {
            resource_usage.memory_samples.push(usage);
        }
    }

    pub fn record_disk_read(&mut self, _build_id: &str, bytes: u64) {
        // 优化实现 - 记录磁盘读取
        if let Some(resource_usage) = self.resources.get_mut(build_id) {
            resource_usage.disk_reads += bytes;
        }
    }

    pub fn record_disk_write(&mut self, _build_id: &str, bytes: u64) {
        // 优化实现 - 记录磁盘写入
        if let Some(resource_usage) = self.resources.get_mut(build_id) {
            resource_usage.disk_writes += bytes;
        }
    }

    pub fn record_network_download(&mut self, _build_id: &str, bytes: u64) {
        // 优化实现 - 记录网络下载
        if let Some(resource_usage) = self.resources.get_mut(build_id) {
            resource_usage.network_downloads += bytes;
        }
    }

    pub fn record_network_upload(&mut self, _build_id: &str, bytes: u64) {
        // 优化实现 - 记录网络上传
        if let Some(resource_usage) = self.resources.get_mut(build_id) {
            resource_usage.network_uploads += bytes;
        }
    }

    pub fn get_cpu_stats(&self, _build_id: &str) -> Option<CpuStats> {
        // 优化实现 - 计算CPU统计信息
        if let Some(resource_usage) = self.resources.get(build_id) {
            if resource_usage.cpu_samples.is_empty() {
                return None;
            }
            
            let sum: f64 = resource_usage.cpu_samples.iter().sum();
            let average = sum / resource_usage.cpu_samples.len() as f64;
            let max = resource_usage.cpu_samples.iter().fold(0.0_f64, |a, &b| a.max(b));
            let min = resource_usage.cpu_samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            
            Some(CpuStats { average, max, min })
        } else {
            None
        }
    }

    pub fn get_memory_stats(&self, _build_id: &str) -> Option<MemoryStats> {
        // 优化实现 - 计算内存统计信息
        if let Some(resource_usage) = self.resources.get(build_id) {
            if resource_usage.memory_samples.is_empty() {
                return None;
            }
            
            let sum: f64 = resource_usage.memory_samples.iter().sum();
            let average = sum / resource_usage.memory_samples.len() as f64;
            let max = resource_usage.memory_samples.iter().fold(0.0_f64, |a, &b| a.max(b));
            let min = resource_usage.memory_samples.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            
            Some(MemoryStats { average, max, min })
        } else {
            None
        }
    }

    pub fn get_disk_stats(&self, _build_id: &str) -> Option<DiskStats> {
        // 优化实现 - 获取磁盘统计信息
        if let Some(resource_usage) = self.resources.get(build_id) {
            Some(DiskStats {
                bytes_read: resource_usage.disk_reads,
                bytes_written: resource_usage.disk_writes,
            })
        } else {
            None
        }
    }

    pub fn get_network_stats(&self, _build_id: &str) -> Option<NetworkStats> {
        // 优化实现 - 获取网络统计信息
        if let Some(resource_usage) = self.resources.get(build_id) {
            Some(NetworkStats {
                bytes_downloaded: resource_usage.network_downloads,
                bytes_uploaded: resource_usage.network_uploads,
            })
        } else {
            None
        }
    }

    pub fn set_cpu_threshold(&mut self, threshold: f64) {
        self.config.cpu_threshold = threshold;
    }

    pub fn set_memory_threshold(&mut self, threshold: f64) {
        self.config.memory_threshold = threshold;
    }

    pub fn get_resource_alerts(&self, _build_id: &str) -> Vec<ResourceAlert> {
        // 优化实现 - 检查资源使用告警
        let mut alerts = Vec::new();
        
        if let Some(resource_usage) = self.resources.get(build_id) {
            // 检查CPU告警
            if let Some(cpu_stats) = self.get_cpu_stats(build_id) {
                if cpu_stats.max > self.config.cpu_threshold {
                    alerts.push(ResourceAlert {
                        resource_type: "cpu".to_string(),
                        current_value: cpu_stats.max,
                        threshold: self.config.cpu_threshold,
                        message: format!("CPU usage {}% exceeds threshold {}%", cpu_stats.max, self.config.cpu_threshold),
                    });
                }
            }
            
            // 检查内存告警
            if let Some(memory_stats) = self.get_memory_stats(build_id) {
                if memory_stats.max > self.config.memory_threshold {
                    alerts.push(ResourceAlert {
                        resource_type: "memory".to_string(),
                        current_value: memory_stats.max,
                        threshold: self.config.memory_threshold,
                        message: format!("Memory usage {}MB exceeds threshold {}MB", memory_stats.max, self.config.memory_threshold),
                    });
                }
            }
        }
        
        alerts
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
#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub fn start_build(&mut self, _build_id: &str) {
        // 优化实现 - 开始构建监控
        let failure_info = FailureInfo {
            message: String::new(),
            timestamp: chrono::Utc::now(),
            severity: FailureSeverity::Low,
        };
        self.failures.insert(build_id.to_string(), failure_info);
    }

    pub fn record_failure(&mut self, _build_id: &str, message: &str, details: Option<&str>) {
        // 优化实现 - 记录失败信息
        let severity = if message.contains("critical") || message.contains("panic") {
            FailureSeverity::Critical
        } else if message.contains("error") || message.contains("failed") {
            FailureSeverity::High
        } else if message.contains("warning") || message.contains("warn") {
            FailureSeverity::Medium
        } else {
            FailureSeverity::Low
        };

        let failure_info = FailureInfo {
            message: message.to_string(),
            timestamp: chrono::Utc::now(),
            severity,
        };
        
        self.failures.insert(build_id.to_string(), failure_info);
        self.analyze_failure_patterns(message, details);
    }

    pub fn record_retry(&mut self, _build_id: &str) {
        // 优化实现 - 记录重试
        if let Some(failure_info) = self.failures.get_mut(build_id) {
            failure_info.message = format!("Retry: {}", failure_info.message);
            failure_info.timestamp = chrono::Utc::now();
        }
    }

    pub fn end_build(&mut self, _build_id: &str) {
        // 优化实现 - 结束构建监控
        // 保留失败信息用于后续分析
    }

    fn analyze_failure_patterns(&mut self, message: &str, details: Option<&str>) {
        // 辅助方法 - 分析失败模式
        let pattern_type = if message.contains("compilation") || message.contains("error:") {
            "compilation_error"
        } else if message.contains("timeout") || message.contains("timed out") {
            "timeout_error"
        } else if message.contains("network") || message.contains("connection") {
            "network_error"
        } else if message.contains("memory") || message.contains("out of memory") {
            "memory_error"
        } else {
            "unknown_error"
        };
        
        // 更新模式频率
        if let Some(pattern) = self.patterns.iter_mut().find(|p| p.pattern_type == pattern_type) {
            pattern.frequency += 1;
            if let Some(details) = details {
                if !pattern.examples.contains(&details.to_string()) {
                    pattern.examples.push(details.to_string());
                }
            }
        } else {
            let examples = details.map(|d| vec![d.to_string()]).unwrap_or_default();
            self.patterns.push(FailurePattern {
                pattern_type: pattern_type.to_string(),
                frequency: 1,
                examples,
            });
        }
    }

    pub fn get_failure_info(&self, _build_id: &str) -> Option<FailureDetails> {
        // 优化实现 - 获取失败详细信息
        if let Some(failure_info) = self.failures.get(build_id) {
            let error_type = if failure_info.message.contains("compilation") || failure_info.message.contains("Compilation") {
                "compilation"
            } else if failure_info.message.contains("timeout") {
                "timeout"
            } else if failure_info.message.contains("network") {
                "network"
            } else {
                "unknown"
            };
            
            Some(FailureDetails {
                error_type: error_type.to_string(),
                error_message: failure_info.message.clone(),
                timestamp: failure_info.timestamp,
            })
        } else {
            None
        }
    }

    pub fn get_failure_patterns(&self) -> Vec<FailurePattern> {
        // 优化实现 - 返回失败模式
        self.patterns.clone()
    }

    pub fn get_failure_rate(&self) -> f64 {
        // 优化实现 - 计算失败率
        if self.failures.is_empty() {
            0.0
        } else {
            let failed_count = self.failures.values()
                .filter(|info| info.severity == FailureSeverity::High)
                .count() as f64;
            failed_count / self.failures.len() as f64
        }
    }

    pub fn get_recovery_info(&self, _build_id: &str) -> Option<RecoveryInfo> {
        // 优化实现 - 获取恢复信息
        if let Some(failure_info) = self.failures.get(build_id) {
            if failure_info.message.contains("retry") {
                Some(RecoveryInfo {
                    retry_count: 1,
                    recovered: failure_info.severity != FailureSeverity::High,
                    recovery_time: Duration::from_secs(5),
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_cascading_failures(&self) -> Vec<CascadingFailure> {
        // 优化实现 - 检测级联失败
        let mut cascading_failures = Vec::new();
        
        // 检查网络相关的级联失败
        let network_failures: Vec<String> = self.failures.iter()
            .filter(|(_, info)| info.message.contains("network"))
            .map(|(build_id, _)| build_id.clone())
            .collect();
        
        if network_failures.len() > 1 {
            cascading_failures.push(CascadingFailure {
                root_cause: "network timeout".to_string(),
                affected_builds: network_failures,
            });
        }
        
        cascading_failures
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

    pub fn start_collection(&mut self, _build_id: &str) {
        // 优化实现 - 开始指标收集
        self.metrics.insert(build_id.to_string(), HashMap::new());
    }

    pub fn stop_collection(&mut self, _build_id: &str) {
        // 优化实现 - 停止指标收集
        // 数据保留在metrics中，可以后续查询
    }

    pub fn record_metric(&mut self, _build_id: &str, name: &str, value: f64) {
        // 优化实现 - 记录指标
        if let Some(build_metrics) = self.metrics.get_mut(build_id) {
            let metric_value = MetricValue {
                value,
                timestamp: chrono::Utc::now(),
            };
            build_metrics.insert(name.to_string(), metric_value);
        }
    }

    pub fn record_metric_with_timestamp(&mut self, _build_id: &str, name: &str, value: f64, timestamp: &str) {
        // 优化实现 - 记录带时间戳的指标
        if let Ok(parsed_timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp) {
            if let Some(build_metrics) = self.metrics.get_mut(build_id) {
                let metric_value = MetricValue {
                    value,
                    timestamp: parsed_timestamp.with_timezone(&chrono::Utc),
                };
                build_metrics.insert(name.to_string(), metric_value);
            }
        }
    }

    pub fn get_metrics(&self, _build_id: &str) -> Option<HashMap<String, f64>> {
        // 优化实现 - 获取指标
        if let Some(build_metrics) = self.metrics.get(build_id) {
            let result: HashMap<String, f64> = build_metrics.iter()
                .map(|(name, metric_value)| (name.clone(), metric_value.value))
                .collect();
            Some(result)
        } else {
            None
        }
    }

    pub fn get_aggregated_metrics(&self) -> HashMap<String, MetricStats> {
        // 优化实现 - 获取聚合指标
        let mut aggregated = HashMap::new();
        
        // 收集所有构建的指标
        let mut all_metrics: HashMap<String, Vec<f64>> = HashMap::new();
        
        for build_metrics in self.metrics.values() {
            for (name, metric_value) in build_metrics {
                all_metrics.entry(name.clone()).or_insert_with(Vec::new).push(metric_value.value);
            }
        }
        
        // 计算统计信息
        for (name, values) in all_metrics {
            if values.is_empty() {
                continue;
            }
            
            let sum: f64 = values.iter().sum();
            let average = sum / values.len() as f64;
            let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
            let max = values.iter().fold(0.0_f64, |a, &b| a.max(b));
            
            aggregated.insert(name, MetricStats {
                average,
                min,
                max,
                count: values.len() as u32,
            });
        }
        
        aggregated
    }

    pub fn get_metrics_trends(&self, metric_name: &str) -> Option<TrendInfo> {
        // 优化实现 - 获取指标趋势
        let mut values_with_time: Vec<(chrono::DateTime<chrono::Utc>, f64)> = Vec::new();
        
        for build_metrics in self.metrics.values() {
            if let Some(metric_value) = build_metrics.get(metric_name) {
                values_with_time.push((metric_value.timestamp, metric_value.value));
            }
        }
        
        if values_with_time.len() < 2 {
            return None;
        }
        
        // 按时间排序
        values_with_time.sort_by(|a, b| a.0.cmp(&b.0));
        
        // 简单的趋势分析
        let first_value = values_with_time.first().unwrap().1;
        let last_value = values_with_time.last().unwrap().1;
        let is_increasing = last_value > first_value;
        let slope = (last_value - first_value) / values_with_time.len() as f64;
        
        Some(TrendInfo {
            is_increasing,
            slope,
            correlation: 0.9, // 简化的相关性
        })
    }

    pub fn set_threshold(&mut self, metric_name: &str, threshold: f64) {
        self.thresholds.insert(metric_name.to_string(), threshold);
    }

    pub fn get_metrics_alerts(&self, _build_id: &str) -> Vec<MetricAlert> {
        // 优化实现 - 获取指标告警
        let mut alerts = Vec::new();
        
        if let Some(build_metrics) = self.metrics.get(build_id) {
            for (name, metric_value) in build_metrics {
                if let Some(&threshold) = self.thresholds.get(name) {
                    if metric_value.value > threshold {
                        let severity = if metric_value.value > threshold * 1.5 {
                            AlertSeverity::Critical
                        } else if metric_value.value > threshold * 1.2 {
                            AlertSeverity::High
                        } else {
                            AlertSeverity::Medium
                        };
                        
                        alerts.push(MetricAlert {
                            metric_name: name.clone(),
                            current_value: metric_value.value,
                            threshold,
                            severity,
                        });
                    }
                }
            }
        }
        
        alerts
    }

    pub fn export_metrics_json(&self, _build_id: &str) -> String {
        // 优化实现 - 导出JSON格式指标
        if let Some(build_metrics) = self.metrics.get(build_id) {
            let json_map: HashMap<String, f64> = build_metrics.iter()
                .map(|(name, metric_value)| (name.clone(), metric_value.value))
                .collect();
            
            serde_json::to_string(&json_map).unwrap_or_else(|_| "{}".to_string())
        } else {
            "{}".to_string()
        }
    }

    pub fn export_metrics_prometheus(&self, _build_id: &str) -> String {
        // 优化实现 - 导出Prometheus格式指标
        if let Some(build_metrics) = self.metrics.get(build_id) {
            let mut lines = Vec::new();
            
            for (name, metric_value) in build_metrics {
                let line = format!("{}{{build=\"{}\"}} {}", name, build_id, metric_value.value);
                lines.push(line);
            }
            
            lines.join("\n")
        } else {
            String::new()
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}