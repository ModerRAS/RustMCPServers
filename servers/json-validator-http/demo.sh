#!/bin/bash

# JSON Validator HTTP Server Demo
# This script demonstrates the complete functionality of the HTTP JSON validator server

SERVER_URL="http://127.0.0.1:8082"

echo "=========================================="
echo "JSON Validator HTTP Server Demo"
echo "=========================================="
echo ""

# 1. 健康检查
echo "1. Health Check"
echo "Endpoint: GET /health"
curl -s "$SERVER_URL/health" | jq .
echo ""

# 2. 服务器信息
echo "2. Server Information"
echo "Endpoint: GET /info"
curl -s "$SERVER_URL/info" | jq .
echo ""

# 3. Ping测试
echo "3. Ping Test"
echo "Endpoint: POST /rpc"
echo "Method: ping"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"ping","params":{},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 4. 基础JSON验证 - 有效数据
echo "4. Basic JSON Validation - Valid Data"
echo "Endpoint: POST /rpc"
echo "Method: validate_json"
echo "Data: {\"name\":\"John\",\"age\":30}"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"John","age":30},"options":{"strict_mode":false}},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 5. 基础JSON验证 - 无效数据
echo "5. Basic JSON Validation - Invalid Data"
echo "Endpoint: POST /rpc"
echo "Method: validate_json"
echo "Data: {\"name\":\"Jane\",\"age\":\"invalid\"}"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"validate_json","params":{"json_data":{"name":"Jane","age":"invalid"},"options":{"strict_mode":false}},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 6. Schema验证 - 有效数据
echo "6. Schema Validation - Valid Data"
echo "Endpoint: POST /rpc"
echo "Method: validate_json_with_schema"
echo "Schema: {\"type\":\"object\",\"properties\":{\"name\":{\"type\":\"string\"},\"age\":{\"type\":\"number\"}},\"required\":[\"name\",\"age\"]}"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"validate_json_with_schema","params":{"json_data":{"name":"Alice","age":25},"schema":{"type":"object","properties":{"name":{"type":"string"},"age":{"type":"number"}},"required":["name","age"]},"options":{"strict_mode":false}},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 7. 批量验证
echo "7. Batch Validation"
echo "Endpoint: POST /rpc"
echo "Method: validate_json_batch"
echo "Items: [{\"id\":\"1\",\"json_data\":{\"name\":\"Item 1\",\"value\":100}},{\"id\":\"2\",\"json_data\":{\"name\":\"Item 2\",\"value\":\"invalid\"}}]"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"validate_json_batch","params":{"items":[{"id":"1","json_data":{"name":"Item 1","value":100}},{"id":"2","json_data":{"name":"Item 2","value":"invalid"}}],"options":{"strict_mode":false}},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 8. 错误处理 - 未知方法
echo "8. Error Handling - Unknown Method"
echo "Endpoint: POST /rpc"
echo "Method: unknown_method"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"2.0","method":"unknown_method","params":{},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 9. 错误处理 - 无效JSON-RPC版本
echo "9. Error Handling - Invalid JSON-RPC Version"
echo "Endpoint: POST /rpc"
echo "Version: 1.0 (should be 2.0)"
curl -s -X POST -H "Content-Type: application/json" \
    -d '{"jsonrpc":"1.0","method":"ping","params":{},"id":1}' \
    "$SERVER_URL/rpc" | jq .
echo ""

# 10. 404错误
echo "10. 404 Error"
echo "Endpoint: GET /nonexistent"
curl -s "$SERVER_URL/nonexistent" | jq .
echo ""

echo "=========================================="
echo "Demo completed successfully!"
echo "=========================================="