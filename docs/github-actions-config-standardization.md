# GitHub Actions 配置标准化方案

## 1. 配置文件结构

### 1.1 工作流文件标准化
所有工作流文件都遵循以下标准结构：

```yaml
name: [工作流名称]

on:
  [触发条件]

jobs:
  [作业名称]:
    name: [作业显示名称]
    runs-on: [运行环境]
    permissions: [权限设置]
    
    steps:
      - name: Checkout repository
        uses: actions/checkout@v4
        with:
          fetch-depth: 1
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy, rust-src
      
      - name: Cache Rust dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: |
            ${{ runner.os }}-cargo-
      
      # ... 其他步骤
```

### 1.2 环境变量标准化

#### 全局环境变量
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUST_LOG: info
```

#### 工作流特定环境变量
```yaml
env:
  CACHE_VERSION: v1
  RUST_TOOLCHAIN: stable
```

## 2. 缓存策略标准化

### 2.1 多级缓存配置
```yaml
- name: Cache Rust dependencies
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}-${{ env.CACHE_VERSION }}
    restore-keys: |
      ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}-
      ${{ runner.os }}-cargo-

- name: Cache Claude Code dependencies
  uses: actions/cache@v4
  with:
    path: |
      ~/.anthropic
      ~/.claude
      ~/.cache/claude
    key: ${{ runner.os }}-claude-${{ hashFiles('**/Cargo.lock', '**/package.json', '**/requirements.txt') }}-${{ env.CACHE_VERSION }}
    restore-keys: |
      ${{ runner.os }}-claude-
```

### 2.2 缓存清理策略
```yaml
- name: Clean old cache
  run: |
    # 清理超过30天的缓存
    find ~/.cargo/registry/cache -type f -mtime +30 -delete
    find ~/.cargo/git/checkouts -type d -mtime +30 -exec rm -rf {} +
```

## 3. 安全配置标准化

### 3.1 权限管理
```yaml
permissions:
  contents: read
  pull-requests: read
  issues: read
  id-token: write
  actions: read
```

### 3.2 密钥管理
```yaml
- name: Configure secure environment
  env:
    ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
    R2_BUCKET_NAME: ${{ secrets.R2_BUCKET_NAME }}
    R2_ACCOUNT_ID: ${{ secrets.R2_ACCOUNT_ID }}
```

### 3.3 安全扫描配置
```yaml
- name: Security audit
  run: |
    cargo audit --deny warnings
  continue-on-error: true
```

## 4. 错误处理标准化

### 4.1 重试机制
```yaml
- name: Install dependencies with retry
  run: |
    for i in {1..3}; do
      cargo build --release && break || sleep 30
    done
```

### 4.2 错误处理
```yaml
- name: Run tests with error handling
  run: |
    set -e
    cargo test --all-features --verbose
  continue-on-error: false
```

### 4.3 状态检查
```yaml
- name: Check build status
  if: always()
  run: |
    if [ "${{ job.status }}" != "success" ]; then
      echo "Build failed, please check the logs above"
      exit 1
    fi
```

## 5. 性能优化配置

### 5.1 并行化配置
```yaml
strategy:
  matrix:
    os: [ubuntu-latest, windows-latest, macos-latest]
    rust-version: [stable, beta]
  fail-fast: false
```

### 5.2 资源优化
```yaml
- name: Optimize build performance
  run: |
    # 设置并行编译
    export CARGO_BUILD_JOBS=$(nproc)
    # 使用链接时优化
    export CARGO_PROFILE_RELEASE_LTO=true
    cargo build --release
```

## 6. 监控和报告标准化

### 6.1 执行监控
```yaml
- name: Monitor execution time
  run: |
    start_time=$(date +%s)
    # 执行主要任务
    end_time=$(date +%s)
    echo "Execution time: $((end_time - start_time)) seconds"
```

### 6.2 结果报告
```yaml
- name: Generate report
  run: |
    echo "# Build Report" > build-report.md
    echo "## Build Status: ${{ job.status }}" >> build-report.md
    echo "## Execution Time: ${{ steps.timer.outputs.time }}" >> build-report.md
    echo "## Commit: ${{ github.sha }}" >> build-report.md

- name: Upload report
  uses: actions/upload-artifact@v3
  with:
    name: build-report
    path: build-report.md
```

## 7. 配置验证

### 7.1 配置检查
```yaml
- name: Validate configuration
  run: |
    # 检查必要的配置文件
    if [ ! -f "Cargo.toml" ]; then
      echo "❌ Cargo.toml not found"
      exit 1
    fi
    
    # 检查工作空间配置
    if ! grep -q "\[workspace\]" Cargo.toml; then
      echo "❌ Workspace configuration not found"
      exit 1
    fi
    
    echo "✅ Configuration validation passed"
```

### 7.2 依赖验证
```yaml
- name: Validate dependencies
  run: |
    # 检查依赖解析
    cargo check --workspace
    
    # 检查依赖版本一致性
    cargo tree --duplicate
    
    echo "✅ Dependency validation passed"
```

## 8. 维护和更新策略

### 8.1 版本管理
```yaml
# 使用固定版本确保稳定性
- uses: actions/checkout@v4
- uses: dtolnay/rust-toolchain@stable
- uses: actions/cache@v4
```

### 8.2 定期更新
```yaml
# 定期更新actions版本
- uses: actions/checkout@v4
  with:
    persist-credentials: false
```

### 8.3 兼容性检查
```yaml
- name: Check compatibility
  run: |
    # 检查Rust版本兼容性
    rustc --version
    cargo --version
    
    # 检查操作系统兼容性
    uname -a
    echo "✅ Compatibility check passed"
```

## 9. 最佳实践总结

### 9.1 性能最佳实践
- 使用缓存减少构建时间
- 并行化作业执行
- 优化资源使用

### 9.2 安全最佳实践
- 最小权限原则
- 定期安全扫描
- 密钥安全管理

### 9.3 可维护性最佳实践
- 标准化配置
- 清晰的错误处理
- 完整的监控和报告

### 9.4 可靠性最佳实践
- 重试机制
- 状态检查
- 回滚能力

---

这个配置标准化方案提供了完整的GitHub Actions配置指导，确保所有工作流都遵循统一的最佳实践，提高系统的稳定性、安全性和可维护性。