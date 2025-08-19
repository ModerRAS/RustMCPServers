# JSON Validator HTTP Server

HTTP协议的JSON验证MCP服务器，提供企业级的JSON验证功能。

## 特性

- **HTTP协议支持**: 基于Axum框架的高性能HTTP服务器
- **JSON-RPC 2.0**: 标准的JSON-RPC over HTTP协议
- **企业级安全**: JWT认证、CORS、限流、IP白名单
- **高性能缓存**: 支持Redis和LRU缓存
- **监控指标**: Prometheus指标收集和Grafana仪表板
- **容器化部署**: Docker和Docker Compose支持
- **可扩展架构**: 水平扩展和负载均衡支持

## 快速开始

### 本地开发

1. 克隆项目
```bash
git clone https://github.com/RustMCPServers/RustMCPServers.git
cd RustMCPServers/servers/json-validator-http
```

2. 构建项目
```bash
cargo build --release
```

3. 运行服务器
```bash
cargo run --release
```

### Docker部署

1. 构建Docker镜像
```bash
docker build -t json-validator-http .
```

2. 运行容器
```bash
docker run -p 8080:8080 -p 9090:9090 json-validator-http
```

### Docker Compose部署

1. 启动所有服务
```bash
docker-compose up -d
```

2. 查看日志
```bash
docker-compose logs -f
```

3. 停止服务
```bash
docker-compose down
```

## API文档

### 端点

#### JSON-RPC端点
- **URL**: `/rpc`
- **方法**: POST
- **内容类型**: `application/json`

#### 健康检查
- **URL**: `/health`
- **方法**: GET

#### 服务器信息
- **URL**: `/info`
- **方法**: GET

#### 指标
- **URL**: `/metrics`
- **方法**: GET

### JSON-RPC方法

#### validate_json
验证JSON数据的基本格式。

**请求示例**:
```json
{
  "jsonrpc": "2.0",
  "method": "validate_json",
  "params": {
    "json_data": {
      "name": "John Doe",
      "age": 30
    },
    "options": {
      "strict_mode": false,
      "detailed_errors": true
    }
  },
  "id": 1
}
```

**响应示例**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "errors": [],
    "warnings": [],
    "execution_time": 5,
    "cache_hit": false
  },
  "id": 1
}
```

#### validate_json_with_schema
使用JSON Schema验证JSON数据。

**请求示例**:
```json
{
  "jsonrpc": "2.0",
  "method": "validate_json_with_schema",
  "params": {
    "json_data": {
      "name": "John Doe",
      "age": 30
    },
    "schema": {
      "type": "object",
      "properties": {
        "name": {"type": "string"},
        "age": {"type": "number"}
      },
      "required": ["name", "age"]
    },
    "options": {
      "strict_mode": false,
      "detailed_errors": true
    }
  },
  "id": 1
}
```

#### validate_json_batch
批量验证多个JSON数据。

**请求示例**:
```json
{
  "jsonrpc": "2.0",
  "method": "validate_json_batch",
  "params": {
    "items": [
      {
        "id": "1",
        "json_data": {"name": "Item 1"}
      },
      {
        "id": "2",
        "json_data": {"name": "Item 2"}
      }
    ],
    "options": {
      "strict_mode": false,
      "detailed_errors": true
    }
  },
  "id": 1
}
```

## 配置

### 配置文件

服务器使用TOML格式的配置文件，默认位置为`config/default.toml`。

主要配置项：

```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 4
max_connections = 1000
timeout = 30

[cache]
enabled = false
cache_type = "lru"
ttl = 3600
max_size = 1000

[security]
enabled = false
jwt_secret = "your-secret-key-here"
rate_limit = 100

[metrics]
enabled = true
port = 9090
```

### 环境变量

可以通过环境变量覆盖配置文件中的设置：

```bash
JSON_VALIDATOR_SERVER__PORT=9090
JSON_VALIDATOR_CACHE__ENABLED=true
JSON_VALIDATOR_SECURITY__ENABLED=true
```

## 监控

### Prometheus指标

服务器提供以下Prometheus指标：

- `http_requests_total`: HTTP请求总数
- `http_requests_success_total`: 成功请求数
- `http_requests_failed_total`: 失败请求数
- `http_response_time_seconds`: 响应时间分布
- `json_validations_total`: JSON验证总数
- `cache_hits_total`: 缓存命中数
- `cache_misses_total`: 缓存未命中数

### Grafana仪表板

使用Docker Compose部署时，会自动启动Grafana服务：

- **地址**: http://localhost:3000
- **用户名**: admin
- **密码**: admin

预配置的仪表板包括：
- 请求指标
- 验证指标
- 缓存指标
- 性能指标

## 性能优化

### 缓存策略

1. **内存缓存**: 编译后的Schema缓存
2. **Redis缓存**: 验证结果缓存
3. **LRU缓存**: 本地内存缓存

### 性能调优

1. **工作线程数**: 根据CPU核心数调整`workers`配置
2. **连接数**: 根据内存大小调整`max_connections`
3. **缓存大小**: 根据可用内存调整缓存大小
4. **超时设置**: 根据业务需求调整超时时间

## 安全配置

### 认证和授权

1. **JWT认证**: 配置`jwt_secret`启用JWT认证
2. **IP白名单**: 配置`allowed_ips`限制访问IP
3. **限流**: 配置`rate_limit`防止滥用

### HTTPS配置

建议在生产环境中使用HTTPS：

```nginx
server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## 故障排除

### 常见问题

1. **端口占用**: 检查端口是否被其他服务占用
2. **权限问题**: 确保有足够的权限创建日志文件
3. **内存不足**: 调整缓存大小和工作线程数
4. **Redis连接失败**: 检查Redis服务状态和网络连接

### 日志分析

启用详细日志：

```bash
RUST_LOG=debug cargo run
```

### 性能分析

使用内置的指标端点分析性能：

```bash
curl http://localhost:9090/metrics
```

## 开发

### 构建和测试

```bash
# 构建
cargo build

# 运行测试
cargo test

# 运行集成测试
cargo test -- --ignored

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 贡献

1. Fork项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建Pull Request

## 许可证

MIT License

## 联系方式

- 项目地址: https://github.com/RustMCPServers/RustMCPServers
- 问题反馈: https://github.com/RustMCPServers/RustMCPServers/issues