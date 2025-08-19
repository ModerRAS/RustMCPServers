# Multi-stage build for JSON Validator HTTP Server - Enterprise Edition
FROM rust:1.75-slim AS builder

# Set working directory
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy project files
COPY . .

# Copy workspace Cargo.toml if it exists
COPY ../../Cargo.toml ./workspace.Cargo.toml 2>/dev/null || true
COPY ../../Cargo.lock ./workspace.Cargo.lock 2>/dev/null || true

# Build with all features for production
RUN cargo build --release --features production

# Runtime stage
FROM debian:12-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    wget \
    dumb-init \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Create directories
RUN mkdir -p /app/config /app/logs /app/certs /var/run/json-validator /var/log/json-validator && \
    chown -R appuser:appuser /app /var/run/json-validator /var/log/json-validator

# Copy binary from builder stage
COPY --from=builder /app/target/release/json-validator-http /app/bin/json-validator-http

# Copy configuration files
COPY --from=builder /app/config/default.toml /app/config/default.toml

# Copy scripts and additional files
COPY --from=builder /app/docker-entrypoint.sh /app/docker-entrypoint.sh
COPY --from=builder /app/health-check.sh /app/health-check.sh

# Make scripts executable
RUN chmod +x /app/docker-entrypoint.sh /app/health-check.sh

# Set working directory
WORKDIR /app

# Switch to non-root user
USER appuser

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD ["/app/health-check.sh"]

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1
ENV JSON_VALIDATOR_CONFIG=/app/config/default.toml
ENV JSON_VALIDATOR_ENVIRONMENT=production

# Set labels
LABEL maintainer="RustMCPServers Team <rustmcpservers@example.com>"
LABEL version="1.0.0"
LABEL description="Enterprise JSON Validator HTTP Server"
LABEL org.opencontainers.image.source="https://github.com/RustMCPServers/RustMCPServers"

# Create volumes for persistent data
VOLUME ["/app/config", "/app/logs", "/app/certs"]

# Run the application
ENTRYPOINT ["/app/docker-entrypoint.sh"]
CMD ["/app/bin/json-validator-http", "--config", "/app/config/default.toml"]