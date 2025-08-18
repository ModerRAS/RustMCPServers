# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Structure

This is a Rust workspace containing a DuckDuckGo MCP server in the `duckduckgo-mcp-server/` directory.

## Quick Commands

### Development
```bash
# Navigate to server directory
cd duckduckgo-mcp-server

# Build and run
cargo run

# Run tests
cargo test

# Run specific test
cargo test test_search_request_building -- --nocapture

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build release
cargo build --release
```

### Docker
```bash
# Build Docker image
docker build -t duckduckgo-mcp-server ./duckduckgo-mcp-server

# Run with Docker Compose
docker-compose -f duckduckgo-mcp-server/docker-compose.yml up -d
```

## Architecture Overview

The DuckDuckGo MCP server follows a modular architecture:

- **Config**: Environment-based configuration (`src/config.rs`)
- **Client**: Enhanced DuckDuckGo client with caching (`src/client.rs`)
- **Auth**: JWT token and API key authentication (`src/auth.rs`)
- **MCP Handlers**: HTTP-based MCP protocol implementation (`src/mcp_handler.rs`)
- **DuckDuckGo**: Raw search functionality (`src/duckduckgo.rs`)
- **Routes**: Authentication endpoints (`src/auth_routes.rs`)

## Key Components

### MCP Protocol
- JSON-RPC 2.0 over HTTP
- Endpoints: `/mcp/initialize`, `/mcp/tools/list`, `/mcp/tools/call`, `/mcp/ping`
- Tools: `search`, `search_news`

### Configuration
All settings via environment variables (see README.md for full list)
Key variables: `HOST`, `PORT`, `SECRET_KEY`, `REQUIRE_AUTH`, `CACHE_TTL_SECONDS`

### Testing Structure
- Unit tests: `tests/unit_tests.rs`
- Integration tests: `tests/integration_tests.rs`
- Run with: `cargo test --test unit_tests` or `cargo test --test integration_tests`

## GitHub Actions

- **CI**: `.github/workflows/ci.yml` - runs tests, clippy, formatting
- **Docker**: `.github/workflows/docker.yml` - builds and publishes Docker images
- Both workflows run in `duckduckgo-mcp-server/` directory context

## Environment Setup

Copy `.env.example` to `.env` and configure:
```bash
cp duckduckgo-mcp-server/.env.example duckduckgo-mcp-server/.env
```

## Common Development Tasks

1. **Add new search tool**: Modify `src/client.rs` and `src/mcp_handler.rs`
2. **Update authentication**: Edit `src/auth.rs` and `src/auth_routes.rs`
3. **Change search parameters**: Update `src/duckduckgo.rs` and `src/mcp_types.rs`
4. **Add new configuration**: Extend `src/config.rs` and update environment variables

## Debug Commands
```bash
# Debug single test
cargo test test_config_from_env -- --nocapture

# Run with debug logging
RUST_LOG=debug cargo run

# Check specific file compilation
cargo check --bin duckduckgo-mcp-server
```

## 注意事项
- 根目录下的tmp文件夹是用来放一些参考用的临时文件的，这个文件夹是被gitignore掉的，所以如果你有需要参考的git仓库，或者是需要参考的示例文件之类的你可以clone或者下载到tmp文件夹里，然后用来参考。
