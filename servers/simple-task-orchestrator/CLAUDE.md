# Simple Task Orchestrator MCP Server

## 项目简介
这是一个基于Rust的MCP (Model Context Protocol) 服务器，提供任务编排和执行功能。

## 开发状态
- ✅ 基础MCP服务器框架完成
- ✅ 任务创建、执行、监控功能实现
- ✅ ClaudeCode执行器集成
- ✅ PM2进程管理配置
- ✅ 完整的测试套件
- ✅ 完善的代码文档注释 (2025-08-22)

## 可用工具
- `create_task` - 创建新任务
- `get_task` - 获取任务信息
- `acquire_task` - 获取待处理任务
- `execute_task` - 执行任务
- `complete_task` - 完成任务
- `list_tasks` - 列出任务
- `get_statistics` - 获取统计信息

## 快速启动
```bash
# 编译项目
cargo build --release

# 启动服务器
cargo run

# 使用PM2管理（推荐）
pm2 start mcp_server --name "simple-task-orchestrator"
pm2 status
pm2 logs simple-task-orchestrator
```

## 测试
```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_task_creation -- --nocapture

# 性能测试
cargo bench
```

## MCP Inspector测试
1. 安装MCP Inspector:
```bash
npx @modelcontextprotocol/inspector
```

2. 在Inspector中输入服务器命令:
```
/root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server
```

## 重要注意事项
- MCP服务器需要正确的初始化流程：initialize请求 → initialized通知 → 正常使用
- 服务器支持多种执行模式：Standard（标准执行）、ClaudeCode（AI辅助执行）
- 任务具有完整的生命周期管理：创建 → 获取 → 执行 → 完成
- 支持任务重试机制和错误处理

## 参考文档
- Rust MCP SDK: `/root/WorkSpace/Rust/RustMCPServers/tmp/rust-sdk/`
- MCP协议规范: https://modelcontextprotocol.io/
- ClaudeCode集成: 参考`docs/CLAUDE-CODE-INTEGRATION.md`