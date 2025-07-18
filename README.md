# RustMCPServers

A collection of high-performance, production-ready MCP (Model Context Protocol) servers implemented in Rust. This workspace provides specialized MCP servers for various use cases, starting with the DuckDuckGo search server.

## 🚀 What's Inside

This repository contains:

- **DuckDuckGo MCP Server** (`duckduckgo-mcp-server/`) - A production-ready MCP server providing DuckDuckGo search capabilities

## 📋 MCP Protocol Support

All servers in this workspace implement the MCP (Model Context Protocol) specification:

- **Protocol Version**: 2024-11-05
- **Transport**: HTTP JSON-RPC 2.0
- **Authentication**: JWT tokens and API key support
- **Rate Limiting**: Configurable per-client rate limits
- **Caching**: Built-in response caching

## 🛠️ Quick Start

### Prerequisites

- **Rust 1.75+** (for building from source)
- **Docker** (optional, for containerized deployment)

### Getting Started

#### 1. DuckDuckGo MCP Server

The DuckDuckGo server provides web and news search capabilities via the MCP protocol.

**Using Docker:**
```bash
cd duckduckgo-mcp-server
docker build -t duckduckgo-mcp-server .
docker run -p 3000:3000 duckduckgo-mcp-server
```

**Using Docker Compose:**
```bash
cd duckduckgo-mcp-server
docker-compose up -d
```

**From Source:**
```bash
cd duckduckgo-mcp-server
cargo build --release
./target/release/duckduckgo-mcp-server
```

#### 2. Verify Installation

Check server health:
```bash
curl http://localhost:3000/health
```

Test MCP initialization:
```bash
curl -X POST http://localhost:3000/mcp/initialize \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'
```

## 📚 Repository Structure

```
RustMCPServers/
├── duckduckgo-mcp-server/          # DuckDuckGo search MCP server
│   ├── src/
│   │   ├── main.rs                 # Server entry point
│   │   ├── mcp_handler.rs          # MCP protocol handlers
│   │   ├── client.rs              # Enhanced DuckDuckGo client
│   │   ├── config.rs              # Environment configuration
│   │   ├── auth.rs                # JWT token authentication
│   │   ├── auth_routes.rs         # Authentication endpoints
│   │   ├── duckduckgo.rs          # Raw search functionality
│   │   └── mcp_types.rs           # MCP type definitions
│   ├── tests/
│   │   ├── unit_tests.rs          # Unit tests
│   │   └── integration_tests.rs   # Integration tests
│   ├── Dockerfile                 # Multi-stage Docker build
│   ├── docker-compose.yml         # Docker Compose configuration
│   └── README.md                  # Server-specific documentation
├── LICENSE                        # MIT License
└── README.md                      # This file
```

## 🔧 Configuration

Each server supports configuration via environment variables. For the DuckDuckGo server:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `127.0.0.1` | Server host address |
| `PORT` | `3000` | Server port |
| `LOG_LEVEL` | `info` | Logging level (debug, info, warn, error) |
| `SECRET_KEY` | `your-secret-key-change-this` | JWT secret key |
| `REQUIRE_AUTH` | `false` | Require authentication for all requests |
| `STATIC_TOKENS` | `` | Comma-separated static API tokens |
| `CORS_ORIGINS` | `*` | Comma-separated CORS origins |
| `RATE_LIMIT_PER_MINUTE` | `60` | Rate limit per client per minute |
| `CACHE_TTL_SECONDS` | `300` | Cache TTL in seconds |
| `MAX_SEARCH_RESULTS` | `20` | Maximum search results per request |
| `REQUEST_TIMEOUT_SECONDS` | `30` | HTTP request timeout |
| `MAX_RETRIES` | `3` | Maximum retries for failed requests |
| `RETRY_DELAY_MS` | `500` | Retry delay in milliseconds |

## 🔍 API Reference

### MCP Protocol Endpoints

All servers implement the standard MCP protocol endpoints:

- **POST /mcp/initialize** - Initialize MCP client connection
- **POST /mcp/tools/list** - List available tools
- **POST /mcp/tools/call** - Execute a tool call
- **POST /mcp/ping** - Health check for MCP clients

### DuckDuckGo Server Tools

#### Search Tool
Search DuckDuckGo for web results.

**Parameters:**
- `query` (string, required): Search query
- `max_results` (integer, optional): Maximum results (1-20, default: 10)
- `region` (string, optional): Region code (e.g., "us", "uk", "cn")
- `time_filter` (string, optional): Time filter ("d", "w", "m", "y")

#### Search News Tool
Search DuckDuckGo for news results.

**Parameters:**
- `query` (string, required): Search query
- `max_results` (integer, optional): Maximum results (1-20, default: 10)

## 🔐 Authentication

Servers support multiple authentication methods:

### API Keys
Include your API key in the `X-API-Key` header:
```bash
curl -H "X-API-Key: your-api-key" http://localhost:3000/mcp/tools/list
```

### JWT Tokens
Obtain a token via login and include it in the `Authorization` header:
```bash
curl -H "Authorization: Bearer your-jwt-token" http://localhost:3000/mcp/tools/list
```

## 🧪 Development

### Setup Development Environment

```bash
# Clone the repository
git clone https://github.com/ModerRAS/RustMCPServers.git
cd RustMCPServers

# Navigate to specific server
cd duckduckgo-mcp-server

# Install Rust dependencies
cargo build

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Development Commands

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Run specific test
cargo test test_search_request_building -- --nocapture

# Build release version
cargo build --release
```

### Docker Development

```bash
# Build development image
docker build -t duckduckgo-mcp-server:dev .

# Run with development settings
docker run -p 3000:3000 \
  -e LOG_LEVEL=debug \
  -e RUST_LOG=debug \
  duckduckgo-mcp-server:dev
```

## 🧪 Testing

Each server includes comprehensive test suites:

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --test unit_tests

# Run integration tests only
cargo test --test integration_tests

# Run with coverage
cargo tarpaulin --out Html
```

## 🐳 Docker Support

All servers include:
- Multi-stage Docker builds for optimized images
- Docker Compose configurations
- Health checks
- Proper signal handling for graceful shutdowns

## 🚀 Contributing

We welcome contributions! Here's how to get started:

1. **Fork the repository**
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes**
4. **Add tests** for new functionality
5. **Run the test suite** (`cargo test`)
6. **Format your code** (`cargo fmt`)
7. **Run the linter** (`cargo clippy`)
8. **Commit your changes** (`git commit -m 'Add amazing feature'`)
9. **Push to your branch** (`git push origin feature/amazing-feature`)
10. **Open a Pull Request**

### Adding New MCP Servers

To add a new MCP server to this workspace:

1. Create a new directory under the workspace root
2. Initialize a new Rust project: `cargo init --name your-server-name`
3. Add your server to the workspace `Cargo.toml`
4. Follow the MCP protocol specification
5. Include comprehensive tests and documentation
6. Add Docker support

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🔗 Links

- [MCP Specification](https://modelcontextprotocol.io/)
- [DuckDuckGo API Documentation](https://duckduckgo.com/api)
- [Rust Documentation](https://doc.rust-lang.org/)

## 🆘 Support

If you encounter any issues or have questions:

1. Check the [Issues](https://github.com/ModerRAS/RustMCPServers/issues) page
2. Create a new issue with detailed information
3. Include relevant logs and error messages

---

**Built with ❤️ using Rust**