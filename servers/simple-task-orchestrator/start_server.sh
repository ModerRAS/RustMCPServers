#!/bin/bash

echo "🚀 启动 Simple Task Orchestrator MCP 服务器"
echo "========================================="

# 停止现有服务
pm2 stop simple-task-orchestrator 2>/dev/null || true

# 启动服务
pm2 start ecosystem.config.js

echo ""
echo "✅ MCP 服务器已启动！"
echo ""
echo "🌐 请使用 MCP Inspector 连接："
echo "   http://0.0.0.0:6274/?MCP_PROXY_AUTH_TOKEN=e2664319461e4c1044b90ca01715aa376f1b6bb261adac45f485982a4d507300"
echo ""
echo "📋 在 Inspector 中，选择 'stdio' 连接方式，然后输入以下命令："
echo "   /root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server"
echo ""
echo "🔧 或者你可以直接使用以下命令测试："
echo "   echo '{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2024-11-05\",\"capabilities\":{\"roots\":{\"listChanged\":true}},\"clientInfo\":{\"name\":\"test-client\",\"version\":\"1.0.0\"}}}' | /root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server"
echo ""
echo "📊 查看日志："
echo "   pm2 logs simple-task-orchestrator"
echo ""
echo "🛑 停止服务："
echo "   pm2 stop simple-task-orchestrator"