# GitHub Actions 测试套件

## 🎯 概述

本测试套件为Rust MCP服务器的GitHub Actions工作流提供全面的测试验证方案。它包括单元测试、集成测试、端到端测试、性能测试和安全测试，确保所有GitHub Actions修复都能通过严格的验证。

## 🏗️ 项目结构

```
tests/
├── src/                          # 测试源代码
│   ├── lib.rs                     # 测试库入口
│   ├── workflow_validator.rs      # 工作流验证器
│   ├── workflow_executor.rs       # 工作流执行器
│   ├── performance_tester.rs       # 性能测试器
│   ├── security_tester.rs         # 安全测试器
│   └── bin/                       # 可执行工具
│       ├── validate_workflow.rs    # 工作流验证工具
│       └── security_test.rs        # 安全测试工具
├── tests/                         # 测试文件
│   ├── unit_tests.rs              # 单元测试
│   ├── integration_tests.rs       # 集成测试
│   └── e2e_tests.rs               # 端到端测试
├── benches/                       # 性能基准测试
│   └── workflow_performance.rs    # 工作流性能基准
├── scripts/                       # 自动化脚本
│   └── run_tests.sh               # 测试运行脚本
├── fixtures/                      # 测试fixtures
│   ├── ci_workflow.yml           # CI工作流模板
│   ├── release_workflow.yml       # 发布工作流模板
│   └── security_workflow.yml     # 安全工作流模板
├── reports/                       # 测试报告目录
├── Cargo.toml                     # 测试项目配置
└── VALIDATION_CHECKLIST.md         # 验证检查清单
```

## 🚀 快速开始

### 环境要求
- Rust 1.70+
- Git
- 网络连接

### 安装和设置

1. **克隆项目**
```bash
git clone <repository-url>
cd RustMCPServers
```

2. **构建测试工具**
```bash
cd tests
cargo build --release
```

3. **运行完整测试套件**
```bash
./scripts/run_tests.sh
```

## 📋 测试类型

### 1. 单元测试
覆盖核心功能的独立测试：
- 工作流配置验证
- 安全规则检查
- 性能指标计算
- 错误处理机制

**运行命令:**
```bash
cd tests
cargo test --lib
```

### 2. 集成测试
验证组件间交互：
- 工作流执行流程
- 触发条件验证
- 矩阵构建配置
- 缓存机制测试

**运行命令:**
```bash
cd tests
cargo test --test integration_tests
```

### 3. 端到端测试
完整CI/CD流程验证：
- 完整的Git工作流模拟
- 多服务器发布流程
- 失败恢复机制
- 性能回归检测

**运行命令:**
```bash
cd tests
cargo test --test e2e_tests
```

### 4. 性能测试
性能基准和优化验证：
- 工作流验证性能
- 安全扫描性能
- 缓存效果测试
- 并发执行测试

**运行命令:**
```bash
cd tests
cargo bench
```

### 5. 安全测试
全面的安全漏洞检测：
- 密钥泄露检测
- 权限配置检查
- 依赖安全验证
- 代码注入检测

**运行命令:**
```bash
# 构建工具
cd tests
cargo build --release

# 运行安全测试
./target/release/security_test .github/workflows/ci.yml
```

## 🔧 工具使用

### 工作流验证工具
```bash
# 基本用法
./target/release/validate_workflow <workflow-file>

# 示例
./target/release/validate_workflow .github/workflows/ci.yml
./target/release/validate_workflow .github/workflows/release.yml
```

**输出:**
- 详细的验证报告
- 安全评分
- 性能指标
- 改进建议

### 安全测试工具
```bash
# 基本用法
./target/release/security_test <workflow-file> [--output-format <json|markdown>]

# 示例
./target/release/security_test .github/workflows/ci.yml --output-format json
./target/release/security_test .github/workflows/security-scan.yml --output-format markdown
```

**输出:**
- 漏洞详细报告
- 安全评分
- 合规性检查
- 修复建议

### 自动化测试脚本
```bash
# 完整测试套件
./scripts/run_tests.sh

# 单独测试类别
./scripts/run_tests.sh --unit-only
./scripts/run_tests.sh --integration-only
./scripts/run_tests.sh --e2e-only
./scripts/run_tests.sh --performance-only
./scripts/run_tests.sh --security-only
./scripts/run_tests.sh --validation-only

# 查看帮助
./scripts/run_tests.sh --help
```

## 📊 测试覆盖

### 功能覆盖
- ✅ YAML语法验证
- ✅ 工作流结构检查
- ✅ 触发条件验证
- ✅ 矩阵配置测试
- ✅ 安全漏洞检测
- ✅ 性能基准测试
- ✅ 缓存效果验证
- ✅ 错误处理测试
- ✅ 报告生成测试
- ✅ CI集成测试

### 安全覆盖
- ✅ 硬编码密钥检测
- ✅ 权限配置验证
- ✅ 依赖安全检查
- ✅ 代码注入检测
- ✅ 网络安全检查
- ✅ 不安全操作检测
- ✅ 过时组件检测

### 性能覆盖
- ✅ 执行时间基准
- ✅ 内存使用监控
- ✅ 缓存命中率
- ✅ 并发性能测试
- ✅ 负载测试
- ✅ 回归检测

## 📈 性能基准

### 目标性能指标
- **工作流验证**: < 1秒
- **安全测试**: < 2秒
- **端到端测试**: < 5分钟
- **内存使用**: < 100MB
- **缓存命中率**: > 80%

### 实际性能数据
```bash
# 运行性能测试
cd tests
cargo bench

# 查看详细报告
open target/criterion/
```

## 🔒 安全标准

### 安全评分标准
- **90-100分**: 优秀
- **80-89分**: 良好
- **70-79分**: 需要改进
- **< 70分**: 不安全

### 漏洞严重程度
- **Critical**: 必须立即修复
- **High**: 优先修复
- **Medium**: 计划修复
- **Low**: 建议修复
- **Info**: 参考信息

## 📋 验证检查清单

详细验证步骤请参考 [VALIDATION_CHECKLIST.md](./VALIDATION_CHECKLIST.md)

### 快速验证
1. 运行完整测试套件
2. 检查所有测试通过
3. 验证安全评分 ≥ 80
4. 确认性能指标达标
5. 检查报告生成正常

## 🤝 CI/CD集成

### GitHub Actions工作流
测试套件包含完整的GitHub Actions工作流配置：

- **`.github/workflows/test-suite.yml`**: 完整测试套件执行
- **自动化触发**: Push、Pull Request、定时任务
- **并行执行**: 多个测试类别并行运行
- **报告生成**: 自动生成和上传测试报告
- **PR评论**: 自动在PR中评论测试结果

### 集成步骤
1. 确保工作流文件在正确位置
2. 配置必要的Secrets和权限
3. 验证工作流可以正常触发
4. 检查测试报告生成和上传

## 🐛 故障排除

### 常见问题
1. **编译错误**
   ```bash
   # 清理构建缓存
   cargo clean
   
   # 更新依赖
   cargo update
   
   # 检查Rust版本
   rustc --version
   ```

2. **测试失败**
   ```bash
   # 详细日志
   RUST_LOG=debug cargo test -- --nocapture
   
   # 单独运行失败的测试
   cargo test test_name
   ```

3. **权限问题**
   ```bash
   # 检查文件权限
   ls -la scripts/
   
   # 设置执行权限
   chmod +x scripts/run_tests.sh
   ```

### 调试技巧
- 使用 `RUST_LOG=debug` 环境变量获取详细日志
- 使用 `--nocapture` 选项查看测试输出
- 检查 `target/` 目录下的生成文件
- 查看生成的测试报告

## 📚 API文档

### 核心结构体
```rust
pub struct WorkflowValidator {
    pub workflow_path: String,
    pub content: String,
}

pub struct SecurityTester {
    pub workflow_path: String,
    pub content: String,
}

pub struct PerformanceTester {
    pub test_runs: usize,
    pub concurrent_runs: usize,
}
```

### 主要方法
```rust
// 工作流验证
impl WorkflowValidator {
    pub fn new(workflow_path: &str) -> Result<Self, Box<dyn std::error::Error>>
    pub fn validate(&self) -> WorkflowValidationResult
}

// 安全测试
impl SecurityTester {
    pub fn new(workflow_path: &str) -> Result<Self, Box<dyn std::error::Error>>
    pub fn run_security_tests(&self) -> SecurityTestResult
}

// 性能测试
impl PerformanceTester {
    pub fn new(test_runs: usize, concurrent_runs: usize) -> Self
    pub async fn test_workflow_performance(&self, workflow_path: &str) -> Result<PerformanceTestResult, Box<dyn std::error::Error>>
}
```

## 🤝 贡献指南

### 添加新测试
1. 在相应的测试文件中添加测试用例
2. 确保测试覆盖新的功能点
3. 更新文档和示例
4. 运行完整测试套件验证

### 报告问题
1. 使用GitHub Issues报告问题
2. 提供详细的复现步骤
3. 包含错误日志和环境信息
4. 期望的行为和实际行为

### 开发环境
```bash
# 安装开发依赖
cargo install cargo-watch cargo-outdated

# 监视模式运行测试
cargo watch -x "test"

# 格式化代码
cargo fmt

# 检查代码质量
cargo clippy
```

## 📄 许可证

本项目采用MIT许可证。详情请参阅 [LICENSE](../LICENSE) 文件。

## 🙏 致谢

感谢所有为这个项目贡献的开发者和测试人员。

---

**注意**: 本测试套件专门为Rust MCP服务器的GitHub Actions工作流设计，请根据实际需求调整配置和测试用例。