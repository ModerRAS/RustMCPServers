#!/bin/bash

# JSON Validator HTTP Server Test Script
# This script tests the HTTP JSON validator server functionality

set -e

# Configuration
SERVER_URL="http://localhost:8080"
METRICS_URL="http://localhost:9090"
TIMEOUT=10

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

# Check if server is running
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

# Test metrics endpoint
test_metrics() {
    log_info "Testing metrics endpoint..."
    
    response=$(curl -s --max-time $TIMEOUT "$SERVER_URL/metrics")
    if [[ $? -eq 0 ]]; then
        log_info "Metrics check passed"
        echo "Metrics count: $(echo "$response" | grep -c '^')"
    else
        log_error "Metrics check failed"
        return 1
    fi
}

# Test JSON validation
test_json_validation() {
    log_info "Testing JSON validation..."
    
    # Valid JSON
    valid_json='{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"test","age":25},"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$valid_json" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Valid JSON test passed"
        echo "Response: $response"
    else
        log_error "Valid JSON test failed"
        return 1
    fi
    
    # Invalid JSON
    invalid_json='{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"test","age":"invalid"},"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$invalid_json" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Invalid JSON test passed"
        echo "Response: $response"
    else
        log_error "Invalid JSON test failed"
        return 1
    fi
}

# Test JSON Schema validation
test_schema_validation() {
    log_info "Testing JSON Schema validation..."
    
    # Valid schema validation
    schema_request='{"jsonrpc":"2.0","method":"validate_json_with_schema","params":{"json_data":{"name":"test","age":25},"schema":{"type":"object","properties":{"name":{"type":"string"},"age":{"type":"number"}},"required":["name","age"]},"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$schema_request" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Valid schema test passed"
        echo "Response: $response"
    else
        log_error "Valid schema test failed"
        return 1
    fi
    
    # Invalid schema validation
    invalid_schema_request='{"jsonrpc":"2.0","method":"validate_json_with_schema","params":{"json_data":{"name":"test","age":"invalid"},"schema":{"type":"object","properties":{"name":{"type":"string"},"age":{"type":"number"}},"required":["name","age"]},"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$invalid_schema_request" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Invalid schema test passed"
        echo "Response: $response"
    else
        log_error "Invalid schema test failed"
        return 1
    fi
}

# Test batch validation
test_batch_validation() {
    log_info "Testing batch validation..."
    
    batch_request='{"jsonrpc":"2.0","method":"validate_json_batch","params":{"items":[{"id":"1","json_data":{"name":"item1"}},{"id":"2","json_data":{"name":"item2"}}],"options":{"strict_mode":false}},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$batch_request" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Batch validation test passed"
        echo "Response: $response"
    else
        log_error "Batch validation test failed"
        return 1
    fi
}

# Test error handling
test_error_handling() {
    log_info "Testing error handling..."
    
    # Invalid JSON-RPC request
    invalid_request='{"invalid":"request"}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$invalid_request" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Error handling test passed"
        echo "Response: $response"
    else
        log_error "Error handling test failed"
        return 1
    fi
    
    # Unknown method
    unknown_method='{"jsonrpc":"2.0","method":"unknown_method","params":{},"id":1}'
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$unknown_method" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Unknown method test passed"
        echo "Response: $response"
    else
        log_error "Unknown method test failed"
        return 1
    fi
}

# Test performance
test_performance() {
    log_info "Testing performance..."
    
    # Test multiple concurrent requests
    start_time=$(date +%s.%N)
    
    for i in {1..10}; do
        test_request='{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"test'$i'","age":'$i'},"options":{"strict_mode":false}},"id":'$i'}'
        curl -s -X POST -H "Content-Type: application/json" -d "$test_request" "$SERVER_URL/rpc" > /dev/null &
    done
    
    wait
    
    end_time=$(date +%s.%N)
    duration=$(echo "$end_time - $start_time" | bc)
    
    log_info "Performance test completed in $duration seconds"
    log_info "Average request time: $(echo "$duration / 10" | bc) seconds"
}

# Test large JSON handling
test_large_json() {
    log_info "Testing large JSON handling..."
    
    # Create a large JSON object
    large_json='{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":'
    large_json+='{"data":['
    
    for i in {1..1000}; do
        large_json+='{"id":"'$i'","name":"item'$i'","description":"This is a test item with number '$i'","value":'$i'},'
    done
    
    large_json=${large_json%,}  # Remove trailing comma
    large_json+=']},"options":{"strict_mode":false}},"id":1}'
    
    response=$(curl -s -X POST -H "Content-Type: application/json" -d "$large_json" "$SERVER_URL/rpc")
    
    if [[ $? -eq 0 ]]; then
        log_info "Large JSON test passed"
        echo "Response length: ${#response}"
    else
        log_error "Large JSON test failed"
        return 1
    fi
}

# Run all tests
run_all_tests() {
    log_info "Starting comprehensive test suite..."
    
    tests=(
        "test_health"
        "test_info"
        "test_metrics"
        "test_json_validation"
        "test_schema_validation"
        "test_batch_validation"
        "test_error_handling"
        "test_performance"
        "test_large_json"
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
    log_info "Metrics URL: $METRICS_URL"
    echo ""
    
    # Check if server is running
    if ! check_server; then
        log_error "Server is not running. Please start the server first."
        log_info "You can start the server with: cargo run"
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
        "metrics")
            test_metrics
            ;;
        "validation")
            test_json_validation
            ;;
        "schema")
            test_schema_validation
            ;;
        "batch")
            test_batch_validation
            ;;
        "error")
            test_error_handling
            ;;
        "performance")
            test_performance
            ;;
        "large")
            test_large_json
            ;;
        "all")
            run_all_tests
            ;;
        *)
            echo "Usage: $0 [health|info|metrics|validation|schema|batch|error|performance|large|all]"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"