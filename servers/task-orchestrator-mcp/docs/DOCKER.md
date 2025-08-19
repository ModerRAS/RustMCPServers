# Task Orchestrator MCP Server - Docker Deployment

## Quick Start

### Using Docker

```bash
# Pull the image
docker pull ghcr.io/moderras/rustmcpservers:latest

# Run the container
docker run -d \
  --name task-orchestrator \
  -p 8080:8080 \
  -v $(pwd)/config.toml:/app/config.toml \
  ghcr.io/moderras/rustmcpservers:latest
```

### Using Docker Compose

```yaml
version: '3.8'

services:
  task-orchestrator:
    image: ghcr.io/moderras/rustmcpservers:latest
    ports:
      - "8080:8080"
    volumes:
      - ./config.toml:/app/config.toml
      - ./data:/app/data
    environment:
      - RUST_LOG=info
      - RUST_BACKTRACE=1
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## Configuration

The server can be configured through:

1. **Environment Variables**:
   ```bash
   docker run -e SERVER_HOST=0.0.0.0 -e SERVER_PORT=8080 ...
   ```

2. **Configuration File** (mounted volume):
   ```bash
   docker run -v $(pwd)/config.toml:/app/config.toml ...
   ```

## Available Endpoints

- `GET /health` - Health check
- `POST /` - MCP JSON-RPC over HTTP
- `GET /` - MCP SSE stream

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_HOST` | Server bind address | `127.0.0.1` |
| `SERVER_PORT` | Server port | `8080` |
| `RUST_LOG` | Log level | `info` |
| `RUST_BACKTRACE` | Enable backtrace | `1` |

## Image Tags

- `latest` - Latest stable version
- `v1.0.0` - Specific version
- `v1.0` - Major.Minor version
- `v1` - Major version

## Health Check

The container includes a built-in health check that verifies the application is responding on `/health` endpoint.

## Security

- Runs as non-root user
- Uses multi-stage build for minimal image size
- Regular security updates through base images