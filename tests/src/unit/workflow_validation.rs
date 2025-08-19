//! 工作流验证单元测试
//! 
//! 测试工作流验证的各个方面，包括：
//! - YAML语法验证
//! - 工作流结构验证
//! - 任务依赖关系验证
//! - 步骤配置验证
//! - 触发条件验证

use std::path::Path;
use tempfile::NamedTempFile;
use crate::test_utils;
use crate::workflow_validator::WorkflowValidator;
use serde_yaml::Value;

/// 测试工作流语法验证
#[cfg(test)]
mod workflow_syntax_tests {
    use super::*;

    #[test]
    fn test_valid_yaml_syntax() {
        let valid_yaml = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(valid_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_syntax(temp_file.path());

        assert!(result.is_valid(), "有效的YAML语法应该通过验证");
        assert!(result.errors.is_empty(), "有效YAML不应有错误");
    }

    #[test]
    fn test_invalid_yaml_syntax() {
        let invalid_yaml = r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      invalid_yaml_structure:
        missing: value
        unclosed: [
"#;

        let temp_file = test_utils::create_temp_workflow(invalid_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_syntax(temp_file.path());

        assert!(!result.is_valid(), "无效的YAML语法应该失败");
        assert!(!result.errors.is_empty(), "无效YAML应有错误信息");
    }

    #[test]
    fn test_empty_workflow() {
        let empty_yaml = "";
        let temp_file = test_utils::create_temp_workflow(empty_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_syntax(temp_file.path());

        assert!(!result.is_valid(), "空工作流应该失败");
    }

    #[test]
    fn test_unicode_content() {
        let unicode_yaml = r#"
name: 测试工作流
description: 包含中文的工作流
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: 设置环境
        run: echo "Hello 世界"
"#;

        let temp_file = test_utils::create_temp_workflow(unicode_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_syntax(temp_file.path());

        assert!(result.is_valid(), "包含Unicode的工作流应该通过验证");
    }
}

/// 测试工作流结构验证
#[cfg(test)]
mod workflow_structure_tests {
    use super::*;

    #[test]
    fn test_complete_workflow_structure() {
        let complete_yaml = r#"
name: Complete Workflow
on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Run Tests
    runs-on: ubuntu-latest
    timeout-minutes: 30
    
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Run Tests
        run: cargo test

  build:
    name: Build
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release
"#;

        let temp_file = test_utils::create_temp_workflow(complete_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        assert!(result.is_valid(), "完整的工作流结构应该通过验证");
    }

    #[test]
    fn test_missing_required_fields() {
        let incomplete_yaml = r#"
name: Incomplete Workflow
# missing 'on' field
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(incomplete_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        assert!(!result.is_valid(), "缺少必需字段的工作流应该失败");
        assert!(result.errors.iter().any(|e| e.contains("on")));
    }

    #[test]
    fn test_invalid_job_configuration() {
        let invalid_job_yaml = r#"
name: Invalid Job Config
on: [push]
jobs:
  test:
    # missing runs-on
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(invalid_job_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        assert!(!result.is_valid(), "无效的作业配置应该失败");
    }

    #[test]
    fn test_empty_jobs_section() {
        let empty_jobs_yaml = r#"
name: Empty Jobs
on: [push]
jobs: {}
"#;

        let temp_file = test_utils::create_temp_workflow(empty_jobs_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        assert!(!result.is_valid(), "空的作业部分应该失败");
    }
}

/// 测试任务依赖关系验证
#[cfg(test)]
mod job_dependency_tests {
    use super::*;

    #[test]
    fn test_valid_job_dependencies() {
        let yaml_with_deps = r#"
name: Job Dependencies
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

  build:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4

  deploy:
    runs-on: ubuntu-latest
    needs: [test, build]
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(yaml_with_deps);
        let validator = WorkflowValidator::new();
        let result = validator.validate_job_dependencies(temp_file.path());

        assert!(result.is_valid(), "有效的任务依赖关系应该通过验证");
    }

    #[test]
    fn test_circular_dependencies() {
        let circular_deps_yaml = r#"
name: Circular Dependencies
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    needs: deploy
    steps:
      - uses: actions/checkout@v4

  deploy:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(circular_deps_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_job_dependencies(temp_file.path());

        assert!(!result.is_valid(), "循环依赖应该失败");
        assert!(result.errors.iter().any(|e| e.contains("circular")));
    }

    #[test]
    fn test_nonexistent_job_dependency() {
        let nonexistant_deps_yaml = r#"
name: Nonexistent Dependency
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    needs: nonexistent_job
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(nonexistant_deps_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_job_dependencies(temp_file.path());

        assert!(!result.is_valid(), "不存在的任务依赖应该失败");
    }

    #[test]
    fn test_self_dependency() {
        let self_dep_yaml = r#"
name: Self Dependency
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    needs: test
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(self_dep_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_job_dependencies(temp_file.path());

        assert!(!result.is_valid(), "自依赖应该失败");
    }
}

/// 测试步骤配置验证
#[cfg(test)]
mod step_validation_tests {
    use super::*;

    #[test]
    fn test_valid_step_configuration() {
        let valid_steps_yaml = r#"
name: Valid Steps
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - name: Run Tests
        run: cargo test --all
        env:
          RUST_BACKTRACE: 1
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: test-results
          path: target/
"#;

        let temp_file = test_utils::create_temp_workflow(valid_steps_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_steps(temp_file.path());

        assert!(result.is_valid(), "有效的步骤配置应该通过验证");
    }

    #[test]
    fn test_invalid_step_configuration() {
        let invalid_steps_yaml = r#"
name: Invalid Steps
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Invalid Step
        # missing 'uses' or 'run'
        with:
          some: config
"#;

        let temp_file = test_utils::create_temp_workflow(invalid_steps_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_steps(temp_file.path());

        assert!(!result.is_valid(), "无效的步骤配置应该失败");
    }

    #[test]
    fn test_step_with_run_and_uses() {
        let invalid_step_yaml = r#"
name: Invalid Step Mix
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        run: echo "This should not be allowed"
"#;

        let temp_file = test_utils::create_temp_workflow(invalid_step_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_steps(temp_file.path());

        assert!(!result.is_valid(), "同时使用run和uses的步骤应该失败");
    }

    #[test]
    fn test_empty_steps() {
        let empty_steps_yaml = r#"
name: Empty Steps
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps: []
"#;

        let temp_file = test_utils::create_temp_workflow(empty_steps_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_steps(temp_file.path());

        assert!(!result.is_valid(), "空的步骤列表应该失败");
    }
}

/// 测试触发条件验证
#[cfg(test)]
mod trigger_validation_tests {
    use super::*;

    #[test]
    fn test_valid_triggers() {
        let valid_triggers_yaml = r#"
name: Valid Triggers
on:
  push:
    branches: [main, develop]
    tags: ['v*']
    paths: ['src/**']
  pull_request:
    branches: [main]
    types: [opened, synchronize]
  schedule:
    - cron: '0 2 * * *'
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to deploy'
        required: true
        default: 'staging'
        type: choice
        options: [staging, production]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(valid_triggers_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_triggers(temp_file.path());

        assert!(result.is_valid(), "有效的触发条件应该通过验证");
    }

    #[test]
    fn test_invalid_cron_expression() {
        let invalid_cron_yaml = r#"
name: Invalid Cron
on:
  schedule:
    - cron: 'invalid cron expression'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(invalid_cron_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_triggers(temp_file.path());

        assert!(!result.is_valid(), "无效的cron表达式应该失败");
    }

    #[test]
    fn test_empty_triggers() {
        let empty_triggers_yaml = r#"
name: Empty Triggers
on: {}

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(empty_triggers_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_triggers(temp_file.path());

        assert!(!result.is_valid(), "空的触发条件应该失败");
    }

    #[test]
    fn test_invalid_event_type() {
        let invalid_event_yaml = r#"
name: Invalid Event
on:
  invalid_event:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(invalid_event_yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_triggers(temp_file.path());

        assert!(!result.is_valid(), "无效的事件类型应该失败");
    }
}

/// 性能测试 - 验证验证器的性能
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_validation_performance() {
        let large_yaml = r#"
name: Large Workflow
on: [push, pull_request, workflow_dispatch]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
"#;

        // 创建一个大的工作流文件
        let mut large_yaml = large_yaml.to_string();
        for i in 0..100 {
            large_yaml.push_str(&format!(r#"
  test_{}:
    runs-on: ubuntu-latest
    timeout-minutes: 30
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
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
      - name: Run tests
        run: cargo test --package test-package-{} --verbose
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: test-results-{}
          path: target/
"#, i, i, i));
        }

        let temp_file = test_utils::create_temp_workflow(&large_yaml);
        let validator = WorkflowValidator::new();
        
        let start = Instant::now();
        let result = validator.validate_all(temp_file.path());
        let duration = start.elapsed();

        assert!(result.is_valid(), "大型工作流应该通过验证");
        assert!(duration.as_millis() < 1000, "验证大型工作流应该少于1秒");
    }

    #[test]
    fn test_memory_usage() {
        // 这个测试需要更复杂的内存使用监控
        // 这里我们只测试基本功能
        let yaml = r#"
name: Memory Test
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#;

        let temp_file = test_utils::create_temp_workflow(yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_all(temp_file.path());

        assert!(result.is_valid(), "内存测试应该通过");
    }
}

/// 边界条件测试
#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_extremely_long_job_name() {
        let long_name = "a".repeat(1000);
        let yaml = format!(r#"
name: Long Job Name
on: [push]
jobs:
  {}:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#, long_name);

        let temp_file = test_utils::create_temp_workflow(&yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        // 长作业名应该被限制或拒绝
        assert!(!result.is_valid(), "过长的作业名应该失败");
    }

    #[test]
    fn test_maximum_job_count() {
        let mut yaml = r#"
name: Max Jobs
on: [push]
jobs:
"#.to_string();

        // 创建256个作业（超过GitHub限制）
        for i in 0..256 {
            yaml.push_str(&format!(r#"
  job_{}:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
"#, i));
        }

        let temp_file = test_utils::create_temp_workflow(&yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        assert!(!result.is_valid(), "超过最大作业数限制应该失败");
    }

    #[test]
    fn test_deep_nesting() {
        let yaml = r#"
name: Deep Nesting
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
        include:
          - os: ubuntu-latest
            rust: stable
            features: ['full']
          - os: windows-latest
            rust: stable
            features: ['minimal']
    steps:
      - uses: actions/checkout@v4
      - name: Test
        run: echo "Deep nesting test"
"#;

        let temp_file = test_utils::create_temp_workflow(yaml);
        let validator = WorkflowValidator::new();
        let result = validator.validate_structure(temp_file.path());

        assert!(result.is_valid(), "深层嵌套的工作流应该通过验证");
    }
}