# RustMCPServers

一个用于开发和部署Rust编写的MCP（模型上下文协议）服务器的工作空间。

[![CI](https://github.com/ModerRAS/RustMCPServers/workflows/CI/badge.svg)](https://github.com/ModerRAS/RustMCPServers/actions)

## 🚀 项目状态

**这是一个新的项目仓库，正在开发中。**

## 📋 项目结构

```
RustMCPServers/
├── crates/                         # 共享库
│   ├── common/                     # 通用工具和类型（待开发）
│   └── mcp-core/                   # MCP核心功能（待开发）
├── servers/                        # MCP服务器实现
│   └── (待添加服务器)
├── examples/                       # 示例代码（待开发）
├── docs/                          # 文档（待开发）
├── Cargo.toml                     # Workspace配置
├── Cargo.lock                     # 依赖锁定文件
├── LICENSE                        # 许可证
├── README.md                      # 本文件
└── .github/workflows/             # CI/CD配置
    ├── ci.yml                     # 持续集成
    └── claude.yml                 # Claude Code配置
```

## 🔧 前置要求

- **Rust**: 最新稳定版本 (1.70+)
- **系统**: Linux, macOS, 或 Windows

## 🛠️ 开发环境设置

```bash
# 克隆仓库
git clone https://github.com/ModerRAS/RustMCPServers.git
cd RustMCPServers

# 构建整个workspace
cargo build

# 运行测试
cargo test

# 格式化代码
cargo fmt --all

# 代码检查
cargo clippy --all-targets --all-features -- -D warnings
```

## 📝 计划功能

- [ ] 基础MCP协议实现
- [ ] 通用工具库
- [ ] 示例MCP服务器
- [ ] 文档和示例
- [ ] Docker支持
- [ ] 更多MCP服务器实现

## 🤝 贡献

欢迎贡献代码！请先阅读贡献指南。

1. Fork 这个仓库
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交你的更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开一个 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。