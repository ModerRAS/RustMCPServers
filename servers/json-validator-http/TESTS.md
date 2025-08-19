# 测试说明

## 测试概述

本项目包含了全面的测试套件，涵盖了单元测试、集成测试、性能测试和基准测试。

## 测试类型

### 1. 单元测试
- **位置**: `src/` 目录各模块中的 `#[test]` 和 `#[tokio::test]` 函数
- **覆盖范围**: 各个模块的核心功能
- **运行命令**: `cargo test --lib`

### 2. 集成测试
- **位置**: `tests/integration_tests.rs`
- **覆盖范围**: 完整的HTTP API功能测试
- **运行命令**: `cargo test --test integration_tests`

### 3. 性能测试
- **位置**: `tests/performance_tests.rs`
- **覆盖范围**: 性能优化模块的功能测试
- **运行命令**: `cargo test --test performance_tests`

### 4. 基准测试
- **位置**: `benches/performance.rs`
- **覆盖范围**: 性能基准测试
- **运行命令**: `cargo bench`

### 5. 文档测试
- **位置**: 代码文档中的示例代码
- **覆盖范围**: 文档中的代码示例
- **运行命令**: `cargo test --doc`

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试
```bash
# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test integration_tests

# 运行性能测试
cargo test --test performance_tests

# 运行基准测试
cargo bench
```

### 运行测试并生成覆盖率报告
```bash
# 安装 cargo-tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage/
```

## 测试配置

### 测试环境配置
- 测试使用 `config/test.toml` 配置文件
- 自动选择可用端口
- 使用测试专用的API密钥
- 启用调试模式

### 测试数据
- 使用小型JSON数据集进行快速测试
- 包含边界情况测试
- 包含错误处理测试

## 测试覆盖率目标

- **单元测试覆盖率**: 90%+
- **集成测试覆盖率**: 85%+
- **性能测试覆盖率**: 80%+
- **整体测试覆盖率**: 85%+

## 测试最佳实践

1. **测试独立性**: 每个测试都应该独立运行
2. **测试隔离**: 使用测试专用配置和数据
3. **异步测试**: 所有异步操作都使用 `tokio::test`
4. **错误处理**: 测试包含错误情况的处理
5. **性能考虑**: 测试不应该因为性能问题而失败

## 测试文件结构

```
tests/
├── integration_tests.rs      # 集成测试
├── performance_tests.rs      # 性能测试
└── ...

benches/
└── performance.rs            # 基准测试

src/
├── *.rs                      # 包含单元测试
└── ...

config/
└── test.toml                 # 测试配置
```

## 持续集成

测试配置为在以下环境中运行：
- Linux (x86_64)
- macOS (x86_64, ARM64)
- Windows (x86_64)

## 故障排除

### 常见问题

1. **端口冲突**: 测试使用端口0，让系统自动分配
2. **权限问题**: 确保有写入测试目录的权限
3. **依赖问题**: 运行 `cargo build` 确保所有依赖都已构建

### 调试测试

```bash
# 启用详细输出
cargo test -- --nocapture

# 只运行失败的测试
cargo test -- --exact --nocapture

# 过滤特定测试
cargo test test_name
```

## 贡献指南

添加新测试时，请遵循以下原则：
1. 测试应该快速运行
2. 测试应该有明确的断言
3. 测试应该覆盖正常和异常情况
4. 测试应该有适当的文档