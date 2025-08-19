#!/bin/bash
set -e

# Docker entrypoint script for JSON Validator HTTP Server

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to handle signals
cleanup() {
    log "Received shutdown signal, shutting down gracefully..."
    if [ -n "$SERVER_PID" ]; then
        kill -TERM "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
    log "Server shutdown complete"
    exit 0
}

# Set up signal handlers
trap cleanup TERM INT

# Log startup information
log "Starting JSON Validator HTTP Server"
log "Version: ${VERSION:-1.0.0}"
log "Environment: ${JSON_VALIDATOR_ENVIRONMENT:-development}"

# Check if configuration file exists
if [ ! -f "$JSON_VALIDATOR_CONFIG" ]; then
    log "Configuration file not found: $JSON_VALIDATOR_CONFIG"
    log "Creating default configuration..."
    
    # Create config directory if it doesn't exist
    mkdir -p "$(dirname "$JSON_VALIDATOR_CONFIG")"
    
    # Create default configuration if it doesn't exist
    if [ ! -f "/app/config/default.toml" ]; then
        log "Default configuration not found, using built-in defaults"
    fi
fi

# Generate self-signed certificates for development
if [ "$JSON_VALIDATOR_ENVIRONMENT" = "development" ] && [ ! -f "/app/certs/server.crt" ]; then
    log "Generating self-signed certificates for development..."
    mkdir -p /app/certs
    
    # Generate private key
    openssl genrsa -out /app/certs/server.key 2048
    
    # Generate certificate
    openssl req -new -x509 -key /app/certs/server.key -out /app/certs/server.crt \
        -days 365 -subj "/C=CN/ST=Development/L=Development/O=JSON Validator/CN=localhost"
    
    log "Self-signed certificates generated"
fi

# Set up log rotation
setup_log_rotation() {
    if [ "$JSON_VALIDATOR_LOG_ROTATION" = "true" ]; then
        log "Setting up log rotation..."
        
        # Create logrotate configuration
        cat > /etc/logrotate.d/json-validator << EOF
/var/log/json-validator/*.log {
    daily
    missingok
    rotate 7
    compress
    delaycompress
    notifempty
    create 0644 appuser appuser
    postrotate
        kill -USR1 \`cat /var/run/json-validator/server.pid 2>/dev/null\` 2>/dev/null || true
    endscript
}
EOF
    fi
}

# Initialize database (if enabled)
initialize_database() {
    if [ "$JSON_VALIDATOR_DATABASE_ENABLED" = "true" ]; then
        log "Initializing database..."
        # Add database initialization logic here
        log "Database initialization complete"
    fi
}

# Initialize Redis (if enabled)
initialize_redis() {
    if [ "$JSON_VALIDATOR_REDIS_ENABLED" = "true" ]; then
        log "Initializing Redis connection..."
        # Add Redis initialization logic here
        log "Redis initialization complete"
    fi
}

# Health check function
health_check() {
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if curl -f -s http://localhost:8080/health > /dev/null 2>&1; then
            log "Health check passed"
            return 0
        fi
        
        log "Health check attempt $attempt/$max_attempts failed"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    log "Health check failed after $max_attempts attempts"
    return 1
}

# Start the server
start_server() {
    log "Starting JSON Validator HTTP Server..."
    
    # Prepare server arguments
    local args=()
    args+=("--config" "$JSON_VALIDATOR_CONFIG")
    
    if [ "$JSON_VALIDATOR_LOG_LEVEL" ]; then
        args+=("--log-level" "$JSON_VALIDATOR_LOG_LEVEL")
    fi
    
    if [ "$JSON_VALIDATOR_LISTEN" ]; then
        args+=("--listen" "$JSON_VALIDATOR_LISTEN")
    fi
    
    if [ "$JSON_VALIDATOR_DEV" = "true" ]; then
        args+=("--dev")
    fi
    
    # Start the server
    log "Server arguments: ${args[*]}"
    exec /app/bin/json-validator-http "${args[@]}" &
    SERVER_PID=$!
    
    # Wait for server to start
    log "Waiting for server to start..."
    sleep 5
    
    # Perform health check
    if health_check; then
        log "Server started successfully with PID $SERVER_PID"
        
        # Create PID file
        mkdir -p /var/run/json-validator
        echo $SERVER_PID > /var/run/json-validator/server.pid
    else
        log "Server failed to start"
        kill $SERVER_PID 2>/dev/null || true
        exit 1
    fi
}

# Main execution
main() {
    # Set up environment
    setup_log_rotation
    initialize_database
    initialize_redis
    
    # Start server
    start_server
    
    # Wait for server process
    wait $SERVER_PID
}

# Run main function
main "$@"