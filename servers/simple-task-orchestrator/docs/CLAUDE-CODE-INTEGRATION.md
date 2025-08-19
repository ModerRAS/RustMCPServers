# Claude Code 任务执行功能

## 概述

任务编排MCP服务器现在支持使用Claude Code来执行任务。这个功能允许您通过Claude Code AI助手来处理复杂的编程任务。

## 功能特性

### 1. 执行模式

任务支持三种执行模式：

- **Standard**: 标准执行模式（默认）
- **ClaudeCode**: 使用Claude Code执行
- **Custom**: 自定义执行器

### 2. Claude Code 执行器

Claude Code执行器提供以下功能：

- 自动调用Claude Code CLI
- 支持多种模型（claude-sonnet, claude-opus等）
- 配置超时和工作目录
- 流式JSON输出解析
- 完整的错误处理

## 使用方法

### 1. 创建Claude Code任务

```bash
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "work_directory": "/path/to/your/project",
    "prompt": "实现一个用户认证系统，包括登录、注册和权限管理功能",
    "priority": "High",
    "execution_mode": "ClaudeCode",
    "tags": ["auth", "backend"]
  }'
```

### 2. 执行单个任务

```bash
curl -X POST http://localhost:8080/api/v1/tasks/{task_id}/execute
```

### 3. 执行指定目录的所有任务

```bash
curl -X POST http://localhost:8080/api/v1/execute/directory/$(encodeURIComponent -path)
```

### 4. 查看执行结果

执行完成后，任务状态会更新为`Completed`，结果包含：

- **output**: Claude Code的输出内容
- **duration_ms**: 执行时间（毫秒）
- **status**: 执行状态（success/failure）

## 配置选项

### Claude Code 配置

```rust
ClaudeCodeConfig {
    claude_path: "claude".to_string(),          // Claude Code CLI路径
    model: Some("claude-sonnet-4-20250514".to_string()),  // 使用的模型
    timeout: 600,                              // 超时时间（秒）
    work_directory: ".".to_string(),            // 工作目录
    verbose: false,                            // 详细输出
}
```

### 支持的模型

- `claude-sonnet-4-20250514`
- `claude-opus-4-20250514`
- `claude-3-5-sonnet-20241022`
- `claude-3-5-haiku-20241022`

## 示例场景

### 1. 代码重构任务

```json
{
  "work_directory": "/src/my-project",
  "prompt": "重构这个项目的代码结构，采用清洁架构原则，提高代码的可维护性和可测试性",
  "execution_mode": "ClaudeCode",
  "priority": "Medium"
}
```

### 2. 新功能开发

```json
{
  "work_directory": "/src/ecommerce",
  "prompt": "实现购物车功能，包括添加商品、删除商品、更新数量和计算总价",
  "execution_mode": "ClaudeCode",
  "priority": "High"
}
```

### 3. 代码审查

```json
{
  "work_directory": "/src/api",
  "prompt": "审查这个API模块的代码，检查安全性、性能和最佳实践",
  "execution_mode": "ClaudeCode",
  "priority": "Medium"
}
```

## 完整工作流程

### 1. 创建任务

```bash
# 创建Claude Code执行任务
TASK_RESPONSE=$(curl -s -X POST http://localhost:8080/api/v1/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "work_directory": "/tmp/test-project",
    "prompt": "创建一个简单的Rust Web服务器",
    "execution_mode": "ClaudeCode",
    "priority": "High"
  }')

# 提取任务ID
TASK_ID=$(echo $TASK_RESPONSE | jq -r '.data.id')
echo "创建任务: $TASK_ID"
```

### 2. 获取任务到工作状态

```bash
# 获取任务
curl -X GET "http://localhost:8080/api/v1/tasks/next?work_path=/tmp/test-project&worker_id=claude-worker"
```

### 3. 执行任务

```bash
# 执行任务
curl -X POST "http://localhost:8080/api/v1/tasks/$TASK_ID/execute"
```

### 4. 查看结果

```bash
# 查看任务结果
curl -X GET "http://localhost:8080/api/v1/tasks/$TASK_ID"
```

## 错误处理

### 常见错误

1. **Claude Code CLI未找到**
   - 确保Claude Code CLI已安装并在PATH中
   - 或在配置中指定正确的路径

2. **任务状态错误**
   - 任务必须处于`Working`状态才能执行
   - 确保先获取任务再执行

3. **超时错误**
   - 增加timeout配置
   - 检查网络连接

### 错误响应示例

```json
{
  "success": false,
  "error": {
    "code": "EXECUTION_ERROR",
    "message": "Claude Code execution failed: Command not found"
  }
}
```

## 监控和日志

### 执行统计

```bash
# 获取系统统计信息
curl -X GET "http://localhost:8080/api/v1/statistics"
```

### 日志配置

服务器提供详细的执行日志：

- **INFO**: 执行开始和完成
- **ERROR**: 执行失败和错误信息
- **DEBUG**: 详细的执行过程

## 性能考虑

### 1. 并发限制

- Claude Code执行是CPU密集型操作
- 建议限制并发执行数量
- 使用队列管理执行任务

### 2. 资源使用

- 每个Claude Code进程会占用较多内存
- 监控系统资源使用情况
- 设置合理的超时时间

### 3. 成本控制

- Claude Code API调用会产生费用
- 监控使用量和成本
- 设置使用限额

## 安全考虑

### 1. 权限管理

- 确保Claude Code有适当的文件系统权限
- 限制访问敏感文件和目录
- 使用沙箱环境执行

### 2. 输入验证

- 验证工作目录路径
- 检查prompt内容安全性
- 防止命令注入攻击

### 3. 输出安全

- 清理执行输出中的敏感信息
- 记录执行日志
- 监控异常行为

## 故障排除

### 1. Claude Code CLI问题

```bash
# 检查Claude Code是否可用
claude --version

# 测试Claude Code连接
claude -p "hello"
```

### 2. 权限问题

```bash
# 检查目录权限
ls -la /path/to/work/directory

# 确保有读写权限
chmod +w /path/to/work/directory
```

### 3. 网络问题

```bash
# 检查网络连接
ping api.anthropic.com

# 检查代理设置
echo $HTTP_PROXY
echo $HTTPS_PROXY
```

## 最佳实践

### 1. 任务设计

- 使用清晰、具体的prompt
- 提供充分的上下文信息
- 设置合理的优先级
- 使用适当的标签分类

### 2. 错误处理

- 实现重试机制
- 记录详细的错误信息
- 提供用户友好的错误消息
- 监控错误率

### 3. 性能优化

- 使用连接池
- 实现缓存机制
- 优化并发控制
- 监控资源使用

### 4. 监控和维护

- 定期检查执行状态
- 监控系统性能
- 更新Claude Code CLI
- 维护执行日志

## API 参考

### 任务执行端点

- `POST /api/v1/tasks/{id}/execute` - 执行单个任务
- `POST /api/v1/execute/directory/{path}` - 执行目录中的所有任务

### 响应格式

```json
{
  "success": true,
  "data": {
    "task_id": "uuid",
    "execution_result": {
      "status": "success",
      "output": "执行结果",
      "duration_ms": 1500
    },
    "executed_at": "2024-01-01T00:00:00Z"
  }
}
```

## 版本历史

### v1.0.0
- 初始Claude Code执行功能
- 支持基本的任务执行
- 提供配置选项
- 错误处理和日志记录

## 贡献指南

欢迎贡献代码和改进建议！请参考项目的贡献指南。

## 许可证

本项目采用MIT许可证。