use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use github_actions_tests::*;
use std::time::Duration;

fn benchmark_workflow_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_validation");
    
    // 测试不同大小的配置文件
    let test_cases = vec![
        ("small", include_str!("../fixtures/ci_workflow.yml")),
        ("medium", include_str!("../fixtures/release_workflow.yml")),
        ("large", include_str!("../fixtures/security_workflow.yml")),
    ];
    
    for (name, content) in test_cases {
        group.bench_with_input(BenchmarkId::new("validation", name), &content, |b, content| {
            b.iter(|| {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&temp_file, content.as_bytes()).unwrap();
                let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
                black_box(validator.validate());
            })
        });
    }
    
    group.finish();
}

fn benchmark_security_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_testing");
    
    let test_cases = vec![
        ("clean", include_str!("../fixtures/ci_workflow.yml")),
        ("with_secrets", r#"
name: Test Workflow
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Deploy
        run: echo "api_key = 'sk-1234567890abcdef'"
"#),
        ("complex", include_str!("../fixtures/security_workflow.yml")),
    ];
    
    for (name, content) in test_cases {
        group.bench_with_input(BenchmarkId::new("security_scan", name), &content, |b, content| {
            b.iter(|| {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&temp_file, content.as_bytes()).unwrap();
                let tester = SecurityTester::new(temp_file.path().to_str().unwrap()).unwrap();
                black_box(tester.run_security_tests());
            })
        });
    }
    
    group.finish();
}

fn benchmark_performance_testing(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_testing");
    
    // 测试不同并发级别的性能
    let concurrency_levels = vec![1, 2, 4, 8];
    
    for concurrency in concurrency_levels {
        group.bench_with_input(BenchmarkId::new("concurrent_execution", concurrency), &concurrency, |b, &concurrency| {
            b.iter(|| {
                let tester = PerformanceTester::new(5, concurrency);
                let workflow_content = include_str!("../fixtures/ci_workflow.yml");
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&temp_file, workflow_content.as_bytes()).unwrap();
                
                let rt = tokio::runtime::Runtime::new().unwrap();
                black_box(rt.block_on(async {
                    tester.test_workflow_performance(temp_file.path().to_str().unwrap()).await.unwrap()
                }));
            })
        });
    }
    
    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");
    
    // 测试缓存启用和禁用的性能差异
    group.bench_function("cache_enabled", |b| {
        b.iter(|| {
            let tester = PerformanceTester::new(10, 1);
            let workflow_content = include_str!("../fixtures/ci_workflow.yml");
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::io::Write::write_all(&temp_file, workflow_content.as_bytes()).unwrap();
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            black_box(rt.block_on(async {
                tester.test_cache_performance(temp_file.path().to_str().unwrap(), true).await.unwrap()
            }));
        })
    });
    
    group.bench_function("cache_disabled", |b| {
        b.iter(|| {
            let tester = PerformanceTester::new(10, 1);
            let workflow_content = include_str!("../fixtures/ci_workflow.yml");
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::io::Write::write_all(&temp_file, workflow_content.as_bytes()).unwrap();
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            black_box(rt.block_on(async {
                tester.test_cache_performance(temp_file.path().to_str().unwrap(), false).await.unwrap()
            }));
        })
    });
    
    group.finish();
}

fn benchmark_workflow_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("workflow_execution");
    
    // 测试不同工作流的执行时间
    let workflows = vec![
        ("ci", include_str!("../fixtures/ci_workflow.yml")),
        ("release", include_str!("../fixtures/release_workflow.yml")),
        ("security", include_str!("../fixtures/security_workflow.yml")),
    ];
    
    for (name, content) in workflows {
        group.bench_with_input(BenchmarkId::new("execution_time", name), &content, |b, content| {
            b.iter(|| {
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                std::io::Write::write_all(&temp_file, content.as_bytes()).unwrap();
                let executor = WorkflowExecutor::new("/tmp", None);
                
                let rt = tokio::runtime::Runtime::new().unwrap();
                black_box(rt.block_on(async {
                    executor.execute_workflow_test(temp_file.path().to_str().unwrap(), "master").await.unwrap()
                }));
            })
        });
    }
    
    group.finish();
}

fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // 测试内存使用情况
    group.bench_function("large_workflow_validation", |b| {
        b.iter(|| {
            let large_workflow = r#"
name: Large Test Workflow
on:
  push:
    branches: [master, develop, feature/*]
  pull_request:
    branches: [master]
  schedule:
    - cron: '0 2 * * 1'

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta, nightly]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: aarch64-apple-darwin
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - name: Cache
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ matrix.target }}-${{ hashFiles('**/Cargo.lock') }}
      - run: cargo build --target ${{ matrix.target }}
      - run: cargo test --target ${{ matrix.target }}
      - run: cargo clippy --target ${{ matrix.target }}
"#;
            
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::io::Write::write_all(&temp_file, large_workflow.as_bytes()).unwrap();
            let validator = WorkflowValidator::new(temp_file.path().to_str().unwrap()).unwrap();
            black_box(validator.validate());
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_workflow_validation,
    benchmark_security_testing,
    benchmark_performance_testing,
    benchmark_cache_performance,
    benchmark_workflow_execution,
    benchmark_memory_usage
);
criterion_main!(benches);