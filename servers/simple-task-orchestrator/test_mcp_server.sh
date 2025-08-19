#!/bin/bash

echo "测试 Simple Task Orchestrator MCP 服务器"
echo "========================================"

# 测试创建任务
echo "1. 测试创建任务..."
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"create_task","arguments":{"work_directory":"/tmp","prompt":"Hello World","priority":"Medium","execution_mode":"Standard"}}}' | timeout 5 /root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server

echo ""
echo "2. 测试列出任务..."
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"list_tasks","arguments":{}}}' | timeout 5 /root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server

echo ""
echo "3. 测试获取统计信息..."
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_statistics","arguments":{}}}' | timeout 5 /root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server

echo ""
echo "测试完成！"