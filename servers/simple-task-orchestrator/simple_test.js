#!/usr/bin/env node

const { spawn } = require('child_process');

async function testMCPServer() {
  console.log('🚀 测试 Simple Task Orchestrator MCP 服务器');
  console.log('=========================================');

  const serverPath = '/root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server';
  
  const server = spawn(serverPath, [], {
    stdio: ['pipe', 'pipe', 'pipe']
  });

  let outputBuffer = '';
  let requestId = 1;

  server.stdout.on('data', (data) => {
    const output = data.toString();
    outputBuffer += output;
    console.log('📤 Server:', output);
  });

  server.stderr.on('data', (data) => {
    console.log('❌ Error:', data.toString());
  });

  server.on('close', (code) => {
    console.log('🔚 Server closed with code:', code);
  });

  // 等待服务器启动
  await new Promise(resolve => setTimeout(resolve, 1000));

  // 发送初始化请求
  const initRequest = {
    jsonrpc: '2.0',
    id: requestId++,
    method: 'initialize',
    params: {
      protocolVersion: '2024-11-05',
      capabilities: {
        roots: {
          listChanged: true
        }
      },
      clientInfo: {
        name: 'test-client',
        version: '1.0.0'
      }
    }
  };

  console.log('🔧 发送初始化请求...');
  server.stdin.write(JSON.stringify(initRequest) + '\n');

  // 等待初始化响应
  await new Promise(resolve => setTimeout(resolve, 1000));

  // 发送初始化通知
  const initializedNotification = {
    jsonrpc: '2.0',
    method: 'initialized'
  };

  console.log('📤 发送初始化通知...');
  server.stdin.write(JSON.stringify(initializedNotification) + '\n');

  // 等待一下
  await new Promise(resolve => setTimeout(resolve, 1000));

  // 获取工具列表
  const toolsRequest = {
    jsonrpc: '2.0',
    id: requestId++,
    method: 'tools/list'
  };

  console.log('📋 获取工具列表...');
  server.stdin.write(JSON.stringify(toolsRequest) + '\n');

  // 等待工具列表响应
  await new Promise(resolve => setTimeout(resolve, 2000));

  // 清理
  server.kill();
  console.log('✅ 测试完成！');
}

testMCPServer().catch(console.error);