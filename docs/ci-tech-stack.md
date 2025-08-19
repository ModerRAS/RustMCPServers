# GitHub Actions CI 技术栈决策文档

## 技术栈概览

本文档详细说明了 Rust MCP Servers 项目 GitHub Actions CI 构建系统的技术栈选择、版本决策和配置策略。重点关注依赖管理、构建优化、缓存策略和监控系统的技术选择。

## 构建工具栈

### Rust 工具链
| 工具 | 版本 | 选择理由 | 配置 |
|------|------|----------|------|
| Rust Toolchain | 1.80+ stable | 最新稳定版本，支持所有依赖，包含最新安全修复 | `rust-toolchain.toml` |
| Cargo | 1.80+ | 包管理器，支持 workspace 2.0 resolver | 内置 |
| Rustfmt | 1.80+ | 代码格式化，确保代码风格一致 | 通过 `rustup component add` |
| Clippy | 1.80+ | 静态分析工具，提前发现潜在问题 | 通过 `rustup component add` |

**选择理由**:
- **稳定性**: 使用稳定版本避免 nightly 版本的不稳定性
- **安全性**: 最新稳定版本包含已知安全漏洞修复
- **兼容性**: 确保所有依赖都能正常工作
- **性能**: 新版本通常包含性能优化

### GitHub Actions 版本
| 组件 | 版本 | 选择理由 |
|------|------|----------|
| Actions/Checkout | v4 | 最新版本，支持 sparse checkout 等新特性 |
| Actions/Cache | v4 | 改进的缓存性能和更大的缓存容量 |
| Actions/Upload-Artifact | v4 | 支持更大的 artifact 文件和更好的压缩 |
| Actions/Download-Artifact | v4 | 改进的下载性能和稳定性 |
| dtolnay/rust-toolchain | stable | 专业的 Rust 工具链管理 Action |

**版本策略**:
- **最新稳定版本**: 使用最新的稳定版本以获得最佳性能和功能
- **兼容性检查**: 确保各个 Action 版本之间的兼容性
- **定期更新**: 建立定期更新机制，避免版本过旧

## 依赖管理技术栈

### Workspace 依赖管理
```toml
# Cargo.toml - workspace 依赖统一配置
[workspace]
resolver = "2"
members = [
    "servers/json-validator-server", 
    "servers/task-orchestrator",
    "tests",
]

[workspace.dependencies]
# 核心依赖 - 使用精确版本或兼容版本
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1.83"
futures = "0.3.31"

# 序列化依赖
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# HTTP 相关
reqwest = { version = "0.12", features = ["json", "stream"] }
axum = "0.7"
tower = "0.5"

# MCP 协议
rmcp = { version = "0.5", features = ["transport-io", "transport-child-process", "macros"] }
```

### 依赖版本策略
| 策略 | 应用场景 | 示例 | 优势 |
|------|----------|------|------|
| 精确版本 | 核心依赖 | `tokio = "1.40"` | 确保稳定性，避免意外更新 |
| 兼容版本 | 库依赖 | `serde = { version = "1.0" }` | 允许补丁更新，修复安全问题 |
| 功能特性 | 按需启用 | `reqwest = { version = "0.12", features = ["json"] }` | 减少编译时间和二进制大小 |

### 依赖安全工具
| 工具 | 用途 | 集成方式 | 执行时机 |
|------|------|----------|----------|
| cargo-audit | 安全漏洞扫描 | CI 工作流 | 每次 PR 和提交 |
| cargo-deny | 依赖许可证检查 | CI 工作流 | 每次 PR 和提交 |
| cargo-outdated | 依赖更新检查 | 手动执行 | 定期维护 |
| cargo-tree | 依赖树分析 | 开发工具 | 调试依赖问题 |

## 缓存策略技术栈

### 多层缓存架构
```yaml
# 缓存层级配置
cache_strategy:
  # 第一层：依赖缓存
  dependency_cache:
    paths:
      - ~/.cargo/registry/
      - ~/.cargo/git/
      - ~/.cargo/bin/
    key: cargo-deps-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: |
      cargo-deps-${{ hashFiles('**/Cargo.lock') }}
      cargo-deps-
    
  # 第二层：构建缓存
  build_cache:
    paths:
      - target/
    key: cargo-build-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-${{ github.sha }}
    restore-keys: |
      cargo-build-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}
      cargo-build-${{ runner.os }}-
    
  # 第三层：工具缓存
  tool_cache:
    paths:
      - ~/.rustup/toolchains/
    key: rust-toolchain-${{ hashFiles('rust-toolchain.toml') }}
```

### 缓存优化技术
| 技术 | 实现方式 | 预期效果 | 适用场景 |
|------|----------|----------|----------|
| 分层缓存 | 按数据类型分层缓存 | 提高缓存命中率 | 大型项目 |
| 增量缓存 | 只缓存变更部分 | 减少缓存时间 | 频繁构建 |
| 智能键值 | 基于内容生成缓存键 | 精确缓存匹配 | 依赖变更 |
| 缓存压缩 | 压缩缓存内容 | 减少存储空间 | 大型缓存 |

## 测试框架技术栈

### 测试工具选择
| 工具 | 类型 | 版本 | 选择理由 |
|------|------|------|----------|
| cargo-test | 单元测试 | 内置 | Rust 官方测试框架 |
| tokio-test | 异步测试 | workspace | 异步代码测试支持 |
| criterion | 性能测试 | "0.5" | 专业的基准测试 |
| mockito | HTTP Mock | workspace | HTTP 服务模拟 |
| tempfile | 临时文件 | "3.8" | 测试文件管理 |

### 测试分类和策略
```yaml
test_strategy:
  # 单元测试
  unit_tests:
    command: cargo test --lib
    parallel: true
    timeout: 10m
    coverage: true
    
  # 集成测试
  integration_tests:
    command: cargo test --test integration_tests
    dependencies: ["unit_tests"]
    timeout: 20m
    
  # E2E 测试
  e2e_tests:
    command: cargo test --test e2e_tests
    dependencies: ["integration_tests"]
    timeout: 30m
    
  # 性能测试
  performance_tests:
    command: cargo bench
    dependencies: ["unit_tests"]
    timeout: 45m
```

## 监控和告警技术栈

### 监控指标收集
| 指标类型 | 收集方式 | 存储方式 | 告警阈值 |
|----------|----------|----------|----------|
| 构建时间 | GitHub Actions | 时间序列数据库 | > 30 分钟 |
| 成功率 | GitHub Actions | 指标数据库 | < 95% |
| 缓存命中率 | 缓存统计 | 指标数据库 | < 80% |
| 测试覆盖率 | cargo-llvm-cov | 报告文件 | < 70% |

### 告警机制
```yaml
alerting:
  # 构建失败告警
  build_failure:
    severity: critical
    channels: ["slack", "email"]
    conditions:
      - status == "failure"
      - workflow == "ci.yml"
    
  # 性能下降告警
  performance_regression:
    severity: warning
    channels: ["slack"]
    conditions:
      - build_time > threshold * 1.5
      - comparison == "previous_build"
    
  # 缓存失效告警
  cache_miss:
    severity: info
    channels: ["logs"]
    conditions:
      - cache_hit_rate < 0.8
      - window == "24h"
```

## 构建优化技术栈

### 并行构建策略
```yaml
parallel_build:
  # 并行任务配置
  jobs:
    - name: format-check
      command: cargo fmt --all -- --check
      timeout: 5m
      
    - name: clippy-check
      command: cargo clippy --all-targets --all-features -- -D warnings
      timeout: 15m
      
    - name: unit-tests
      command: cargo test --lib
      timeout: 20m
      
    - name: integration-tests
      command: cargo test --test integration_tests
      depends_on: ["unit-tests"]
      timeout: 25m
      
    - name: build-release
      command: cargo build --release
      depends_on: ["unit-tests", "integration-tests"]
      timeout: 30m
```

### 编译优化技术
| 优化技术 | 实现方式 | 效果 | 适用场景 |
|----------|----------|------|----------|
| LTO (Link Time Optimization) | `lto = true` | 减少二进制大小 | 发布构建 |
| 代码生成单元 | `codegen-units = 1` | 更好的优化 | 发布构建 |
| 增量编译 | `incremental = true` | 更快的重新编译 | 开发构建 |
| 并行代码生成 | `codegen-units = 16` | 更快的编译速度 | 开发构建 |

## 安全技术栈

### 代码安全扫描
| 工具 | 扫描类型 | 执行时机 | 集成方式 |
|------|----------|----------|----------|
| cargo-audit | 依赖漏洞 | 每次 PR | GitHub Action |
| cargo-deny | 许可证合规 | 每次 PR | GitHub Action |
| clippy | 代码质量 | 每次 PR | Cargo 内置 |
| rustfmt | 代码格式 | 每次 PR | Cargo 内置 |

### 构建环境安全
```yaml
security_environment:
  # 容器安全
  container:
    image: ubuntu:22.04
    user: non-root
    read_only: true
    
  # 权限限制
  permissions:
    contents: read
    actions: write
    checks: write
    
  # 网络限制
  network:
    allowed_hosts:
      - github.com
      - crates.io
      - static.crates.io
```

## 部署技术栈

### 发布策略
| 策略 | 实现方式 | 优势 | 适用场景 |
|------|----------|------|----------|
| 标签触发 | Git tag 自动发布 | 版本控制清晰 | 正式发布 |
| 分支触发 | branch 保护 | 开发流程集成 | 开发环境 |
| 手动触发 | workflow_dispatch | 灵活控制 | 特殊场景 |

### 多平台构建
```yaml
matrix_build:
  include:
    - os: ubuntu-latest
      target: x86_64-unknown-linux-gnu
      rust_target: x86_64-unknown-linux-gnu
      
    - os: windows-latest
      target: x86_64-pc-windows-msvc
      rust_target: x86_64-pc-windows-msvc
      
    - os: macos-latest
      target: x86_64-apple-darwin
      rust_target: x86_64-apple-darwin
      
    - os: macos-latest
      target: aarch64-apple-darwin
      rust_target: aarch64-apple-darwin
```

## 配置管理技术栈

### 环境配置
```yaml
environment_config:
  # 开发环境
  development:
    rust_profile: dev
    features: ["dev-tools"]
    debug: true
    
  # 测试环境
  testing:
    rust_profile: test
    features: ["test-utils"]
    debug: true
    
  # 生产环境
  production:
    rust_profile: release
    features: []
    debug: false
    lto: true
```

### 特性开关
```toml
[features]
default = []

# 开发特性
dev-tools = ["tokio/full", "tracing/max_level_debug"]

# 测试特性
test-utils = ["mockito", "tempfile"]

# 生产特性
performance = ["tokio/full", "tracing/max_level_info"]
```

## 版本管理技术栈

### 语义化版本控制
| 版本类型 | 格式 | 触发条件 | 示例 |
|----------|------|----------|------|
| 主版本 | MAJOR.MINOR.PATCH | 破坏性变更 | 1.0.0 → 2.0.0 |
| 次版本 | MAJOR.MINOR.PATCH | 新功能添加 | 1.0.0 → 1.1.0 |
| 补丁版本 | MAJOR.MINOR.PATCH | 错误修复 | 1.0.0 → 1.0.1 |

### 自动化版本管理
```yaml
version_management:
  # 依赖更新
  dependencies:
    tool: cargo-update
    schedule: "weekly"
    auto_merge: "patch"
    
  # 版本发布
  release:
    tool: cargo-release
    automation: true
    pre_release_checks: ["test", "lint", "security"]
```

## 技术栈维护策略

### 定期更新计划
| 组件类型 | 更新频率 | 更新方式 | 测试要求 |
|----------|----------|----------|----------|
| Rust 工具链 | 月度 | rustup update | 完整测试 |
| GitHub Actions | 季度 | 手动更新 | 回归测试 |
| 依赖包 | 周度 | cargo update | 单元测试 |
| 缓存策略 | 按需 | 配置更新 | 性能测试 |

### 回滚机制
```yaml
rollback_strategy:
  # 快速回滚
  fast_rollback:
    method: "git_revert"
    timeout: "10m"
    
  # 分支回滚
  branch_rollback:
    method: "branch_reset"
    backup: true
    
  # 配置回滚
  config_rollback:
    method: "config_restore"
    backup_location: "configs/backup/"
```

## 总结

本技术栈文档详细定义了 Rust MCP Servers 项目 GitHub Actions CI 构建系统的技术选择和配置策略。通过选择合适的工具版本、实施有效的缓存策略、建立完善的监控告警机制，可以显著提升构建效率和系统稳定性。

关键技术选择包括：
- **Rust 1.80+** 稳定版本确保兼容性和安全性
- **GitHub Actions v4** 获得最新功能和性能优化
- **三层缓存策略** 显著提升构建速度
- **并行构建** 最大化利用资源
- **全面监控** 及时发现和解决问题

这些技术选择基于最佳实践和项目实际需求，能够在保证构建质量的同时，提供高效的 CI/CD 流程。