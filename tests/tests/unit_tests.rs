#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_workflow_validator_valid_ci_workflow() {
        let workflow_content = r#"
name: CI
on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Run clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
      - name: Run tests
        run: cargo test --verbose
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();

        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_workflow_validator_missing_required_fields() {
        let workflow_content = r#"
name: Test Workflow
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "test"
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();

        assert!(!result.is_valid);
        assert!(result.errors.contains(&"Missing required field: on".to_string()));
    }

    #[test]
    fn test_workflow_validator_security_warnings() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: echo "api_key = 'secret123'"
      - run: echo "password = 'hardcoded'"
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();

        assert!(result.warnings.len() > 0);
        assert!(result.warnings.iter().any(|w| w.contains("hardcoded secrets")));
        assert!(result.warnings.iter().any(|w| w.contains("outdated checkout action")));
    }

    #[test]
    fn test_workflow_validator_performance_warnings() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "no cache configured"
  test2:
    runs-on: ubuntu-latest
    steps:
      - run: echo "another job"
  test3:
    runs-on: ubuntu-latest
    steps:
      - run: echo "third job"
  test4:
    runs-on: ubuntu-latest
    steps:
      - run: echo "fourth job"
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = validator.validate();

        assert!(result.warnings.iter().any(|w| w.contains("No caching configured")));
        assert!(result.info.iter().any(|i| i.contains("Multiple jobs detected")));
    }

    #[test]
    fn test_workflow_executor_trigger_conditions() {
        let workflow_content = r#"
name: Test Workflow
on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]
  schedule:
    - cron: '0 2 * * 1'
jobs:
  test:
    runs-on: ubuntu-latest
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let executor = WorkflowExecutor::new("/tmp", None);
        let results = executor.test_trigger_conditions(temp_file.path().to_str().unwrap()).unwrap();

        assert_eq!(results.len(), 3);
        
        let push_result = results.iter().find(|r| r.trigger_type == "push").unwrap();
        assert!(push_result.is_configured);
        assert_eq!(push_result.branches, vec!["master", "develop"]);

        let pr_result = results.iter().find(|r| r.trigger_type == "pull_request").unwrap();
        assert!(pr_result.is_configured);
        assert_eq!(pr_result.branches, vec!["master"]);

        let schedule_result = results.iter().find(|r| r.trigger_type == "schedule").unwrap();
        assert!(schedule_result.is_configured);
    }

    #[test]
    fn test_workflow_executor_matrix_configuration() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        target: [x86_64, aarch64]
    runs-on: ${{ matrix.os }}
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let executor = WorkflowExecutor::new("/tmp", None);
        let result = executor.test_matrix_configuration(temp_file.path().to_str().unwrap()).unwrap();

        assert!(result.has_matrix);
        assert!(result.is_valid);
        assert!(result.matrix_size > 0);
    }

    #[test]
    fn test_security_tester_secret_detection() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: |
          echo "API_KEY = 'sk-1234567890abcdef'"
          echo "AWS_SECRET_ACCESS_KEY = 'wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'"
          echo "github_token = '${{ secrets.GITHUB_TOKEN }}'"
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();

        assert!(!result.is_secure);
        assert!(result.score < 70);
        assert!(result.vulnerabilities.iter().any(|v| v.category == "Secret Leak"));
    }

    #[test]
    fn test_security_tester_permissions() {
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

        let temp_file = create_temp_workflow(workflow_content);
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();

        assert!(result.vulnerabilities.iter().any(|v| v.category == "Permissions"));
        assert!(result.vulnerabilities.iter().any(|v| v.recommendation.contains("minimal required permissions")));
    }

    #[test]
    fn test_security_tester_outdated_actions() {
        let workflow_content = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v1
      - uses: actions/setup-python@v1
      - uses: actions/cache@v1
      - run: echo "test"
"#;

        let temp_file = create_temp_workflow(workflow_content);
        let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
        let result = tester.run_security_tests();

        let outdated_count = result.vulnerabilities.iter()
            .filter(|v| v.category == "Dependency Security")
            .count();
        assert!(outdated_count >= 3);
    }

    #[test]
    fn test_test_suite_result() {
        let mut suite_result = TestSuiteResult::new("Test Suite".to_string());
        
        let test1 = TestCaseResult {
            name: "Test 1".to_string(),
            status: TestStatus::Passed,
            duration_ms: 100,
            error_message: None,
            output: Some("Test 1 passed".to_string()),
        };
        
        let test2 = TestCaseResult {
            name: "Test 2".to_string(),
            status: TestStatus::Failed,
            duration_ms: 200,
            error_message: Some("Test 2 failed".to_string()),
            output: Some("Test 2 output".to_string()),
        };

        suite_result.add_test_result(test1);
        suite_result.add_test_result(test2);

        assert_eq!(suite_result.total_tests, 2);
        assert_eq!(suite_result.passed_tests, 1);
        assert_eq!(suite_result.failed_tests, 1);
        assert_eq!(suite_result.success_rate(), 50.0);
    }

    #[test]
    fn test_generate_test_tag() {
        let tag = generate_test_tag("json-validator", "1.0.0");
        assert_eq!(tag, "mcp-json-validator-v1.0.0");
        
        let tag2 = generate_test_tag("task-orchestrator", "2.1.0");
        assert_eq!(tag2, "mcp-task-orchestrator-v2.1.0");
    }

    #[test]
    fn test_github_env_vars() {
        setup_github_env_vars();
        
        assert_eq!(std::env::var("GITHUB_REPOSITORY").unwrap(), "test/repo");
        assert_eq!(std::env::var("GITHUB_SHA").unwrap(), "abc123");
        assert_eq!(std::env::var("GITHUB_REF").unwrap(), "refs/heads/master");
        assert_eq!(std::env::var("GITHUB_TOKEN").unwrap(), "fake_token_for_testing");

        cleanup_github_env_vars();
        
        assert!(std::env::var("GITHUB_REPOSITORY").is_err());
        assert!(std::env::var("GITHUB_SHA").is_err());
    }
}