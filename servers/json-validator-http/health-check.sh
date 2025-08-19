#!/bin/bash
# Health check script for JSON Validator HTTP Server

# Set strict error handling
set -euo pipefail

# Configuration
HEALTH_URL="${HEALTH_URL:-http://localhost:8080/health}"
METRICS_URL="${METRICS_URL:-http://localhost:9090/metrics}"
TIMEOUT="${TIMEOUT:-10}"
RETRIES="${RETRIES:-3}"

# Function to log messages
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1"
}

# Function to check HTTP endpoint
check_endpoint() {
    local url="$1"
    local description="$2"
    local attempt=1
    
    while [ $attempt -le $RETRIES ]; do
        if curl -f -s -m "$TIMEOUT" "$url" > /dev/null 2>&1; then
            log "$description check passed"
            return 0
        fi
        
        log "$description check attempt $attempt/$RETRIES failed"
        sleep 2
        attempt=$((attempt + 1))
    done
    
    log "$description check failed after $RETRIES attempts"
    return 1
}

# Function to check server process
check_process() {
    local pid_file="/var/run/json-validator/server.pid"
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            log "Server process is running (PID: $pid)"
            return 0
        else
            log "Server process is not running (stale PID file)"
            return 1
        fi
    else
        log "Server PID file not found"
        return 1
    fi
}

# Function to check memory usage
check_memory() {
    if command -v free >/dev/null 2>&1; then
        local memory_usage=$(free | grep Mem | awk '{printf "%.2f", $3/$2 * 100.0}')
        local memory_threshold=${MEMORY_THRESHOLD:-90}
        
        if (( $(echo "$memory_usage > $memory_threshold" | bc -l) )); then
            log "Memory usage too high: ${memory_usage}% (threshold: ${memory_threshold}%)"
            return 1
        else
            log "Memory usage is normal: ${memory_usage}%"
            return 0
        fi
    else
        log "Memory check skipped (free command not available)"
        return 0
    fi
}

# Function to check disk usage
check_disk() {
    if command -v df >/dev/null 2>&1; then
        local disk_usage=$(df /app | tail -1 | awk '{print $5}' | sed 's/%//')
        local disk_threshold=${DISK_THRESHOLD:-90}
        
        if [ "$disk_usage" -gt "$disk_threshold" ]; then
            log "Disk usage too high: ${disk_usage}% (threshold: ${disk_threshold}%)"
            return 1
        else
            log "Disk usage is normal: ${disk_usage}%"
            return 0
        fi
    else
        log "Disk check skipped (df command not available)"
        return 0
    fi
}

# Function to check log files
check_logs() {
    local log_dir="/var/log/json-validator"
    
    if [ -d "$log_dir" ]; then
        local log_count=$(find "$log_dir" -name "*.log" -type f | wc -l)
        local log_size=$(du -sh "$log_dir" 2>/dev/null | cut -f1)
        
        log "Log files: $log_count files, total size: $log_size"
        
        # Check for recent errors
        if find "$log_dir" -name "*.log" -mmin -5 -exec grep -l "ERROR\|FATAL" {} \; | grep -q .; then
            log "Recent errors found in log files"
            return 1
        fi
        
        return 0
    else
        log "Log directory not found: $log_dir"
        return 0
    fi
}

# Function to check SSL certificates
check_certificates() {
    local cert_dir="/app/certs"
    
    if [ -d "$cert_dir" ]; then
        if [ -f "$cert_dir/server.crt" ] && [ -f "$cert_dir/server.key" ]; then
            # Check certificate expiration
            if command -v openssl >/dev/null 2>&1; then
                local expiry_date=$(openssl x509 -enddate -noout -in "$cert_dir/server.crt" | cut -d= -f2)
                local expiry_timestamp=$(date -d "$expiry_date" +%s)
                local current_timestamp=$(date +%s)
                local days_until_expiry=$(( (expiry_timestamp - current_timestamp) / 86400 ))
                
                if [ "$days_until_expiry" -lt 30 ]; then
                    log "SSL certificate expires in $days_until_expiry days"
                    return 1
                else
                    log "SSL certificate is valid for $days_until_expiry more days"
                    return 0
                fi
            else
                log "SSL certificate check skipped (openssl not available)"
                return 0
            fi
        else
            log "SSL certificate files not found"
            return 1
        fi
    else
        log "Certificate directory not found: $cert_dir"
        return 0
    fi
}

# Main health check
main() {
    log "Starting health check..."
    
    local failed_checks=0
    
    # Check server process
    if ! check_process; then
        failed_checks=$((failed_checks + 1))
    fi
    
    # Check HTTP endpoints
    if ! check_endpoint "$HEALTH_URL" "Health endpoint"; then
        failed_checks=$((failed_checks + 1))
    fi
    
    if ! check_endpoint "$METRICS_URL" "Metrics endpoint"; then
        failed_checks=$((failed_checks + 1))
    fi
    
    # Check system resources
    if ! check_memory; then
        failed_checks=$((failed_checks + 1))
    fi
    
    if ! check_disk; then
        failed_checks=$((failed_checks + 1))
    fi
    
    # Check logs
    if ! check_logs; then
        failed_checks=$((failed_checks + 1))
    fi
    
    # Check certificates
    if ! check_certificates; then
        failed_checks=$((failed_checks + 1))
    fi
    
    # Determine overall health
    if [ "$failed_checks" -eq 0 ]; then
        log "Health check passed - all checks successful"
        exit 0
    else
        log "Health check failed - $failed_checks check(s) failed"
        exit 1
    fi
}

# Run main function
main "$@"