# Task Orchestrator MCP Server

一个高性能的任务编排MCP服务器，基于Rust和Axum构建，提供可靠的任务管理和调度功能。

## 🚀 特性

- **高性能**: 基于Rust + Tokio + Axum，提供卓越的性能和并发处理能力
- **可靠性**: SQLite事务保证 + 乐观锁机制 + 错误恢复
- **高并发**: 异步架构 + 连接池 + 资源管理
- **可扩展**: 清洁架构设计 + 模块化 + 水平扩展支持
- **易维护**: 完整的日志记录 + 监控指标 + 健康检查
- **生产就绪**: Docker容器化 + Kubernetes部署 + CI/CD支持

## 📋 系统架构

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   表现层 (API)   │────│   应用层 (服务)   │────│  基础设施层 (DB) │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ HTTP 处理器  │ │    │ │ 任务服务     │ │    │ │ SQLite      │ │
│ │ 路由        │ │    │ │ 调度器       │ │ │    │ │ 连接池      │ │
│ │ 中间件      │ │    │ │ 验证器       │ │ │    │ │ 迁移        │ │
│ └─────────────┘ │    │ │ 监控器       │ │ │    │ └─────────────┘ │
└─────────────────┘    │ └─────────────┘ │    └─────────────────┘
                       └─────────────────┘
```

## 🛠️ 技术栈

- **编程语言**: Rust 1.70+
- **Web框架**: Axum 0.7
- **异步运行时**: Tokio 1.35
- **数据库**: SQLite 3.35+ (WAL模式)
- **ORM**: SQLx 0.7 (编译时检查)
- **序列化**: Serde 1.0
- **错误处理**: thiserror + anyhow
- **日志**: tracing + tracing-subscriber
- **监控**: Prometheus + Grafana
- **部署**: Docker + Kubernetes

## 🚀 快速开始

### 本地开发

1. **克隆项目**
   ```bash
   git clone https://github.com/your-org/task-orchestrator.git
   cd task-orchestrator
   ```

2. **安装依赖**
   ```bash
   # 安装Rust工具链
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # 安装开发依赖
   cargo install cargo-watch cargo-audit
   ```

3. **配置环境**
   ```bash
   # 复制配置文件
   cp config/default.toml config/local.toml
   
   # 修改本地配置
   vim config/local.toml
   ```

4. **运行数据库迁移**
   ```bash
   cargo run --bin migrate
   ```

5. **启动服务**
   ```bash
   # 开发模式
   cargo watch -x run
   
   # 生产模式
   cargo run --release
   ```

### Docker运行

1. **构建镜像**
   ```bash
   docker build -t task-orchestrator:latest .
   ```

2. **运行容器**
   ```bash
   docker run -p 8080:8080 \
     -v $(pwd)/data:/data \
     -v $(pwd)/logs:/logs \
     task-orchestrator:latest
   ```

### Docker Compose

1. **启动服务**
   ```bash
   docker-compose up -d
   ```

2. **查看日志**
   ```bash
   docker-compose logs -f task-orchestrator
   ```

3. **停止服务**
   ```bash
   docker-compose down
   ```

## 📖 API文档

### 基础URL
- 开发环境: `http://localhost:8080`
- 生产环境: `https://your-domain.com`

### 认证
所有API请求都需要在Header中包含API密钥：
```
Authorization: Bearer your-api-key
```

### 端点

#### 任务管理

##### 创建任务
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

##### 获取下一个任务
```http
GET /api/v1/tasks/next?work_path=/path/to/work&worker_id=worker-1
```

##### 完成任务
```http
POST /api/v1/tasks/{task_id}/complete
Content-Type: application/json

{
  "original_prompt": "Task description",
  "result": {
    "status": "success",
    "output": "Task completed",
    "duration": 1500
  }
}
```

##### 获取任务详情
```http
GET /api/v1/tasks/{task_id}
```

##### 列出任务
```http
GET /api/v1/tasks?status=waiting&priority=high&limit=10&offset=0
```

##### 取消任务
```http
POST /api/v1/tasks/{task_id}/cancel
Content-Type: application/json

{
  "reason": "User requested cancellation"
}
```

##### 重试任务
```http
POST /api/v1/tasks/{task_id}/retry
```

#### 系统管理

##### 健康检查
```http
GET /health
```

##### 获取统计信息
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

错误响应：
```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "field": "work_directory",
      "message": "Work directory must be an absolute path"
    }
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## ⚙️ 配置

### 环境变量

| 变量名 | 描述 | 默认值 |
|--------|------|--------|
| `APP_ENV` | 运行环境 | `development` |
| `RUST_LOG` | 日志级别 | `info` |
| `APP_DATABASE_URL` | 数据库连接字符串 | `sqlite:///data/tasks.db` |
| `APP_SERVER_HOST` | 服务器地址 | `0.0.0.0` |
| `APP_SERVER_PORT` | 服务器端口 | `8080` |

### 配置文件

配置文件位于 `config/` 目录：

- `default.toml`: 默认配置
- `local.toml`: 本地开发配置
- `production.toml`: 生产环境配置

### 数据库配置

```toml
[database]
url = "sqlite:///data/tasks.db"
max_connections = 100
min_connections = 10
enable_wal_mode = true
busy_timeout = 30
```

### 任务配置

```toml
[task]
max_concurrent_tasks = 100
default_task_timeout = 3600
max_task_retries = 3
task_cleanup_interval = 3600
```

## 🔧 开发

### 项目结构

```
task-orchestrator/
├── src/
│   ├── main.rs                 # 应用入口
│   ├── bin/
│   │   └── migrate.rs         # 数据库迁移工具
│   ├── domain/                # 领域层
│   ├── infrastructure/        # 基础设施层
│   ├── services/             # 应用层
│   ├── handlers/             # 表现层
│   ├── models/               # 数据模型
│   ├── config/               # 配置管理
│   ├── errors/               # 错误处理
│   └── utils/                # 工具类
├── migrations/               # 数据库迁移
├── config/                   # 配置文件
├── tests/                    # 测试
├── Dockerfile               # Docker配置
├── docker-compose.yml        # Docker Compose
└── k8s/                     # Kubernetes配置
```

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行集成测试
cargo test --test integration_tests

# 运行基准测试
cargo bench
```

### 代码检查

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 安全检查
cargo audit
```

## 📊 监控

### 健康检查

服务提供健康检查端点：
```bash
curl http://localhost:8080/health
```

响应：
```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T00:00:00Z",
  "version": "1.0.0",
  "uptime": "1h30m",
  "components": {
    "database": {
      "healthy": true,
      "response_time": 5.2,
      "last_checked": "2024-01-01T00:00:00Z"
    }
  },
  "metrics": {
    "memory_usage": "78MB",
    "cpu_usage": 12.5,
    "active_connections": 15,
    "queue_size": 0
  }
}
```

### Prometheus指标

服务暴露Prometheus格式的指标：
```bash
curl http://localhost:8080/metrics
```

主要指标：
- `task_created_total`: 创建的任务总数
- `task_completed_total`: 完成的任务总数
- `task_failed_total`: 失败的任务总数
- `response_time_seconds`: 响应时间分布
- `active_tasks`: 当前活跃任务数

### 日志

服务支持结构化日志输出：
- JSON格式（生产环境）
- Pretty格式（开发环境）

日志级别：
- `ERROR`: 错误信息
- `WARN`: 警告信息
- `INFO`: 一般信息
- `DEBUG`: 调试信息
- `TRACE`: 追踪信息

## 🚀 部署

### Kubernetes

1. **部署到Kubernetes**
   ```bash
   kubectl apply -f k8s/
   ```

2. **查看部署状态**
   ```bash
   kubectl get pods -l app=task-orchestrator
   kubectl get svc task-orchestrator-service
   ```

3. **查看日志**
   ```bash
   kubectl logs -f deployment/task-orchestrator
   ```

### 水平扩展

服务支持自动水平扩展：
- 基于CPU使用率（目标70%）
- 基于内存使用率（目标80%）
- 最小副本数：3
- 最大副本数：10

### 负载均衡

Kubernetes Service提供负载均衡：
- 类型：ClusterIP
- 端口：80 -> 8080
- 选择器：app=task-orchestrator

## 🔒 安全

### API认证

- API密钥认证
- 速率限制（默认1000请求/分钟）
- CORS配置

### 数据安全

- SQLite WAL模式
- 外键约束
- 数据加密（可选）

### 网络安全

- HTTPS支持
- 防火墙规则
- 网络策略（Kubernetes）

## 📈 性能

### 基准测试

运行基准测试：
```bash
cargo bench
```

预期性能指标：
- QPS: 10,000+
- 响应时间: < 100ms (P99)
- 内存使用: < 128MB
- CPU使用: < 50% (100 QPS)

### 优化策略

1. **数据库优化**
   - SQLite WAL模式
   - 连接池配置
   - 索引优化

2. **缓存策略**
   - 内存缓存热点数据
   - 查询结果缓存

3. **并发控制**
   - 异步处理
   - 乐观锁机制
   - 资源池管理

## 🤝 贡献

欢迎贡献代码！请遵循以下步骤：

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

### 开发规范

- 遵循Rust代码风格
- 编写单元测试和集成测试
- 更新文档
- 确保所有测试通过

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Rust](https://www.rust-lang.org/) - 编程语言
- [Axum](https://github.com/tokio-rs/axum) - Web框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [SQLite](https://www.sqlite.org/) - 数据库
- [Prometheus](https://prometheus.io/) - 监控系统

## 📞 支持

如果您遇到问题或有建议，请：

1. 查看 [Issues](https://github.com/your-org/task-orchestrator/issues)
2. 创建新的 Issue
3. 联系维护团队

---

**Task Orchestrator MCP Server** - 让任务管理变得简单高效！ 🚀