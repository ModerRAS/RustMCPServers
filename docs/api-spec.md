# JSON验证MCP服务器API规格

## 概述

本文档定义了JSON验证MCP服务器的HTTP API规格，基于JSON-RPC 2.0协议。该API提供了完整的JSON Schema验证功能，支持标准HTTP协议。

## 基本信息

- **基础URL**: `https://api.example.com/v1`
- **协议**: HTTPS
- **数据格式**: JSON
- **认证**: Bearer Token (JWT)
- **编码**: UTF-8

## 通用响应格式

### 成功响应
```json
{
  "jsonrpc": "2.0",
  "result": {
    "success": true,
    "data": {},
    "metadata": {
      "request_id": "uuid",
      "timestamp": "2024-01-01T00:00:00Z",
      "execution_time": 125
    }
  },
  "id": 1
}
```

### 错误响应
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32600,
    "message": "Invalid Request",
    "data": {
      "details": "Invalid JSON format",
      "request_id": "uuid"
    }
  },
  "id": 1
}
```

## 认证

### Bearer Token认证
所有API请求都需要在Authorization头中包含Bearer Token：

```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## API端点

### 1. JSON-RPC端点

#### POST /rpc
**描述**: 主要的JSON-RPC端点，处理所有MCP协议调用

**请求头**:
```http
POST /rpc HTTP/1.1
Host: api.example.com
Content-Type: application/json
Authorization: Bearer <token>
X-Request-ID: uuid
```

**请求体**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "validate_json",
    "arguments": {
      "json_data": {
        "name": "John Doe",
        "age": 30,
        "email": "john@example.com"
      },
      "schema": {
        "type": "object",
        "properties": {
          "name": {"type": "string"},
          "age": {"type": "integer", "minimum": 0},
          "email": {"type": "string", "format": "email"}
        },
        "required": ["name", "email"]
      },
      "options": {
        "strict_mode": true,
        "allow_additional_properties": false,
        "custom_formats": {}
      }
    }
  },
  "id": 1
}
```

**成功响应**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": true,
    "errors": [],
    "warnings": [],
    "execution_time": 1250,
    "cache_hit": false,
    "schema_hash": "a1b2c3d4e5f6",
    "data_hash": "f6e5d4c3b2a1"
  },
  "id": 1
}
```

**验证失败响应**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "valid": false,
    "errors": [
      {
        "instance_path": "/age",
        "schema_path": "/properties/age/minimum",
        "message": "age must be greater than or equal to 0",
        "error_code": "minimum",
        "details": {
          "expected": 0,
          "actual": -5
        }
      }
    ],
    "warnings": [],
    "execution_time": 890,
    "cache_hit": false
  },
  "id": 1
}
```

**批量验证请求**:
```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "validate_batch",
    "arguments": {
      "validations": [
        {
          "json_data": {"name": "Alice", "age": 25},
          "schema": {"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}
        },
        {
          "json_data": {"name": "Bob", "age": "thirty"},
          "schema": {"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}
        }
      ]
    }
  },
  "id": 2
}
```

**批量验证响应**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "results": [
      {
        "valid": true,
        "errors": [],
        "warnings": [],
        "execution_time": 450
      },
      {
        "valid": false,
        "errors": [
          {
            "instance_path": "/age",
            "schema_path": "/properties/age/type",
            "message": "age must be integer",
            "error_code": "type",
            "details": {
              "expected": "integer",
              "actual": "string"
            }
          }
        ],
        "warnings": [],
        "execution_time": 320
      }
    ],
    "total_execution_time": 770
  },
  "id": 2
}
```

### 2. 健康检查端点

#### GET /health
**描述**: 检查服务健康状态

**请求头**:
```http
GET /health HTTP/1.1
Host: api.example.com
```

**成功响应**:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "1.0.0",
  "uptime": 86400,
  "components": {
    "database": "healthy",
    "cache": "healthy",
    "monitoring": "healthy"
  },
  "metrics": {
    "request_count": 15000,
    "error_rate": 0.001,
    "avg_response_time": 45
  }
}
```

**服务不可用响应**:
```json
{
  "status": "unhealthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "error": "Database connection failed",
  "components": {
    "database": "unhealthy",
    "cache": "healthy",
    "monitoring": "healthy"
  }
}
```

### 3. 指标端点

#### GET /metrics
**描述**: Prometheus格式的指标

**请求头**:
```http
GET /metrics HTTP/1.1
Host: api.example.com
```

**响应格式** (text/plain):
```
# HELP json_validator_requests_total Total number of validation requests
# TYPE json_validator_requests_total counter
json_validator_requests_total{method="validate_json",status="success"} 15000
json_validator_requests_total{method="validate_json",status="error"} 15

# HELP json_validator_request_duration_seconds Request duration in seconds
# TYPE json_validator_request_duration_seconds histogram
json_validator_request_duration_seconds_bucket{le="0.1"} 12000
json_validator_request_duration_seconds_bucket{le="0.5"} 14800
json_validator_request_duration_seconds_bucket{le="1.0"} 14950
json_validator_request_duration_seconds_bucket{le="5.0"} 15000
json_validator_request_duration_seconds_bucket{le="+Inf"} 15000
json_validator_request_duration_seconds_sum 675.0
json_validator_request_duration_seconds_count 15000

# HELP json_validator_cache_hits_total Cache hit count
# TYPE json_validator_cache_hits_total counter
json_validator_cache_hits_total{cache_type="schema"} 8000
json_validator_cache_hits_total{cache_type="validation"} 5000
```

### 4. Schema管理端点

#### GET /schemas
**描述**: 获取所有Schema列表

**请求头**:
```http
GET /schemas HTTP/1.1
Host: api.example.com
Authorization: Bearer <token>
```

**查询参数**:
- `page`: 页码 (默认: 1)
- `limit`: 每页数量 (默认: 20, 最大: 100)
- `search`: 搜索关键词 (可选)

**成功响应**:
```json
{
  "schemas": [
    {
      "id": "user-schema",
      "name": "User Schema",
      "version": "1.0.0",
      "description": "User data validation schema",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z",
      "usage_count": 1500
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1,
    "pages": 1
  }
}
```

#### GET /schemas/{id}
**描述**: 获取特定Schema

**请求头**:
```http
GET /schemas/user-schema HTTP/1.1
Host: api.example.com
Authorization: Bearer <token>
```

**成功响应**:
```json
{
  "id": "user-schema",
  "name": "User Schema",
  "version": "1.0.0",
  "description": "User data validation schema",
  "schema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "age": {"type": "integer", "minimum": 0},
      "email": {"type": "string", "format": "email"}
    },
    "required": ["name", "email"]
  },
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z",
  "usage_count": 1500
}
```

#### PUT /schemas/{id}
**描述**: 创建或更新Schema

**请求头**:
```http
PUT /schemas/user-schema HTTP/1.1
Host: api.example.com
Authorization: Bearer <token>
Content-Type: application/json
```

**请求体**:
```json
{
  "name": "User Schema",
  "description": "User data validation schema",
  "schema": {
    "type": "object",
    "properties": {
      "name": {"type": "string"},
      "age": {"type": "integer", "minimum": 0},
      "email": {"type": "string", "format": "email"}
    },
    "required": ["name", "email"]
  }
}
```

**成功响应**:
```json
{
  "id": "user-schema",
  "name": "User Schema",
  "version": "1.0.0",
  "description": "User data validation schema",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

#### DELETE /schemas/{id}
**描述**: 删除Schema

**请求头**:
```http
DELETE /schemas/user-schema HTTP/1.1
Host: api.example.com
Authorization: Bearer <token>
```

**成功响应**:
```json
{
  "message": "Schema deleted successfully",
  "id": "user-schema"
}
```

## 数据类型

### ValidationRequest
```json
{
  "type": "object",
  "properties": {
    "json_data": {
      "type": "object",
      "description": "要验证的JSON数据"
    },
    "schema": {
      "type": "object",
      "description": "JSON Schema定义"
    },
    "options": {
      "type": "object",
      "properties": {
        "strict_mode": {
          "type": "boolean",
          "default": true,
          "description": "严格模式，不允许额外属性"
        },
        "allow_additional_properties": {
          "type": "boolean",
          "default": false,
          "description": "是否允许额外属性"
        },
        "custom_formats": {
          "type": "object",
          "description": "自定义格式验证器"
        }
      }
    },
    "cache_key": {
      "type": "string",
      "description": "可选的缓存键"
    }
  },
  "required": ["json_data", "schema"]
}
```

### ValidationResponse
```json
{
  "type": "object",
  "properties": {
    "valid": {
      "type": "boolean",
      "description": "验证是否通过"
    },
    "errors": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "instance_path": {
            "type": "string",
            "description": "JSON数据中的错误路径"
          },
          "schema_path": {
            "type": "string",
            "description": "Schema中的错误路径"
          },
          "message": {
            "type": "string",
            "description": "错误消息"
          },
          "error_code": {
            "type": "string",
            "description": "错误代码"
          },
          "details": {
            "type": "object",
            "description": "错误详细信息"
          }
        }
      }
    },
    "warnings": {
      "type": "array",
      "description": "验证警告"
    },
    "execution_time": {
      "type": "integer",
      "description": "执行时间（毫秒）"
    },
    "cache_hit": {
      "type": "boolean",
      "description": "是否命中缓存"
    }
  }
}
```

### BatchValidationRequest
```json
{
  "type": "object",
  "properties": {
    "validations": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "json_data": {
            "type": "object",
            "description": "要验证的JSON数据"
          },
          "schema": {
            "type": "object",
            "description": "JSON Schema定义"
          },
          "options": {
            "type": "object",
            "description": "验证选项"
          }
        },
        "required": ["json_data", "schema"]
      }
    }
  },
  "required": ["validations"]
}
```

## 错误代码

### JSON-RPC标准错误代码
- `-32700`: Parse error - 无效的JSON
- `-32600`: Invalid Request - 无效的请求
- `-32601`: Method not found - 方法不存在
- `-32602`: Invalid params - 无效的参数
- `-32603`: Internal error - 内部错误

### 自定义错误代码
- `-32000`: Validation error - 验证错误
- `-32001`: Schema compilation error - Schema编译错误
- `-32002`: Cache error - 缓存错误
- `-32003`: Rate limit exceeded - 请求频率超限
- `-32004`: Authentication failed - 认证失败
- `-32005`: Authorization failed - 授权失败
- `-32006`: Request timeout - 请求超时
- `-32007`: Payload too large - 请求体过大

### 错误响应示例
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Validation error",
    "data": {
      "type": "validation_error",
      "details": "Invalid JSON data format",
      "validation_errors": [
        {
          "instance_path": "/email",
          "schema_path": "/properties/email/format",
          "message": "Invalid email format",
          "error_code": "format"
        }
      ]
    }
  },
  "id": 1
}
```

## 请求限制

### 限流规则
- **认证用户**: 1000请求/分钟
- **匿名用户**: 100请求/分钟
- **批量验证**: 每批次最多100个验证项

### 请求大小限制
- **单个请求**: 最大1MB
- **批量请求**: 最大10MB
- **Schema大小**: 最大100KB

### 超时设置
- **请求超时**: 30秒
- **连接超时**: 10秒
- **读取超时**: 20秒

## 版本控制

### API版本
- **当前版本**: v1
- **支持策略**: 向后兼容
- **弃用通知**: 提前6个月通知

### 版本头
```http
X-API-Version: 1.0.0
```

## 响应头

### 标准响应头
```http
Content-Type: application/json
Content-Length: 1234
X-Request-ID: uuid
X-Response-Time: 125ms
X-Cache: HIT/MISS
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

## WebSocket支持 (可选)

### 连接端点
```
wss://api.example.com/ws
```

### 消息格式
```json
{
  "type": "validation_request",
  "id": "uuid",
  "payload": {
    "json_data": {},
    "schema": {}
  }
}
```

## 示例代码

### cURL示例
```bash
# 基本验证
curl -X POST https://api.example.com/v1/rpc \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-token" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tools/call",
    "params": {
      "name": "validate_json",
      "arguments": {
        "json_data": {"name": "John", "age": 30},
        "schema": {"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}}
      }
    },
    "id": 1
  }'

# 健康检查
curl -X GET https://api.example.com/v1/health

# 获取指标
curl -X GET https://api.example.com/v1/metrics
```

### Python示例
```python
import requests
import json

# 基本验证
def validate_json(json_data, schema, token):
    url = "https://api.example.com/v1/rpc"
    headers = {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {token}"
    }
    
    payload = {
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "validate_json",
            "arguments": {
                "json_data": json_data,
                "schema": schema
            }
        },
        "id": 1
    }
    
    response = requests.post(url, headers=headers, json=payload)
    return response.json()

# 使用示例
result = validate_json(
    {"name": "John", "age": 30},
    {"type": "object", "properties": {"name": {"type": "string"}, "age": {"type": "integer"}}},
    "your-token"
)
print(result)
```

### JavaScript示例
```javascript
// 基本验证
async function validateJson(jsonData, schema, token) {
    const response = await fetch('https://api.example.com/v1/rpc', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${token}`
        },
        body: JSON.stringify({
            jsonrpc: '2.0',
            method: 'tools/call',
            params: {
                name: 'validate_json',
                arguments: {
                    json_data: jsonData,
                    schema: schema
                }
            },
            id: 1
        })
    });
    
    return await response.json();
}

// 使用示例
validateJson(
    {name: 'John', age: 30},
    {type: 'object', properties: {name: {type: 'string'}, age: {type: 'integer'}}},
    'your-token'
).then(console.log);
```