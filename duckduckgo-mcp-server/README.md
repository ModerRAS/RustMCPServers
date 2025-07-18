# DuckDuckGo MCP Server

一个基于HTTP的DuckDuckGo搜索MCP服务器，使用Rust编写，支持标准MCP协议和认证功能。

## 功能特性

- 🔍 DuckDuckGo网页搜索
- 📰 DuckDuckGo新闻搜索
- 🔐 标准MCP认证协议支持
- 🌐 HTTP传输协议
- ⚡ 高性能异步处理
- 🛡️ JWT和静态令牌认证
- 📊 健康检查和监控

## 安装

### 从源码构建

```bash
# 克隆仓库
git clone <repository-url>
cd duckduckgo-mcp-server

# 构建项目
cargo build --release

# 运行服务器
./target/release/duckduckgo-mcp-server
```

## 使用方法

### 基本启动

```bash
# 默认配置：监听127.0.0.1:3000
./duckduckgo-mcp-server

# 自定义端口和主机
./duckduckgo-mcp-server --port 8080 --host 0.0.0.0

# 启用认证
./duckduckgo-mcp-server --require-auth --secret-key "your-secret-key"
```

### 命令行参数

```bash
duckduckgo-mcp-server [OPTIONS]

OPTIONS:
    -p, --port <PORT>          监听端口 [默认: 3000]
    -h, --host <HOST>          绑定主机 [默认: 127.0.0.1]
        --secret-key <KEY>     JWT密钥 [默认: your-secret-key-change-this]
        --require-auth         启用认证要求 [默认: false]
        --static-tokens <TOKENS>  静态API令牌（逗号分隔）
```

## API端点

### MCP协议端点

- `POST /mcp/initialize` - 初始化MCP连接
- `POST /mcp/tools/list` - 获取可用工具列表
- `POST /mcp/tools/call` - 调用工具
- `POST /mcp/ping` - 健康检查

### 认证端点

- `POST /auth/login` - 用户登录获取JWT令牌
- `POST /auth/validate` - 验证令牌有效性
- `POST /auth/tokens` - 添加静态令牌
- `POST /auth/tokens/remove` - 移除静态令牌

### 工具端点

#### 搜索工具

**工具名称**: `search`

**参数**:
```json
{
  "query": "搜索关键词",
  "max_results": 10,
  "region": "us",
  "time_filter": "d"
}
```

**示例请求**:
```bash
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "search",
      "arguments": {
        "query": "rust programming",
        "max_results": 5
      }
    }
  }'
```

#### 新闻搜索工具

**工具名称**: `search_news`

**参数**:
```json
{
  "query": "新闻关键词",
  "max_results": 10
}
```

## 认证配置

### JWT认证

1. 获取JWT令牌：
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password"}'
```

2. 使用令牌：
```bash
curl -H "Authorization: Bearer your-jwt-token" ...
```

### 静态令牌认证

1. 添加静态令牌：
```bash
curl -X POST http://localhost:3000/auth/tokens \
  -H "Content-Type: application/json" \
  -d '{"token": "your-static-token"}'
```

2. 使用令牌：
```bash
curl -H "X-API-Key: your-static-token" ...
```

## MCP配置示例

### VS Code Cline配置

在 `~/.vscode-server/data/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json` 中添加：

```json
{
  "mcpServers": {
    "duckduckgo": {
      "command": "node",
      "args": ["http://localhost:3000/mcp"],
      "transport": "http",
      "env": {
        "MCP_API_KEY": "your-api-token"
      }
    }
  }
}
```

### Claude Desktop配置

在 `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) 或相应配置文件中添加：

```json
{
  "mcpServers": {
    "duckduckgo": {
      "command": "http://localhost:3000/mcp",
      "transport": "http",
      "env": {
        "MCP_API_KEY": "your-api-token"
      }
    }
  }
}
```

## 开发

### 运行测试

```bash
cargo test
```

### 代码格式化

```bash
cargo fmt
```

### 代码检查

```bash
cargo clippy
```

## 环境变量

- `RUST_LOG`: 设置日志级别 (例如: `debug`, `info`, `warn`)
- `MCP_SECRET_KEY`: JWT密钥 (覆盖--secret-key参数)
- `MCP_REQUIRE_AUTH`: 是否要求认证 (覆盖--require-auth参数)
- `MCP_STATIC_TOKENS`: 静态API令牌 (覆盖--static-tokens参数)

## 许可证

MIT License
