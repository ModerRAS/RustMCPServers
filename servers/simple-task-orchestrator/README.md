# Simple Task Orchestrator MCP Server

一个简化的任务编排MCP服务器，基于Rust和Axum构建，提供基本的任务管理和调度功能。

## 🚀 特性

- **简洁架构**: 基于内存存储，易于理解和部署
- **高性能**: 异步处理 + Tokio运行时
- **任务管理**: 创建、获取、更新、删除任务
- **状态管理**: 任务状态流转和重试机制
- **并发控制**: 简单的锁机制和速率限制
- **REST API**: 完整的HTTP API接口
- **监控支持**: 健康检查和基础监控

## 📋 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   表现层 (API)   │────│   应用层 (服务)   │────│  基础设施层 (存储) │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ HTTP 处理器  │ │    │ │ 任务服务     │ │    │ │ 内存仓库     │ │
│ │ 路由        │ │    │ │ 调度器       │ │    │ │ 锁管理器     │ │
│ │ 中间件      │ │    │ │ 监控器       │ │    │ │             │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🛠️ 技术栈

- **编程语言**: Rust 1.70+
- **Web框架**: Axum 0.7
- **异步运行时**: Tokio 1.35
- **序列化**: Serde 1.0
- **错误处理**: thiserror
- **日志**: tracing + tracing-subscriber

## 🚀 快速开始

### 本地开发

1. **克隆项目**
   ```bash
   cd /root/WorkSpace/Rust/RustMCPServers/servers/simple-task-orchestrator
   ```

2. **安装依赖**
   ```bash
   # 安装Rust工具链
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # 安装开发依赖
   cargo install cargo-watch
   ```

3. **配置环境**
   ```bash
   # 设置环境变量（可选）
   export RUST_LOG=info
   export APP_SERVER_PORT=8080
   ```

4. **运行服务**
   ```bash
   # 开发模式
   cargo run
   
   # 生产模式
   cargo run --release
   ```

### 测试API

服务启动后，可以通过以下命令测试API：

```bash
# 创建任务
curl -X POST "http://localhost:8080/api/v1/tasks" \
  -H "Content-Type: application/json" \
  -d '{
    "work_directory": "/home/user/projects/my-project",
    "prompt": "Analyze the codebase and identify potential security vulnerabilities",
    "priority": "high",
    "tags": ["security", "analysis"]
  }'

# 获取下一个任务
curl -X GET "http://localhost:8080/api/v1/tasks/next?work_path=/home/user/projects/my-project&worker_id=worker-001"

# 获取任务列表
curl -X GET "http://localhost:8080/api/v1/tasks"

# 获取统计信息
curl -X GET "http://localhost:8080/api/v1/statistics"

# 健康检查
curl -X GET "http://localhost:8080/health"
```

## 📖 API文档

### 基础URL
- 开发环境: `http://localhost:8080`

### 任务管理

#### 创建任务
```http
POST /api/v1/tasks
Content-Type: application/json

{
  "work_directory": "/path/to/work",
  "prompt": "Task description",
  "priority": "high",
  "tags": ["urgent", "production"]
}
```

#### 获取下一个任务
```http
GET /api/v1/tasks/next?work_path=/path/to/work&worker_id=worker-1
```

#### 完成任务
```http
POST /api/v1/tasks/{task_id}/complete
Content-Type: application/json

{
  "original_prompt": "Task description",
  "result": {
    "status": "success",
    "output": "Task completed",
    "duration_ms": 1500
  }
}
```

#### 获取任务详情
```http
GET /api/v1/tasks/{task_id}
```

#### 列出任务
```http
GET /api/v1/tasks?status=waiting&priority=high&limit=10&offset=0
```

#### 取消任务
```http
POST /api/v1/tasks/{task_id}/cancel
Content-Type: application/json

{
  "reason": "User requested cancellation"
}
```

#### 重试任务
```http
POST /api/v1/tasks/{task_id}/retry
```

### 系统管理

#### 健康检查
```http
GET /health
```

#### 获取统计信息
```http
GET /api/v1/statistics
```

### 响应格式

所有API响应都遵循统一格式：

```json
{
  "success": true,
  "data": {
    // 响应数据
  },
  "error": null,
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## ⚙️ 配置

### 环境变量

| 变量名 | 描述 | 默认值 |
|--------|------|--------|
| `RUST_LOG` | 日志级别 | `info` |
| `APP_SERVER_HOST` | 服务器地址 | `0.0.0.0` |
| `APP_SERVER_PORT` | 服务器端口 | `8080` |
| `APP_SERVER_TIMEOUT` | 请求超时(秒) | `30` |
| `APP_TASK_MAX_RETRIES` | 最大重试次数 | `3` |
| `APP_TASK_TIMEOUT` | 任务超时(秒) | `3600` |
| `APP_SECURITY_RATE_LIMIT` | 速率限制(请求/分钟) | `1000` |

## 🔧 开发

### 项目结构

```
simple-task-orchestrator/
├── src/
│   ├── main.rs                 # 应用入口
│   ├── bin/
│   │   └── migrate.rs         # 数据库迁移工具
│   ├── domain/                # 领域层
│   ├── infrastructure/        # 基础设施层
│   ├── services/             # 应用层
│   ├── handlers/             # 表现层
│   ├── config/               # 配置管理
│   ├── errors/               # 错误处理
│   └── utils/                # 工具类
├── Cargo.toml                # 项目配置
└── README.md                 # 项目文档
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_rate_limiter
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 📊 监控

### 健康检查

服务提供健康检查端点：
```bash
curl http://localhost:8080/health
```

### 日志

服务支持结构化日志输出：
- JSON格式（生产环境）
- Pretty格式（开发环境）

日志级别：
- `ERROR`: 错误信息
- `WARN`: 警告信息
- `INFO`: 一般信息
- `DEBUG`: 调试信息

## 🚀 部署

### 直接运行

```bash
cargo build --release
./target/release/simple-task-orchestrator
```

### 使用Docker

```bash
# 构建镜像
docker build -t simple-task-orchestrator:latest .

# 运行容器
docker run -p 8080:8080 simple-task-orchestrator:latest
```

## 🎯 使用场景

这个简化的任务编排服务器特别适合：

- **开发测试**: 快速原型开发和测试
- **小规模部署**: 轻量级任务处理
- **学习参考**: 理解任务编排系统架构
- **集成测试**: 作为其他系统的测试依赖

## 🌟 主要特点

1. **简单易用**: 最小化的依赖和配置
2. **内存存储**: 无需外部数据库，启动快速
3. **类型安全**: Rust的编译时类型检查
4. **异步处理**: 高性能的异步架构
5. **RESTful API**: 标准化的HTTP接口
6. **健康检查**: 完整的系统监控

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

### 开发规范

- 遵循Rust代码风格
- 编写单元测试
- 更新文档
- 确保所有测试通过

## 📄 许可证

本项目采用 MIT 许可证。

## 🙏 致谢

- [Rust](https://www.rust-lang.org/) - 编程语言
- [Axum](https://github.com/tokio-rs/axum) - Web框架
- [Tokio](https://tokio.rs/) - 异步运行时

---

**Simple Task Orchestrator MCP Server** - 让任务管理变得简单高效！ 🚀