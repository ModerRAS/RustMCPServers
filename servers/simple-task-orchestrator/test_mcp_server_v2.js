#!/usr/bin/env node

const { spawn } = require('child_process');

class MCPServerTester {
  constructor() {
    this.server = null;
    this.requestId = 1;
  }

  async startServer() {
    return new Promise((resolve, reject) => {
      const serverPath = '/root/WorkSpace/Rust/RustMCPServers/target/debug/mcp_server';
      
      this.server = spawn(serverPath, [], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      this.server.stdout.on('data', (data) => {
        console.log('Server output:', data.toString());
      });

      this.server.stderr.on('data', (data) => {
        console.log('Server error:', data.toString());
      });

      this.server.on('error', (error) => {
        console.error('Server error:', error);
        reject(error);
      });

      this.server.on('close', (code) => {
        console.log('Server closed with code:', code);
      });

      // 等待服务器启动
      setTimeout(() => {
        resolve();
      }, 1000);
    });
  }

  sendRequest(method, params = {}) {
    return new Promise((resolve, reject) => {
      const request = {
        jsonrpc: '2.0',
        id: this.requestId++,
        method: method,
        params: params
      };

      const requestStr = JSON.stringify(request);
      
      let buffer = '';
      const handleOutput = (data) => {
        buffer += data.toString();
        try {
          const response = JSON.parse(buffer);
          if (response.id === request.id - 1) {
            this.server.stdout.removeListener('data', handleOutput);
            resolve(response);
          }
        } catch (e) {
          // 不是JSON格式，继续等待
        }
      };

      this.server.stdout.on('data', handleOutput);

      // 发送请求
      this.server.stdin.write(requestStr + '\n');
      console.log('Sent:', requestStr);

      // 超时处理
      setTimeout(() => {
        this.server.stdout.removeListener('data', handleOutput);
        reject(new Error('Request timeout'));
      }, 5000);
    });
  }

  sendNotification(method, params = {}) {
    const notification = {
      jsonrpc: '2.0',
      method: method,
      params: params
    };
    const notificationStr = JSON.stringify(notification);
    this.server.stdin.write(notificationStr + '\n');
    console.log('Sent notification:', notificationStr);
  }

  async test() {
    try {
      console.log('🚀 启动MCP服务器...');
      await this.startServer();

      console.log('🔧 初始化服务器...');
      const initResponse = await this.sendRequest('initialize', {
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
      });
      console.log('初始化响应:', initResponse);

      // 发送初始化通知
      console.log('📤 发送初始化通知...');
      this.sendNotification('initialized');

      console.log('📋 获取工具列表...');
      const toolsResponse = await this.sendRequest('tools/list');
      console.log('工具列表:', toolsResponse);

      if (toolsResponse.result && toolsResponse.result.tools) {
        console.log('可用工具:');
        toolsResponse.result.tools.forEach(tool => {
          console.log(`  - ${tool.name}: ${tool.description}`);
        });
      }

      console.log('🔧 测试创建任务...');
      const createTaskResponse = await this.sendRequest('tools/call', {
        name: 'create_task',
        arguments: {
          work_directory: '/tmp',
          prompt: 'Hello World Test',
          priority: 'Medium',
          execution_mode: 'Standard'
        }
      });
      console.log('创建任务响应:', createTaskResponse);

      console.log('📋 测试列出任务...');
      const listTasksResponse = await this.sendRequest('tools/call', {
        name: 'list_tasks',
        arguments: {}
      });
      console.log('列出任务响应:', listTasksResponse);

      console.log('📊 测试获取统计信息...');
      const statsResponse = await this.sendRequest('tools/call', {
        name: 'get_statistics',
        arguments: {}
      });
      console.log('统计信息响应:', statsResponse);

      console.log('✅ 所有测试完成！');

    } catch (error) {
      console.error('❌ 测试失败:', error);
    } finally {
      if (this.server) {
        this.server.kill();
      }
    }
  }
}

// 运行测试
const tester = new MCPServerTester();
tester.test();