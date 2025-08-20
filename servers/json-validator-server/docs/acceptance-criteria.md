# JSON验证MCP服务器HTTP协议转换验收标准

## 概述

本文档定义了JSON验证MCP服务器从stdio协议转换为HTTP协议的详细验收标准。每个验收标准都包含具体的测试场景、预期结果和验证方法。

## 验收标准组织结构

### 验收级别
- **Level 1**: 基础功能验证 (Must Pass)
- **Level 2**: 完整功能验证 (Should Pass)  
- **Level 3**: 高级功能验证 (Nice to Have)

### 验证方法
- **自动化测试**: 通过测试脚本自动验证
- **手动测试**: 需要人工干预的测试
- **性能测试**: 验证性能指标
- **集成测试**: 验证与其他系统的集成

## Level 1: 基础功能验证

### AC-001: HTTP服务器启动和基础配置
**描述**: 验证HTTP服务器能够正常启动并响应基本请求

**测试场景**:
1. 默认配置启动
2. 自定义端口配置启动
3. 配置文件配置启动
4. 环境变量配置启动

**验收标准**:
- [ ] 服务器使用默认配置启动成功，监听端口8080
- [ ] 服务器使用自定义端口启动成功，监听指定端口
- [ ] 服务器能够从配置文件正确读取配置
- [ ] 服务器能够从环境变量正确读取配置
- [ ] 服务器启动时显示正确的启动信息和监听地址

**验证方法**: 自动化测试 + 手动验证

**测试命令**:
```bash
# 默认配置启动
cargo run --bin json-validator-http

# 自定义端口启动
cargo run --bin json-validator-http -- --port 9090

# 配置文件启动
cargo run --bin json-validator-http -- --config config.toml

# 环境变量启动
JSON_VALIDATOR_PORT=8081 cargo run --bin json-validator-http
```

### AC-002: JSON-RPC端点功能验证
**描述**: 验证JSON-RPC端点能够正确处理MCP协议消息

**测试场景**:
1. 有效JSON-RPC请求处理
2. 无效JSON-RPC请求处理
3. 工具调用请求处理
4. 批量请求处理

**验收标准**:
- [ ] POST请求到根路径返回正确的JSON-RPC响应
- [ ] 无效JSON格式请求返回适当的错误响应
- [ ] JSON-RPC版本不匹配返回版本错误
- [ ] 请求ID在响应中正确映射
- [ ] 批量请求被正确处理

**验证方法**: 自动化测试

**测试用例**:
```json
// 有效请求
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "validate_json_content",
    "arguments": {
      "json_content": "{\"test\": \"value\"}"
    }
  }
}

// 无效请求
{
  "jsonrpc": "1.0",
  "id": 1,
  "method": "tools/call",
  "params": {}
}
```

### AC-003: 现有工具功能验证
**描述**: 验证所有现有工具在HTTP版本中正常工作

**测试场景**:
1. validate_json_file工具测试
2. validate_json_content工具测试
3. format_json工具测试

**验收标准**:
- [ ] validate_json_file工具验证有效JSON文件返回正确结果
- [ ] validate_json_file工具验证无效JSON文件返回错误信息
- [ ] validate_json_file工具处理文件不存在的情况
- [ ] validate_json_content工具验证有效JSON返回正确结果
- [ ] validate_json_content工具验证无效JSON返回错误信息
- [ ] format_json工具正确格式化有效JSON
- [ ] format_json工具处理无效JSON返回错误
- [ ] 所有工具返回格式与stdio版本完全一致

**验证方法**: 自动化测试

**测试数据**:
```json
// 有效JSON文件测试
{
  "file_path": "/path/to/valid.json",
  "expected_result": {
    "valid": true,
    "message": "JSON file is valid"
  }
}

// 无效JSON文件测试
{
  "file_path": "/path/to/invalid.json",
  "expected_result": {
    "valid": false,
    "message": "Invalid JSON: ...",
    "error_line": 1,
    "error_column": 5
  }
}
```

### AC-004: 错误处理验证
**描述**: 验证服务器能够正确处理各种错误情况

**测试场景**:
1. HTTP错误状态码处理
2. JSON-RPC错误处理
3. 系统错误处理
4. 网络错误处理

**验收标准**:
- [ ] 无效HTTP方法返回405 Method Not Allowed
- [ ] 无效JSON格式返回400 Bad Request
- [ ] 服务器内部错误返回500 Internal Server Error
- [ ] JSON-RPC错误格式符合规范
- [ ] 错误信息包含足够的调试信息
- [ ] 服务器在错误情况下保持稳定运行

**验证方法**: 自动化测试 + 手动验证

**测试用例**:
```bash
# 测试无效HTTP方法
curl -X GET http://localhost:8080/ -H "Content-Type: application/json"

# 测试无效JSON格式
curl -X POST http://localhost:8080/ -H "Content-Type: application/json" -d "invalid json"

# 测试不存在的工具
curl -X POST http://localhost:8080/ -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"nonexistent_tool","arguments":{}}}'
```

### AC-005: 健康检查验证
**描述**: 验证健康检查端点功能正常

**测试场景**:
1. 正常状态健康检查
2. 服务器负载状态健康检查

**验收标准**:
- [ ] GET /health返回200状态码
- [ ] 健康检查响应包含服务器状态信息
- [ ] 响应包含版本信息
- [ ] 响应包含运行时间统计
- [ ] 响应格式为JSON

**验证方法**: 自动化测试

**测试命令**:
```bash
curl http://localhost:8080/health
```

**预期响应**:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "0.1.0",
  "uptime": 3600,
  "service": "json-validator-http"
}
```

## Level 2: 完整功能验证

### AC-006: 配置管理验证
**描述**: 验证服务器配置管理功能

**测试场景**:
1. 配置文件加载测试
2. 环境变量配置测试
3. 命令行参数测试
4. 配置验证测试

**验收标准**:
- [ ] 服务器能够正确加载TOML配置文件
- [ ] 环境变量配置优先级正确
- [ ] 命令行参数覆盖其他配置源
- [ ] 无效配置提供清晰的错误信息
- [ ] 配置热重载功能正常（如果实现）

**验证方法**: 自动化测试 + 手动验证

**配置文件示例**:
```toml
[server]
host = "0.0.0.0"
port = 8080
timeout = 30

[logging]
level = "info"
format = "json"

[security]
max_request_size = 1048576
rate_limit = 100
```

### AC-007: 日志记录验证
**描述**: 验证日志记录功能

**测试场景**:
1. 请求日志记录
2. 错误日志记录
3. 性能日志记录
4. 日志格式验证

**验收标准**:
- [ ] 所有HTTP请求都被记录
- [ ] 错误信息被正确记录
- [ ] 日志包含请求ID和时间戳
- [ ] 支持结构化JSON日志格式
- [ ] 支持日志级别配置
- [ ] 日志不包含敏感信息

**验证方法**: 手动验证 + 自动化测试

**日志格式示例**:
```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "level": "INFO",
  "request_id": "req_123456",
  "method": "POST",
  "path": "/",
  "status_code": 200,
  "duration_ms": 15,
  "message": "Request processed successfully"
}
```

### AC-008: 性能基准验证
**描述**: 验证服务器性能满足要求

**测试场景**:
1. 响应时间测试
2. 并发请求测试
3. 内存使用测试
4. 吞吐量测试

**验收标准**:
- [ ] 99%的请求响应时间 < 500ms
- [ ] 支持100个并发连接
- [ ] 空闲状态内存使用 < 100MB
- [ ] 吞吐量 > 1000 requests/second
- [ ] 服务器在负载下保持稳定

**验证方法**: 性能测试

**测试工具**:
```bash
# 使用wrk进行性能测试
wrk -t12 -c400 -d30s http://localhost:8080/health

# 使用curl进行响应时间测试
curl -o /dev/null -s -w "%{time_total}\n" http://localhost:8080/health
```

### AC-009: 安全性验证
**描述**: 验证服务器安全性控制

**测试场景**:
1. 输入验证测试
2. 请求大小限制测试
3. 恶意请求防护测试
4. 信息泄露防护测试

**验收标准**:
- [ ] 恶意JSON输入被拒绝
- [ ] 超过大小限制的请求被拒绝
- [ ] 错误信息不包含敏感数据
- [ ] 服务器不暴露内部实现细节
- [ ] 支持CORS配置

**验证方法**: 安全测试 + 手动验证

**安全测试用例**:
```bash
# 测试请求大小限制
curl -X POST http://localhost:8080/ -H "Content-Type: application/json" -d "$(cat /dev/urandom | tr -dc 'a-zA-Z0-9' | fold -w 1000000 | head -n 1)"

# 测试恶意JSON输入
curl -X POST http://localhost:8080/ -H "Content-Type: application/json" -d '{"malicious": "input", "nested": {"deep": {"structure": {"with": {"injection": "alert('XSS')"}}}}}'
```

### AC-010: 向后兼容性验证
**描述**: 验证与stdio版本的兼容性

**测试场景**:
1. API接口兼容性测试
2. 响应格式兼容性测试
3. 错误处理兼容性测试
4. 功能行为兼容性测试

**验收标准**:
- [ ] 所有工具名称和参数保持不变
- [ ] 响应格式与stdio版本完全一致
- [ ] 错误处理逻辑保持一致
- [ ] 所有功能行为保持一致
- [ ] 现有客户端可以无缝切换

**验证方法**: 兼容性测试

**兼容性测试方法**:
```bash
# 同时运行stdio和HTTP版本进行对比
./json-validator-stdio &
./json-validator-http &

# 使用相同的测试用例测试两个版本
```

## Level 3: 高级功能验证

### AC-011: 容器化部署验证
**描述**: 验证Docker容器化部署

**测试场景**:
1. Docker镜像构建测试
2. 容器运行测试
3. 环境变量配置测试
4. 健康检查测试

**验收标准**:
- [ ] Docker镜像能够成功构建
- [ ] 容器能够正常启动和运行
- [ ] 环境变量配置在容器中正常工作
- [ ] 容器健康检查功能正常
- [ ] 容器日志输出正确

**验证方法**: 容器化测试

**测试命令**:
```bash
# 构建Docker镜像
docker build -t json-validator-http .

# 运行容器
docker run -p 8080:8080 json-validator-http

# 测试容器健康检查
docker ps
curl http://localhost:8080/health
```

### AC-012: 监控和指标验证
**描述**: 验证监控和指标收集功能

**测试场景**:
1. 性能指标收集测试
2. 监控端点测试
3. 指标格式验证测试

**验收标准**:
- [ ] 性能指标被正确收集
- [ ] 监控端点返回有效数据
- [ ] 指标格式符合Prometheus标准
- [ ] 指标包含关键性能数据

**验证方法**: 监控测试

**测试命令**:
```bash
curl http://localhost:8080/metrics
```

### AC-013: 优雅关闭验证
**描述**: 验证服务器优雅关闭功能

**测试场景**:
1. SIGINT信号处理测试
2. SIGTERM信号处理测试
3. 请求完成测试
4. 资源清理测试

**验收标准**:
- [ ] 服务器正确处理SIGINT信号
- [ ] 服务器正确处理SIGTERM信号
- [ ] 正在处理的请求能够完成
- [ ] 所有资源被正确清理
- [ ] 关闭过程有适当的日志记录

**验证方法**: 手动验证

**测试方法**:
```bash
# 启动服务器
./json-validator-http &

# 发送测试请求
curl -X POST http://localhost:8080/ -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"validate_json_content","arguments":{"json_content":"{\"test\":\"value\"}"}}}' &

# 立即发送关闭信号
kill -SIGTERM $!

# 检查进程是否优雅关闭
```

## 测试数据和环境

### 测试环境要求
- **操作系统**: Linux (Ubuntu 20.04+)
- **Rust版本**: 1.70+
- **内存**: 最少2GB
- **磁盘空间**: 最少1GB

### 测试数据文件
```json
// valid.json
{
  "name": "test",
  "value": 123,
  "active": true,
  "items": ["item1", "item2"]
}

// invalid.json
{
  "name": "test",
  "value": 123,
  "active": true,
  "items": ["item1", "item2",
  "missing": "closing brace"
```

### 测试工具
- **curl**: HTTP客户端测试
- **wrk**: 性能测试工具
- **docker**: 容器化测试
- **jq**: JSON处理工具

## 验收流程

### 预验收检查
1. 代码审查完成
2. 单元测试通过 (覆盖率 > 80%)
3. 集成测试通过
4. 文档完成

### 正式验收
1. Level 1测试通过 (100%)
2. Level 2测试通过 (90%+)
3. Level 3测试通过 (70%+)
4. 性能测试通过
5. 安全测试通过

### 验收标准
- **通过**: 所有Level 1测试通过，Level 2测试通过率 > 90%
- **有条件通过**: Level 1测试通过，Level 2测试通过率 > 80%，已知问题不影响核心功能
- **不通过**: Level 1测试有失败，或Level 2测试通过率 < 80%

## 问题跟踪

### 严重级别定义
- **Critical**: 阻止功能使用，需要立即修复
- **Major**: 影响主要功能，需要在发布前修复
- **Minor**: 影响次要功能，可以在后续版本修复
- **Trivial**: 不影响功能的微小问题

### 问题修复优先级
1. Critical问题：立即修复
2. Major问题：24小时内修复
3. Minor问题：下个版本修复
4. Trivial问题：可选修复

## 验收报告模板

### 测试总结
- **测试日期**: [日期]
- **测试环境**: [环境信息]
- **测试人员**: [测试人员]
- **测试版本**: [版本号]

### 测试结果
- **Level 1通过率**: [百分比]
- **Level 2通过率**: [百分比]
- **Level 3通过率**: [百分比]
- **总体通过率**: [百分比]

### 问题统计
- **Critical问题**: [数量]
- **Major问题**: [数量]
- **Minor问题**: [数量]
- **Trivial问题**: [数量]

### 验收结论
- **验收状态**: [通过/有条件通过/不通过]
- **主要发现**: [主要发现和问题]
- **建议**: [改进建议]
- **后续步骤**: [后续行动计划]

## 附录

### 测试检查清单
- [ ] 所有测试环境准备完成
- [ ] 测试数据准备完成
- [ ] 测试工具安装完成
- [ ] 测试用例编写完成
- [ ] 自动化测试脚本准备完成
- [ ] 性能测试环境配置完成
- [ ] 安全测试工具准备完成
- [ ] 验收流程确认完成

### 参考资料
- [MCP协议规范](https://modelcontextprotocol.io/)
- [JSON-RPC 2.0规范](https://www.jsonrpc.org/specification)
- [HTTP/1.1规范](https://tools.ietf.org/html/rfc7231)
- [Docker文档](https://docs.docker.com/)

### 相关文档
- [需求规格文档](./requirements.md)
- [用户故事文档](./user-stories.md)
- [技术架构文档](./architecture.md)
- [部署指南](./deployment.md)
- [API文档](./api.md)