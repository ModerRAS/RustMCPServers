# DuckDuckGo MCP Server

A production-ready MCP (Model Context Protocol) server that provides DuckDuckGo search functionality via HTTP transport. Built with Rust for high performance and reliability.

## Features

- 🔍 **DuckDuckGo Search**: Web and news search capabilities
- 🚀 **High Performance**: Built with Rust for speed and efficiency
- 🔒 **Authentication**: JWT tokens and API key support
- 🚦 **Rate Limiting**: Configurable rate limits per client
- 🗄️ **Caching**: Built-in caching with configurable TTL
- 🐳 **Docker Ready**: Multi-stage Docker builds
- 🔍 **Comprehensive Logging**: Structured logging with configurable levels
- ⚙️ **Environment Configuration**: Full configuration via environment variables
- 🧪 **Test Coverage**: Comprehensive unit and integration tests

## Quick Start

### Using Docker

```bash
# Pull the latest image
docker pull ghcr.io/moderRAS/duckduckgo-mcp-server:latest

# Run with default settings
docker run -p 3000:3000 ghcr.io/moderRAS/duckduckgo-mcp-server:latest

# Run with custom configuration
docker run -p 3000:3000 \
  -e REQUIRE_AUTH=true \
  -e SECRET_KEY=your-secret-key \
  -e STATIC_TOKENS=token1,token2 \
  ghcr.io/moderRAS/duckduckgo-mcp-server:latest
```

### Using Docker Compose

```bash
# Clone the repository
git clone https://github.com/ModerRAS/RustMCPServers.git
cd duckduckgo-mcp-server

# Start with Docker Compose
docker-compose up -d
```

### From Source

```bash
# Clone the repository
git clone https://github.com/ModerRAS/RustMCPServers.git
cd duckduckgo-mcp-server

# Build and run
cargo build --release
./target/release/duckduckgo-mcp-server
```

## Configuration

All configuration is done through environment variables:

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

## API Endpoints

### MCP Protocol Endpoints

#### Initialize
```http
POST /mcp/initialize
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "test-client",
      "version": "1.0.0"
    }
  }
}
```

#### List Tools
```http
POST /mcp/tools/list
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list"
}
```

#### Call Tool
```http
POST /mcp/tools/call
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "search",
    "arguments": {
      "query": "rust programming language",
      "max_results": 5,
      "region": "us",
      "time_filter": "d"
    }
  }
}
```

#### Ping
```http
POST /mcp/ping
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "ping"
}
```

### Authentication Endpoints

#### Login
```http
POST /auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password"
}
```

#### Validate Token
```http
POST /auth/validate
Authorization: Bearer <your-token>
```

#### Create API Key
```http
POST /auth/api-keys
Authorization: Bearer <your-token>
Content-Type: application/json

{
  "name": "my-key",
  "scopes": ["read", "search"]
}
```

#### List API Keys
```http
GET /auth/api-keys
Authorization: Bearer <your-token>
```

#### Revoke API Key
```http
POST /auth/api-keys/<key-id>/revoke
Authorization: Bearer <your-token>
```

### Health Check
```http
GET /health
```

### Metrics
```http
GET /metrics
```

## Tools

### Search
Search DuckDuckGo for web results.

**Parameters:**
- `query` (string, required): Search query
- `max_results` (integer, optional): Maximum results (1-20, default: 10)
- `region` (string, optional): Region code (e.g., "us", "uk", "cn")
- `time_filter` (string, optional): Time filter ("d", "w", "m", "y")

### Search News
Search DuckDuckGo for news results.

**Parameters:**
- `query` (string, required): Search query
- `max_results` (integer, optional): Maximum results (1-20, default: 10)

## Authentication

The server supports two authentication methods:

### API Keys
Include your API key in the `X-API-Key` header:
```http
X-API-Key: your-api-key
```

### JWT Tokens
Obtain a token via login and include it in the `Authorization` header:
```http
Authorization: Bearer your-jwt-token
```

## Usage Examples

### Using curl

```bash
# Health check
curl http://localhost:3000/health

# MCP initialization
curl -X POST http://localhost:3000/mcp/initialize \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test-client","version":"1.0.0"}}}'

# Search using MCP
curl -X POST http://localhost:3000/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search","arguments":{"query":"rust programming","max_results":5}}}'
```

### Using JavaScript

```javascript
// MCP client example
const response = await fetch('http://localhost:3000/mcp/tools/call', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'tools/call',
    params: {
      name: 'search',
      arguments: {
        query: 'rust programming language',
        max_results: 5
      }
    }
  })
});

const result = await response.json();
console.log(result);
```

## Development

### Prerequisites
- Rust 1.75 or later
- Docker (optional)

### Setup
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone https://github.com/ModerRAS/RustMCPServers.git
cd duckduckgo-mcp-server

# Run tests
cargo test

# Run with development settings
cargo run
```

### Testing

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

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Docker build
docker build -t duckduckgo-mcp-server .
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
