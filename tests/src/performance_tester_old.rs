use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

/// 工作流性能测试器
pub struct PerformanceTester {
    pub test_runs: usize,
    pub concurrent_runs: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PerformanceTestResult {
    pub test_name: String,
    pub total_duration_ms: u64,
    pub average_duration_ms: f64,
    pub min_duration_ms: u64,
    pub max_duration_ms: u64,
    pub throughput_rps: f64,
    pub success_rate: f64,
    pub memory_usage_mb: Option<f64>,
    pub cache_hit_rate: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CachePerformanceResult {
    pub cache_enabled: bool,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub cache_hit_rate: f64,
    pub time_saved_ms: u64,
}

impl PerformanceTester {
    /// 创建新的性能测试器
    pub fn new(test_runs: usize, concurrent_runs: usize) -> Self {
        Self {
            test_runs,
            concurrent_runs,
        }
    }

    /// 测试工作流执行性能
    pub async fn test_workflow_performance(&self, _workflow_path: &str) -> Result<PerformanceTestResult, Box<dyn std::error::Error>> {
        let mut durations = Vec::new();
        let mut successes = 0;
        
        let start_time = Instant::now();

        // 并发执行测试
        let mut tasks = Vec::new();
        for _ in 0..self.concurrent_runs {
            let workflow_path = workflow_path.to_string();
            tasks.push(tokio::spawn(async move {
                // 模拟执行并返回结果
                match simulate_workflow_execution_fixed(&workflow_path).await {
                    Ok(result) => result,
                    Err(_) => 0, // 失败时返回0
                }
            }));
        }

        // 等待所有任务完成
        for task in tasks {
            match task.await {
                Ok(Ok(duration)) => {
                    durations.push(duration);
                    successes += 1;
                }
                Ok(Err(_)) => {
                    // 失败的执行不计入持续时间
                }
                Err(_) => {
                    // 任务执行错误
                }
            }
        }

        let total_duration = start_time.elapsed().as_millis() as u64;
        
        Ok(PerformanceTestResult {
            test_name: format!("Workflow Performance Test - {}", workflow_path),
            total_duration_ms: total_duration,
            average_duration_ms: if !durations.is_empty() {
                durations.iter().sum::<u64>() as f64 / durations.len() as f64
            } else {
                0.0
            },
            min_duration_ms: durations.iter().min().copied().unwrap_or(0),
            max_duration_ms: durations.iter().max().copied().unwrap_or(0),
            throughput_rps: if total_duration > 0 {
                (self.test_runs * self.concurrent_runs) as f64 / (total_duration as f64 / 1000.0)
            } else {
                0.0
            },
            success_rate: if self.test_runs * self.concurrent_runs > 0 {
                (successes as f64 / (self.test_runs * self.concurrent_runs) as f64) * 100.0
            } else {
                0.0
            },
            memory_usage_mb: self.get_memory_usage(),
            cache_hit_rate: None, // 将在缓存测试中设置
        })
    }

    /// 测试缓存性能
    pub async fn test_cache_performance(&self, _workflow_path: &str, cache_enabled: bool) -> Result<CachePerformanceResult, Box<dyn std::error::Error>> {
        let mut cache_hits = 0;
        let mut cache_misses = 0;
        let mut total_time_with_cache = 0;
        let mut total_time_without_cache = 0;

        for i in 0..self.test_runs {
            let start = Instant::now();
            
            // 模拟缓存命中/未命中
            if cache_enabled && i > 0 {
                cache_hits += 1;
                // 缓存命中，执行时间较短
                sleep(Duration::from_millis(50)).await;
            } else {
                cache_misses += 1;
                // 缓存未命中，执行时间较长
                sleep(Duration::from_millis(200)).await;
            }
            
            let duration = start.elapsed().as_millis() as u64;
            if cache_enabled {
                total_time_with_cache += duration;
            } else {
                total_time_without_cache += duration;
            }
        }

        let cache_hit_rate = if cache_hits + cache_misses > 0 {
            (cache_hits as f64 / (cache_hits + cache_misses) as f64) * 100.0
        } else {
            0.0
        };

        let time_saved = if total_time_without_cache > 0 {
            total_time_without_cache - total_time_with_cache
        } else {
            0
        };

        Ok(CachePerformanceResult {
            cache_enabled,
            cache_hits,
            cache_misses,
            cache_hit_rate,
            time_saved_ms: time_saved,
        })
    }

    /// 测试并发性能
    pub async fn test_concurrent_performance(&self, _workflow_path: &str, max_concurrent: usize) -> Result<Vec<PerformanceTestResult>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for concurrent in 1..=max_concurrent {
            let tester = PerformanceTester::new(self.test_runs, concurrent);
            let result = tester.test_workflow_performance(workflow_path).await?;
            results.push(result);
        }

        Ok(results)
    }

    /// 模拟工作流执行
    async fn simulate_workflow_execution(&self, _workflow_path: &str) -> Result<u64, Box<dyn std::error::Error>> {
        simulate_workflow_execution_fixed(workflow_path).await
    }
}

/// 固定的工作流执行模拟函数
async fn simulate_workflow_execution_fixed(_workflow_path: &str) -> Result<u64, Box<dyn std::error::Error>> {
        let start_time = Instant::now();

        // 模拟不同的工作流步骤
        let steps = vec![
            ("checkout", 100),
            ("setup_rust", 200),
            ("cache_dependencies", 150),
            ("run_tests", 500),
            ("build", 300),
            ("upload_artifacts", 100),
        ];

        for (step, duration_ms) in steps {
            sleep(Duration::from_millis(duration_ms)).await;
            
            // 模拟10%的失败率
            if rand::random::<f64>() < 0.1 {
                return Err(format!("Step {} failed").into());
            }
        }

        Ok(start_time.elapsed().as_millis() as u64)
    }

    /// 获取内存使用情况（模拟）
    fn get_memory_usage(&self) -> Option<f64> {
        // 在实际环境中，这里会获取真实的内存使用情况
        Some(rand::random::<f64>() * 100.0 + 50.0)
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self, results: &[PerformanceTestResult]) -> String {
        let mut report = String::new();
        report.push_str("# Performance Test Report\n\n");
        report.push_str(&format!("Test Runs: {}\n", self.test_runs));
        report.push_str(&format!("Concurrent Runs: {}\n\n", self.concurrent_runs));

        for result in results {
            report.push_str(&format!("## {}\n", result.test_name));
            report.push_str(&format!("- Average Duration: {:.2}ms\n", result.average_duration_ms));
            report.push_str(&format!("- Min Duration: {}ms\n", result.min_duration_ms));
            report.push_str(&format!("- Max Duration: {}ms\n", result.max_duration_ms));
            report.push_str(&format!("- Throughput: {:.2} RPS\n", result.throughput_rps));
            report.push_str(&format!("- Success Rate: {:.2}%\n", result.success_rate));
            
            if let Some(memory) = result.memory_usage_mb {
                report.push_str(&format!("- Memory Usage: {:.2}MB\n", memory));
            }
            
            report.push('\n');
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_workflow_performance() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: echo "test"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();

        let tester = PerformanceTester::new(3, 1);
        let result = tester.test_workflow_performance(temp_file.path().to_str().unwrap()).await.unwrap();

        assert!(result.average_duration_ms > 0.0);
        assert!(result.success_rate > 0.0);
        assert_eq!(result.test_runs, 3);
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/cache@v3
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", workflow_content).unwrap();

        let tester = PerformanceTester::new(5, 1);
        let result = tester.test_cache_performance(temp_file.path().to_str().unwrap(), true).await.unwrap();

        assert!(result.cache_hit_rate >= 0.0 && result.cache_hit_rate <= 100.0);
        assert!(result.cache_hits + result.cache_misses > 0);
    }
}