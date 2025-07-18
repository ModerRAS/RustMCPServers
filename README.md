# DuckDuckGoMCP-rs

一个高性能的 DuckDuckGo MCP（模型上下文协议）服务器的 Rust 实现，提供快速、可靠的搜索功能，支持缓存和身份验证。

[![CI](https://github.com/ModerRAS/RustMCPServers/workflows/CI/badge.svg)](https://github.com/ModerRAS/RustMCPServers/actions)
[![Docker](https://github.com/ModerRAS/RustMCPServers/workflows/Docker/badge.svg)](https://github.com/ModerRAS/RustMCPServers/actions)

## 🚀 快速开始

### 使用 Docker（推荐）

```bash
# 拉取并运行最新镜像
docker run -d \
  -p 8080:8080 \
  -e SECRET_KEY=your-secret-key \
  --name duckduckgo-mcp-server \
  moder/duckduckgo-mcp-server:latest

# 或者使用 Docker Compose
docker-compose -f duckduckgo-mcp-server/docker-compose.yml up -d
```

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/ModerRAS/RustMCPServers.git
cd RustMCPServers/duckduckgo-mcp-server

# 构建并运行
cargo run --release
```

## 📋 前置要求

- **Rust**: 最新稳定版本 (1.70+)
- **Docker**: 20.10+（可选，用于容器化部署）
- **系统**: Linux, macOS, 或 Windows

## 🔧 配置选项

所有配置都通过环境变量进行：

| 变量名 | 默认值 | 描述 |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | 服务器绑定地址 |
| `PORT` | `8080` | 服务器端口 |
| `SECRET_KEY` | - | JWT 密钥（需要身份验证时必填） |
| `REQUIRE_AUTH` | `false` | 启用身份验证 |
| `CACHE_TTL_SECONDS` | `3600` | 缓存过期时间（秒） |
| `MAX_CACHE_SIZE_MB` | `100` | 最大缓存大小（MB） |

### 环境设置

```bash
# 复制示例环境文件
cp .env.example .env

# 编辑配置
nano .env
```

## 🏗️ 项目结构

```
RustMCPServers/
├── duckduckgo-mcp-server/          # DuckDuckGo MCP 服务器
│   ├── src/
│   │   ├── main.rs                 # 程序入口
│   │   ├── config.rs               # 配置管理
│   │   ├── client.rs               # MCP 客户端实现
│   │   ├── mcp_handler.rs          # MCP 协议处理器
│   │   ├── mcp_types.rs            # MCP 数据类型
│   │   ├── duckduckgo.rs           # 搜索功能
│   │   ├── auth.rs                 # 身份验证逻辑
│   │   └── auth_routes.rs          # 身份验证端点
│   ├── tests/
│   │   ├── unit_tests.rs           # 单元测试
│   │   └── integration_tests.rs    # 集成测试
│   ├── Cargo.toml                  # 依赖配置
│   └── Dockerfile                  # 容器配置
├── docs/
│   └── README_EN.md               # 英文文档
├── LICENSE                        # 许可证
└── README.md                      # 本文件
```

## 🔌 API 参考

### MCP 协议端点

服务器通过 HTTP 实现模型上下文协议（MCP）：

- **初始化**: `POST /mcp/initialize`
- **工具列表**: `POST /mcp/tools/list`
- **调用工具**: `POST /mcp/tools/call`
- **健康检查**: `GET /mcp/ping`

### 可用工具

#### 1. 搜索
使用 DuckDuckGo 进行网页搜索。

**请求:**
```json
{
  "tool": "search",
  "arguments": {
    "query": "rust 编程语言",
    "max_results": 5
  }
}
```

**响应:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "搜索结果..."
    }
  ]
}
```

#### 2. 新闻搜索
使用 DuckDuckGo 新闻进行新闻搜索。

**请求:**
```json
{
  "tool": "search_news",
  "arguments": {
    "query": "科技",
    "max_results": 10
  }
}
```

## 🔐 身份验证

### JWT 令牌验证

生成令牌：
```bash
curl -X POST http://localhost:8080/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "你的-api-密钥"}'
```

使用令牌：
```bash
curl -X POST http://localhost:8080/mcp/tools/call \
  -H "Authorization: Bearer <令牌>" \
  -H "Content-Type: application/json" \
  -d '{"tool": "search", "arguments": {"query": "测试"}}'
```

### API 密钥验证

直接使用 API 密钥：
```bash
curl -X POST http://localhost:8080/mcp/tools/call \
  -H "X-API-Key: 你的-api-密钥" \
  -H "Content-Type: application/json" \
  -d '{"tool": "search", "arguments": {"query": "测试"}}'
```

## 🧪 开发

### 设置开发环境

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆和设置
git clone https://github.com/ModerRAS/RustMCPServers.git
cd RustMCPServers/duckduckgo-mcp-server

# 运行测试
cargo test

# 使用调试日志运行
RUST_LOG=debug cargo run
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_search_request_building -- --nocapture

# 运行集成测试
cargo test --test integration_tests

# 运行单元测试
cargo test --test unit_tests
```

### 代码质量

```bash
# 格式化代码
cargo fmt --all

# 运行代码检查器
cargo clippy --all-targets --all-features -- -D warnings

# 检查格式化
cargo fmt --all -- --check
```

## 🐳 Docker 支持

### 构建自定义镜像

```bash
# 从源码构建
docker build -t duckduckgo-mcp-server ./duckduckgo-mcp-server

# 运行自定义镜像
docker run -d -p 8080:8080 --env-file .env duckduckgo-mcp-server
```

### Docker Compose

```yaml
version: '3.8'
services:
  duckduckgo-mcp-server:
    build: ./duckduckgo-mcp-server
    ports:
      - "8080:8080"
    environment:
      - SECRET_KEY=你的密钥
      - REQUIRE_AUTH=true
    restart: unless-stopped
```

## 🤝 贡献

1. Fork 这个仓库
2. 创建功能分支 (`git checkout -b feature/令人惊奇的功能`)
3. 提交你的更改 (`git commit -m '添加令人惊奇的功能'`)
4. 推送到分支 (`git push origin feature/令人惊奇的功能`)
5. 打开一个 Pull Request

### 开发工作流

- 遵循 Rust 编码标准
- 为新功能添加测试
- 按需更新文档
- 确保所有测试通过
- 运行 clippy 和格式化检查

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [DuckDuckGo](https://duckduckgo.com) 提供搜索 API
- [模型上下文协议](https://modelcontextprotocol.io) 提供 MCP 规范
- Rust 社区提供优秀的工具和库

## 🔗 链接

- [DuckDuckGo 隐私政策](https://duckduckgo.com/privacy)
- [MCP 规范](https://modelcontextprotocol.io)
- [Rust 文档](https://doc.rust-lang.org/)

## 🌐 语言

- **中文**: 本文件 (README.md)
- **English**: [docs/README_EN.md](docs/README_EN.md)

---

**注意**: 这是项目的中文文档。如需英文版本，请查看 [docs/README_EN.md](docs/README_EN.md)。