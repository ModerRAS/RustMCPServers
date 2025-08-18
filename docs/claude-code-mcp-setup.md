# Claude Code MCP服务器配置指南

本文档介绍如何在Claude Code中配置和连接MCP服务器。

## 远程服务器连接

### HTTP传输方式
```bash
claude mcp add --transport http context7 https://mcp.context7.com/mcp
```

### SSE传输方式
```bash
claude mcp add --transport sse context7 https://mcp.context7.com/sse
```

## 本地服务器连接

### 使用npx运行本地服务器
```bash
claude mcp add context7 -- npx -y @upstash/context7-mcp
```

### 使用二进制文件运行本地服务器
```bash
claude mcp add my-server -- /path/to/your/server-binary
```

### 使用Cargo运行Rust服务器
```bash
claude mcp add my-rust-server -- cargo run --bin my-server
```

## 配置说明

- `--transport`: 指定传输协议（http、sse、stdio等）
- 服务器名称: 用于在Claude Code中标识服务器的唯一名称
- 连接参数: 根据不同的传输方式提供相应的连接参数

## 常见问题

### 1. 如何查看已配置的MCP服务器？
```bash
claude mcp list
```

### 2. 如何删除已配置的MCP服务器？
```bash
claude mcp remove <server-name>
```

### 3. 如何测试MCP服务器连接？
使用Claude Code的交互模式，检查工具列表中是否包含该服务器的工具。

## 注意事项

- 确保服务器地址可访问
- 本地服务器需要保持运行状态
- 远程服务器需要正确的认证信息（如需要）
- 服务器名称需要唯一，避免冲突