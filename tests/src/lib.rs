pub mod simple_tests;
pub mod workflow_validator;
pub mod workflow_executor;
pub mod performance_tester;
pub mod security_tester;
pub mod coverage_reporter;
pub mod comprehensive_reporter;
pub mod unit;
pub mod integration;
pub mod e2e;

pub use simple_tests::*;
pub use workflow_validator::*;
pub use workflow_executor::*;
pub use performance_tester::*;
pub use security_tester::*;
pub use coverage_reporter::*;
pub use comprehensive_reporter::*;
pub use unit::*;
pub use integration::*;
pub use e2e::*;

/// 测试结果汇总
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestSuiteResult {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub execution_time_ms: u64,
    pub test_results: Vec<TestCaseResult>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TestCaseResult {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
}

impl TestSuiteResult {
    pub fn new(suite_name: String) -> Self {
        Self {
            suite_name,
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            execution_time_ms: 0,
            test_results: Vec::new(),
        }
    }

    pub fn add_test_result(&mut self, result: TestCaseResult) {
        self.total_tests += 1;
        match result.status {
            TestStatus::Passed => self.passed_tests += 1,
            TestStatus::Failed => self.failed_tests += 1,
            TestStatus::Skipped => {}
        }
        self.test_results.push(result);
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            (self.passed_tests as f64 / self.total_tests as f64) * 100.0
        }
    }
}

/// 测试工具函数
pub mod test_utils {
    use std::fs;
    use std::path::Path;
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// 创建临时工作流文件
    pub fn create_temp_workflow(_content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    /// 创建测试目录结构
    pub fn create_test_structure(base_path: &Path) -> std::io::Result<()> {
        fs::create_dir_all(base_path.join(".github/workflows"))?;
        fs::create_dir_all(base_path.join("tests"))?;
        fs::create_dir_all(base_path.join("src"))?;
        Ok(())
    }

    /// 生成随机标签用于测试
    pub fn generate_test_tag(server_name: &str, version: &str) -> String {
        format!("mcp-{}-v{}", server_name, version)
    }

    /// 模拟GitHub Actions环境变量
    pub fn setup_github_env_vars() {
        std::env::set_var("GITHUB_REPOSITORY", "test/repo");
        std::env::set_var("GITHUB_SHA", "abc123");
        std::env::set_var("GITHUB_REF", "refs/heads/master");
        std::env::set_var("GITHUB_TOKEN", "fake_token_for_testing");
    }

    /// 清理环境变量
    pub fn cleanup_github_env_vars() {
        std::env::remove_var("GITHUB_REPOSITORY");
        std::env::remove_var("GITHUB_SHA");
        std::env::remove_var("GITHUB_REF");
        std::env::remove_var("GITHUB_TOKEN");
    }
}