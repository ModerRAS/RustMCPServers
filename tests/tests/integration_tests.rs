#[cfg(test)]
mod integration_tests {
    use super::*;
    use crate::test_utils::*;
    use tempfile::{NamedTempFile, TempDir};
    use std::fs;
    use std::path::Path;
    use std::io::Write;

    #[tokio::test]
    async fn test_ci_workflow_integration() {
        // 创建测试仓库结构
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        create_test_structure(repo_path).unwrap();
        
        // 创建Cargo.toml
        let cargo_toml = r#"[workspace]
members = ["servers/*"]

[package]
name = "rust-mcp-servers"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
"#;
        fs::write(repo_path.join("Cargo.toml"), cargo_toml).unwrap();
        
        // 创建Cargo.lock
        fs::write(repo_path.join("Cargo.lock"), "# This is a dummy Cargo.lock file").unwrap();
        
        // 创建CI工作流
        let ci_workflow = r#"
name: CI
on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
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
    
    - name: Build
      run: cargo build --release --verbose

  security:
    name: Security audit
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
    
    - name: Run security audit
      run: cargo install cargo-audit && cargo audit
"#;
        
        let workflow_path = repo_path.join(".github/workflows/ci.yml");
        fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
        fs::write(&workflow_path, ci_workflow).unwrap();
        
        // 测试工作流验证
        let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
        let validation_result = validator.validate();
        
        assert!(validation_result.is_valid, "CI workflow should be valid");
        assert!(validation_result.errors.is_empty(), "CI workflow should have no errors");
        
        // 测试工作流执行器
        let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
        let trigger_results = executor.test_trigger_conditions(workflow_path.to_str().unwrap()).unwrap();
        
        assert_eq!(trigger_results.len(), 2, "Should have push and pull_request triggers");
        
        let push_trigger = trigger_results.iter().find(|t| t.trigger_type == "push").unwrap();
        assert!(push_trigger.is_configured);
        assert_eq!(push_trigger.branches, vec!["master", "develop"]);
        
        let pr_trigger = trigger_results.iter().find(|t| t.trigger_type == "pull_request").unwrap();
        assert!(pr_trigger.is_configured);
        assert_eq!(pr_trigger.branches, vec!["master"]);
        
        // 测试安全性
        let security_tester = SecurityTester::new(workflow_path.to_str().unwrap()).unwrap();
        let security_result = security_tester.run_security_tests();
        
        assert!(security_result.is_secure, "CI workflow should be secure");
        assert!(security_result.score >= 80, "CI workflow should have high security score");
        
        // 验证没有严重的安全漏洞
        let critical_vulns: Vec<_> = security_result.vulnerabilities.iter()
            .filter(|v| v.severity == Severity::Critical)
            .collect();
        assert_eq!(critical_vulns.len(), 0, "Should have no critical vulnerabilities");
    }

    #[tokio::test]
    async fn test_release_workflow_integration() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        create_test_structure(repo_path).unwrap();
        
        // 创建发布工作流
        let release_workflow = r#"
name: release

on:
  push:
    tags:
      - 'mcp-*-v*'

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        server: [json-validator, task-orchestrator]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            bin: json-validator
            server: json-validator
            server_dir: json-validator-server
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            bin: json-validator.exe
            server: json-validator
            server_dir: json-validator-server
          - os: macos-latest
            target: aarch64-apple-darwin
            bin: json-validator
            server: json-validator
            server_dir: json-validator-server
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            bin: task-orchestrator
            server: task-orchestrator
            server_dir: task-orchestrator
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            bin: task-orchestrator.exe
            server: task-orchestrator
            server_dir: task-orchestrator
          - os: macos-latest
            target: aarch64-apple-darwin
            bin: task-orchestrator
            server: task-orchestrator
            server_dir: task-orchestrator

    steps:
      - uses: actions/checkout@v4

      - name: parse tag
        id: vars
        run: |
          TAG=${GITHUB_REF#refs/tags/}
          SERVER=${TAG%%-v*}
          VERSION=${TAG##*-v}
          
          if [[ "$SERVER" == "mcp-${{ matrix.server }}" ]]; then
            echo "should_build=true" >> $GITHUB_OUTPUT
            echo "server=$SERVER" >> $GITHUB_OUTPUT
            echo "server_dir=${{ matrix.server_dir }}" >> $GITHUB_OUTPUT
            echo "version=$VERSION" >> $GITHUB_OUTPUT
            echo "bin=${{ matrix.bin }}" >> $GITHUB_OUTPUT
          else
            echo "should_build=false" >> $GITHUB_OUTPUT
          fi

      - uses: dtolnay/rust-toolchain@stable
        if: steps.vars.outputs.should_build == 'true'
        with:
          targets: ${{ matrix.target }}
      - run: |
          cd servers/${{ steps.vars.outputs.server_dir }}
          cargo build --release --bin ${{ steps.vars.outputs.bin }} --target ${{ matrix.target }}
        if: steps.vars.outputs.should_build == 'true'

      - name: upload
        uses: softprops/action-gh-release@v2
        if: steps.vars.outputs.should_build == 'true'
        with:
          tag_name: ${{ github.ref_name }}
          files: |
            servers/${{ steps.vars.outputs.server_dir }}/target/${{ matrix.target }}/release/${{ steps.vars.outputs.bin }}
"#;
        
        let workflow_path = repo_path.join(".github/workflows/release.yml");
        fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
        fs::write(&workflow_path, release_workflow).unwrap();
        
        // 测试工作流验证
        let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
        let validation_result = validator.validate();
        
        assert!(validation_result.is_valid, "Release workflow should be valid");
        
        // 测试触发条件
        let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
        let trigger_results = executor.test_trigger_conditions(workflow_path.to_str().unwrap()).unwrap();
        
        assert_eq!(trigger_results.len(), 1, "Should have tag trigger");
        let tag_trigger = trigger_results.first().unwrap();
        assert_eq!(tag_trigger.trigger_type, "push");
        
        // 测试矩阵配置
        let matrix_result = executor.test_matrix_configuration(workflow_path.to_str().unwrap()).unwrap();
        
        assert!(matrix_result.has_matrix, "Release workflow should have matrix");
        assert!(matrix_result.is_valid, "Matrix configuration should be valid");
        assert!(matrix_result.matrix_size >= 6, "Should have at least 6 matrix combinations");
        
        // 测试安全性
        let security_tester = SecurityTester::new(workflow_path.to_str().unwrap()).unwrap();
        let security_result = security_tester.run_security_tests();
        
        assert!(security_result.is_secure, "Release workflow should be secure");
        
        // 验证没有密钥泄露
        let secret_vulns: Vec<_> = security_result.vulnerabilities.iter()
            .filter(|v| v.category == "Secret Leak")
            .collect();
        assert_eq!(secret_vulns.len(), 0, "Should have no secret leaks");
    }

    #[tokio::test]
    async fn test_security_scan_workflow_integration() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        create_test_structure(repo_path).unwrap();
        
        // 创建安全扫描工作流
        let security_workflow = r#"
name: Security Scan

on:
  push:
    branches: [master, develop]
  pull_request:
    branches: [master]
  schedule:
    - cron: '0 2 * * 1'

jobs:
  security-scan:
    name: Security Vulnerability Scan
    runs-on: ubuntu-latest
    
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
        key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}
    
    - name: Install security tools
      run: |
        cargo install cargo-audit cargo-outdated cargo-tarpaulin || true
    
    - name: Run security audit
      run: |
        cargo audit
      continue-on-error: true
    
    - name: Check for outdated dependencies
      run: |
        cargo outdated --exit-code 1
      continue-on-error: true
    
    - name: Run clippy with security lints
      run: |
        cargo clippy --all-targets --all-features -- -D warnings -W clippy::cargo
      continue-on-error: true
    
    - name: Check for common security issues
      run: |
        if grep -r "api[_-]key\|secret\|password" --include="*.rs" --include="*.toml" --exclude-dir=target --exclude-dir=tmp . | grep -v "example\|test\|mock"; then
          echo "⚠️  Potential hardcoded secrets found"
        fi
        
        if grep -r "unsafe" --include="*.rs" --exclude-dir=target --exclude-dir=tmp . | grep -v "test\|example"; then
          echo "⚠️  Unsafe code blocks found (review required)"
        fi
        
        echo "✅ Security pattern check completed"
    
    - name: Generate security report
      run: |
        echo "# Security Scan Report" > security-report.md
        echo "" >> security-report.md
        echo "## Scan Date: $(date)" >> security-report.md
        echo "" >> security-report.md
        echo "## Commit: ${{ github.sha }}" >> security-report.md
    
    - name: Upload security report
      uses: actions/upload-artifact@v3
      with:
        name: security-report
        path: security-report.md
        retention-days: 30
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
        let validation_result = validator.validate();
        assert!(validation_result.warnings.iter().any(|w| w.contains("caching")), 
                "Security workflow should use caching");
    }

    #[tokio::test]
    async fn test_performance_workflow_integration() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        create_test_structure(repo_path).unwrap();
        
        // 创建性能测试工作流
        let performance_workflow = r#"
name: Performance Test

on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

jobs:
  performance-test:
    runs-on: ubuntu-latest
    
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
        key: ${{ runner.os }}-cargo-${{ hashFiles('Cargo.lock') }}
    
    - name: Run performance tests
      run: |
        cargo bench --verbose
    
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
        
        // 测试性能测试器
        let performance_tester = PerformanceTester::new(3, 2);
        let performance_result = performance_tester.test_workflow_performance(workflow_path.to_str().unwrap()).await.unwrap();
        
        assert!(performance_result.average_duration_ms > 0.0, "Performance test should have duration");
        assert!(performance_result.success_rate > 0.0, "Performance test should have success rate");
        assert_eq!(performance_result.test_runs, 3, "Should run 3 tests");
        assert_eq!(performance_result.concurrent_runs, 2, "Should run 2 concurrent tests");
        
        // 测试缓存性能
        let cache_result = performance_tester.test_cache_performance(workflow_path.to_str().unwrap(), true).await.unwrap();
        
        assert!(cache_result.cache_enabled, "Cache should be enabled");
        assert!(cache_result.cache_hit_rate >= 0.0 && cache_result.cache_hit_rate <= 100.0, 
                "Cache hit rate should be between 0 and 100");
        
        // 测试并发性能
        let concurrent_results = performance_tester.test_concurrent_performance(workflow_path.to_str().unwrap(), 3).await.unwrap();
        
        assert_eq!(concurrent_results.len(), 3, "Should have results for 1, 2, and 3 concurrent runs");
        
        // 验证性能随并发数增加的变化
        for (i, result) in concurrent_results.iter().enumerate() {
            assert_eq!(result.concurrent_runs, i + 1, "Concurrent runs should match index");
        }
    }

    #[tokio::test]
    async fn test_workflow_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        create_test_structure(repo_path).unwrap();
        
        // 创建有错误的工作流
        let error_workflow = r#"
name: Error Test Workflow
on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v1
      - run: echo "This will fail"
        continue-on-error: true
      - run: exit 1
        if: failure()
      - name: This step should not run
        run: echo "This should not run"
        if: success()
"#;
        
        let workflow_path = repo_path.join(".github/workflows/error-test.yml");
        fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
        fs::write(&workflow_path, error_workflow).unwrap();
        
        // 测试工作流验证应该检测到问题
        let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
        let validation_result = validator.validate();
        
        // 应该检测到使用过时的checkout action
        assert!(validation_result.warnings.iter().any(|w| w.contains("outdated checkout action")));
        
        // 测试安全性
        let security_tester = SecurityTester::new(workflow_path.to_str().unwrap()).unwrap();
        let security_result = security_tester.run_security_tests();
        
        // 应该检测到过时的动作版本
        assert!(security_result.vulnerabilities.iter().any(|v| v.category == "Dependency Security"));
    }

    #[tokio::test]
    async fn test_multi_workspace_integration() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();
        
        create_test_structure(repo_path).unwrap();
        
        // 创建多个服务器项目
        let servers = ["json-validator-server", "task-orchestrator"];
        
        for server in &servers {
            let server_path = repo_path.join("servers").join(server);
            fs::create_dir_all(&server_path).unwrap();
            
            let cargo_toml = format!(r#"
[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.0", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
"#, server);
            
            fs::write(server_path.join("Cargo.toml"), cargo_toml).unwrap();
            
            let src_dir = server_path.join("src");
            fs::create_dir_all(&src_dir).unwrap();
            
            let main_rs = r#"
use std::io;

fn main() {
    println!("Hello, world!");
}
"#;
            
            fs::write(src_dir.join("main.rs"), main_rs).unwrap();
        }
        
        // 创建工作空间Cargo.toml
        let workspace_cargo = r#"
[workspace]
members = ["servers/*"]

[workspace.dependencies]
tokio = "1.0"
serde = "1.0"
"#;
        
        fs::write(repo_path.join("Cargo.toml"), workspace_cargo).unwrap();
        
        // 创建CI工作流
        let ci_workflow = r#"
name: Multi-Workspace CI
on:
  push:
    branches: [master]
  pull_request:
    branches: [master]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        server: [json-validator-server, task-orchestrator]
    
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
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Test ${{ matrix.server }}
        run: |
          cd servers/${{ matrix.server }}
          cargo test
      
      - name: Build ${{ matrix.server }}
        run: |
          cd servers/${{ matrix.server }}
          cargo build --release
"#;
        
        let workflow_path = repo_path.join(".github/workflows/multi-workspace-ci.yml");
        fs::create_dir_all(workflow_path.parent().unwrap()).unwrap();
        fs::write(&workflow_path, ci_workflow).unwrap();
        
        // 测试工作流验证
        let validator = WorkflowValidator::new(workflow_path.to_str().unwrap()).unwrap();
        let validation_result = validator.validate();
        
        assert!(validation_result.is_valid, "Multi-workspace CI should be valid");
        
        // 测试矩阵配置
        let executor = WorkflowExecutor::new(repo_path.to_str().unwrap(), None);
        let matrix_result = executor.test_matrix_configuration(workflow_path.to_str().unwrap()).unwrap();
        
        assert!(matrix_result.has_matrix, "Should have matrix configuration");
        assert!(matrix_result.is_valid, "Matrix should be valid");
        assert_eq!(matrix_result.matrix_size, 2, "Should have 2 server combinations");
        
        // 测试性能
        let performance_tester = PerformanceTester::new(2, 1);
        let performance_result = performance_tester.test_workflow_performance(workflow_path.to_str().unwrap()).await.unwrap();
        
        assert!(performance_result.success_rate > 0.0, "Multi-workspace CI should have success rate");
    }
}