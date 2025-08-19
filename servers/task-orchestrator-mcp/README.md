# Task Orchestrator MCP Server - 简化版本

这是一个基于rmcp的HTTP协议MCP服务器，提供任务调度功能。

## 🚀 特性

- **HTTP传输**: 基于rmcp的HTTP协议MCP服务器
- **内存存储**: 使用内存存储任务数据（适合演示）
- **任务调度**: 支持任务创建、获取、执行、完成等完整生命周期
- **优先级管理**: 支持任务优先级和状态管理
- **重试机制**: 内置任务重试和错误处理
- **完整工具**: 提供所有必要的MCP工具

## 🚀 快速开始

### 1. 编译和运行

```bash
# 编译项目
cargo build --release

# 运行服务器
cargo run --release
```

### 2. 验证服务

```bash
# 健康检查
curl http://localhost:8080/health

# 获取MCP工具列表
curl -X POST http://localhost:8080/ \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

## 📖 MCP工具

### create_task
创建新任务
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_task",
    "arguments": {
      "work_directory": "/path/to/work",
      "prompt": "Task description",
      "priority": "high",
      "tags": ["urgent", "production"],
      "max_retries": 3,
      "timeout_seconds": 3600
    }
  }
}
```

### get_task
获取任务详情
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_task",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

### acquire_task
获取待处理任务
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "acquire_task",
    "arguments": {
      "worker_id": "worker-1",
      "work_directory": "/path/to/work"
    }
  }
}
```

### complete_task
完成任务
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "complete_task",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000",
      "status": "success",
      "output": "Task completed successfully",
      "duration_ms": 1500,
      "metadata": {
        "additional_info": "value"
      }
    }
  }
}
```

### list_tasks
列出任务
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "list_tasks",
    "arguments": {
      "status": "waiting",
      "priority": "high",
      "limit": 10,
      "offset": 0
    }
  }
}
```

### get_statistics
获取统计信息
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "get_statistics",
    "arguments": {}
  }
}
```

### retry_task
重试任务
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "retry_task",
    "arguments": {
      "task_id": "550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

## ⚙️ 配置

### 环境变量

| 变量名 | 描述 | 默认值 |
|--------|------|--------|
| `SERVER_HOST` | 服务器地址 | `127.0.0.1` |
| `SERVER_PORT` | 服务器端口 | `8080` |
| `RUST_LOG` | 日志级别 | `info` |
| `LOG_FORMAT` | 日志格式 | `pretty` |

### 配置文件

配置文件 `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080
timeout_seconds = 30

[logging]
level = "info"
format = "pretty"

[task]
max_concurrent_tasks = 10
max_retries = 3
```

## 🤝 Claude Code集成

### 配置Claude Code

在Claude Code中配置MCP服务器：

```json
{
  "mcpServers": {
    "task-orchestrator": {
      "command": "curl",
      "args": [
        "-X", "POST",
        "http://localhost:8080/",
        "-H", "Content-Type: application/json",
        "-d", "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{\"tools\":{}},\"clientInfo\":{\"name\":\"claude-code\",\"version\":\"1.0.0\"}}}"
      ]
    }
  }
}
```

### 使用示例

在Claude Code中使用任务调度器：

```javascript
// 创建任务
const task = await mcp.callTool('create_task', {
  work_directory: '/tmp/project',
  prompt: 'Analyze the codebase and create documentation',
  priority: 'high'
});

// 获取任务
const taskDetails = await mcp.callTool('get_task', {
  task_id: task.task_id
});

// 完成任务
await mcp.callTool('complete_task', {
  task_id: task.task_id,
  status: 'success',
  output: 'Documentation created successfully'
});
```

## 📄 许可证

本项目采用 MIT 许可证。

## 🙏 致谢

- [Rust](https://www.rust-lang.org/) - 编程语言
- [rmcp](https://github.com/modelcontextprotocol/rust-sdk) - MCP Rust SDK
- [Axum](https://github.com/tokio-rs/axum) - Web框架
- [Tokio](https://tokio.rs/) - 异步运行时

---

**Task Orchestrator MCP Server** - 基于HTTP协议的任务调度MCP服务器！🚀