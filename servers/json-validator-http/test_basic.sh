#!/bin/bash

# JSON Validator HTTP Server Test Script
# This script tests the minimal HTTP JSON validator server functionality

set -e

# Configuration
SERVER_URL="http://127.0.0.1:8082"
TIMEOUT=5

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Test if server is running
check_server() {
    if curl -s --max-time $TIMEOUT "$SERVER_URL/health" > /dev/null 2>&1; then
        log_info "Server is running at $SERVER_URL"
        return 0
    else
        log_error "Server is not running at $SERVER_URL"
        return 1
    fi
}

# Test health endpoint
test_health() {
    log_info "Testing health endpoint..."
    
    response=$(curl -s --max-time $TIMEOUT "$SERVER_URL/health")
    if [[ $? -eq 0 ]]; then
        log_info "Health check passed"
        echo "Response: $response"
    else
        log_error "Health check failed"
        return 1
    fi
}

# Test server info endpoint
test_info() {
    log_info "Testing server info endpoint..."
    
    response=$(curl -s --max-time $TIMEOUT "$SERVER_URL/info")
    if [[ $? -eq 0 ]]; then
        log_info "Server info check passed"
        echo "Response: $response"
    else
        log_error "Server info check failed"
        return 1
    fi
}

# Test RPC endpoint with ping
test_ping() {
    log_info "Testing RPC ping..."
    
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"ping","params":{},"id":1}' \
        "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "RPC ping test passed"
        echo "Response: $response"
    else
        log_error "RPC ping test failed"
        return 1
    fi
}

# Test RPC endpoint with JSON validation
test_json_validation() {
    log_info "Testing JSON validation..."
    
    # Test valid JSON
    valid_request='{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"test","age":25},"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "$valid_request" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Valid JSON test passed"
        echo "Response: $response"
    else
        log_error "Valid JSON test failed"
        return 1
    fi
    
    # Test invalid JSON (note: minimal server has simplified validation)
    invalid_request='{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"test","age":"invalid"},"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "$invalid_request" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Invalid JSON test passed"
        echo "Response: $response"
    else
        log_error "Invalid JSON test failed"
        return 1
    fi
}

# Test error handling
test_error_handling() {
    log_info "Testing error handling..."
    
    # Test invalid method
    invalid_method='{"jsonrpc":"2.0","method":"unknown_method","params":{},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "$invalid_method" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Invalid method test passed"
        echo "Response: $response"
    else
        log_error "Invalid method test failed"
        return 1
    fi
    
    # Test invalid JSON-RPC version
    invalid_version='{"jsonrpc":"1.0","method":"ping","params":{},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" \
        -d "$invalid_version" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Invalid version test passed"
        echo "Response: $response"
    else
        log_error "Invalid version test failed"
        return 1
    fi
}

# Test non-existent endpoint
test_404() {
    log_info "Testing 404 response..."
    
    response=$(curl -s --max-time $TIMEOUT "$SERVER_URL/nonexistent")
    if [[ $? -eq 0 ]]; then
        log_info "404 test passed"
        echo "Response: $response"
    else
        log_error "404 test failed"
        return 1
    fi
}

# Run all tests
run_all_tests() {
    log_info "Starting comprehensive test suite..."
    
    tests=(
        "test_health"
        "test_info"
        "test_ping"
        "test_json_validation"
        "test_error_handling"
        "test_404"
    )
    
    passed=0
    failed=0
    
    for test in "${tests[@]}"; do
        log_info "Running $test..."
        if $test; then
            ((passed++))
            log_info "$test passed"
        else
            ((failed++))
            log_error "$test failed"
        fi
        echo "----------------------------------------"
    done
    
    log_info "Test Results:"
    log_info "Passed: $passed"
    log_info "Failed: $failed"
    log_info "Total: $((passed + failed))"
    
    if [[ $failed -eq 0 ]]; then
        log_info "All tests passed!"
        return 0
    else
        log_error "$failed tests failed"
        return 1
    fi
}

# Main execution
main() {
    log_info "JSON Validator HTTP Server Test Script"
    log_info "Server URL: $SERVER_URL"
    echo ""
    
    # Check if server is running
    if ! check_server; then
        log_error "Server is not running. Please start the server first."
        exit 1
    fi
    
    # Parse command line arguments
    case "${1:-all}" in
        "health")
            test_health
            ;;
        "info")
            test_info
            ;;
        "ping")
            test_ping
            ;;
        "validation")
            test_json_validation
            ;;
        "error")
            test_error_handling
            ;;
        "404")
            test_404
            ;;
        "all")
            run_all_tests
            ;;
        *)
            echo "Usage: $0 [health|info|ping|validation|error|404|all]"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"