# DuckDuckGoMCP-rs

A high-performance Rust implementation of the DuckDuckGo MCP (Model Context Protocol) server that provides fast, reliable search capabilities with caching and authentication.

## 🚀 Quick Start

### Using Docker (Recommended)

```bash
# Pull and run the latest image
docker run -d \
  -p 8080:8080 \
  -e SECRET_KEY=your-secret-key \
  --name duckduckgo-mcp-server \
  moder/duckduckgo-mcp-server:latest

# Or use Docker Compose
docker-compose -f duckduckgo-mcp-server/docker-compose.yml up -d
```

### From Source

```bash
# Clone the repository
git clone https://github.com/ModerRAS/RustMCPServers.git
cd RustMCPServers/duckduckgo-mcp-server

# Build and run
cargo run --release
```

## 📋 Prerequisites

- **Rust**: Latest stable version (1.70+)
- **Docker**: 20.10+ (optional, for containerized deployment)
- **System**: Linux, macOS, or Windows

## 🔧 Configuration

All configuration is done through environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `HOST` | `0.0.0.0` | Server bind address |
| `PORT` | `8080` | Server port |
| `SECRET_KEY` | - | JWT secret key (required for auth) |
| `REQUIRE_AUTH` | `false` | Enable authentication |
| `CACHE_TTL_SECONDS` | `3600` | Cache expiration time |
| `MAX_CACHE_SIZE_MB` | `100` | Maximum cache size in MB |

### Environment Setup

```bash
# Copy example environment file
cp .env.example .env

# Edit configuration
nano .env
```

## 🏗️ Repository Structure

```
RustMCPServers/
├── duckduckgo-mcp-server/
│   ├── src/
│   │   ├── main.rs          # Entry point
│   │   ├── config.rs        # Configuration management
│   │   ├── client.rs        # MCP client implementation
│   │   ├── mcp_handler.rs   # MCP protocol handlers
│   │   ├── mcp_types.rs     # MCP data types
│   │   ├── duckduckgo.rs    # Search functionality
│   │   ├── auth.rs          # Authentication logic
│   │   └── auth_routes.rs   # Auth endpoints
│   ├── tests/
│   │   ├── unit_tests.rs    # Unit tests
│   │   └── integration_tests.rs # Integration tests
│   ├── Cargo.toml           # Dependencies
│   └── Dockerfile           # Container configuration
└── docs/
    └── README_EN.md         # This file
```

## 🔌 API Reference

### MCP Protocol Endpoints

The server implements the Model Context Protocol (MCP) over HTTP:

- **Initialize**: `POST /mcp/initialize`
- **List Tools**: `POST /mcp/tools/list`
- **Call Tool**: `POST /mcp/tools/call`
- **Health Check**: `GET /mcp/ping`

### Available Tools

#### 1. Search
Search the web using DuckDuckGo.

**Request:**
```json
{
  "tool": "search",
  "arguments": {
    "query": "rust programming language",
    "max_results": 5
  }
}
```

**Response:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "Search results..."
    }
  ]
}
```

#### 2. Search News
Search for news using DuckDuckGo News.

**Request:**
```json
{
  "tool": "search_news",
  "arguments": {
    "query": "technology",
    "max_results": 10
  }
}
```

## 🔐 Authentication

### JWT Token Authentication

Generate a token:
```bash
curl -X POST http://localhost:8080/auth/token \
  -H "Content-Type: application/json" \
  -d '{"api_key": "your-api-key"}'
```

Use the token:
```bash
curl -X POST http://localhost:8080/mcp/tools/call \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"tool": "search", "arguments": {"query": "test"}}'
```

### API Key Authentication

Direct API key usage:
```bash
curl -X POST http://localhost:8080/mcp/tools/call \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{"tool": "search", "arguments": {"query": "test"}}'
```

## 🧪 Development

### Setup Development Environment

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and setup
git clone https://github.com/ModerRAS/RustMCPServers.git
cd RustMCPServers/duckduckgo-mcp-server

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_search_request_building -- --nocapture

# Run integration tests
cargo test --test integration_tests

# Run unit tests
cargo test --test unit_tests
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check
```

## 🐳 Docker Support

### Build Custom Image

```bash
# Build from source
docker build -t duckduckgo-mcp-server ./duckduckgo-mcp-server

# Run custom image
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
      - SECRET_KEY=your-secret-key
      - REQUIRE_AUTH=true
    restart: unless-stopped
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Workflow

- Follow Rust coding standards
- Add tests for new functionality
- Update documentation as needed
- Ensure all tests pass
- Run clippy and formatting checks

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](../LICENSE) file for details.

## 🙏 Acknowledgments

- [DuckDuckGo](https://duckduckgo.com) for providing the search API
- [Model Context Protocol](https://modelcontextprotocol.io) for the MCP specification
- Rust community for excellent tooling and libraries

## 🔗 Links

- [DuckDuckGo Privacy Policy](https://duckduckgo.com/privacy)
- [MCP Specification](https://modelcontextprotocol.io)
- [Rust Documentation](https://doc.rust-lang.org/)

---

**Note**: This is the English version of the README. For the Chinese version, see [README.md](../README.md).