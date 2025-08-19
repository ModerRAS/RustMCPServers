# JSON Validator HTTP Server - Docker部署指南

## 🐳 Docker镜像

### GitHub Container Registry

我们的Docker镜像托管在GitHub Container Registry (ghcr.io)：

- **独立版本**: `ghcr.io/moderras/rustmcpservers:standalone-latest`
- **企业级版本**: `ghcr.io/moderras/rustmcpservers:enterprise-latest` (开发中)

### 快速开始

#### 1. 使用Docker运行独立版本

```bash
# 拉取镜像
docker pull ghcr.io/moderras/rustmcpservers:standalone-latest

# 运行容器
docker run -d \
  --name json-validator-standalone \
  -p 8082:8082 \
  -e RUST_LOG=info \
  --restart unless-stopped \
  ghcr.io/moderras/rustmcpservers:standalone-latest

# 查看日志
docker logs json-validator-standalone

# 健康检查
curl http://localhost:8082/health
```

#### 2. 使用Docker Compose

```bash
# 启动服务
docker-compose -f servers/json-validator-http/docker-compose.standalone.yml up -d

# 查看状态
docker-compose -f servers/json-validator-http/docker-compose.standalone.yml ps

# 查看日志
docker-compose -f servers/json-validator-http/docker-compose.standalone.yml logs -f

# 运行测试
docker-compose -f servers/json-validator-http/docker-compose.standalone.yml --profile test up json-validator-client

# 停止服务
docker-compose -f servers/json-validator-http/docker-compose.standalone.yml down
```

## 🔧 配置

### 环境变量

| 变量名 | 默认值 | 描述 |
|--------|--------|------|
| `RUST_LOG` | `info` | 日志级别 |
| `RUST_BACKTRACE` | `1` | 是否显示堆栈跟踪 |
| `JSON_VALIDATOR_HOST` | `127.0.0.1` | 服务器主机地址 |
| `JSON_VALIDATOR_PORT` | `8082` | 服务器端口 |
| `JSON_VALIDATOR_MAX_CONNECTIONS` | `1000` | 最大连接数 |

### 示例配置

```bash
# 生产环境配置
docker run -d \
  --name json-validator-prod \
  -p 8082:8082 \
  -e RUST_LOG=warn \
  -e JSON_VALIDATOR_HOST=0.0.0.0 \
  -e JSON_VALIDATOR_MAX_CONNECTIONS=5000 \
  --restart unless-stopped \
  ghcr.io/moderras/rustmcpservers:standalone-latest

# 开发环境配置
docker run -d \
  --name json-validator-dev \
  -p 8082:8082 \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  -v ./logs:/app/logs \
  --restart unless-stopped \
  ghcr.io/moderras/rustmcpservers:standalone-latest
```

## 📊 监控

### 健康检查

Docker镜像包含内置的健康检查：

```bash
# 手动健康检查
curl http://localhost:8082/health

# 查看容器健康状态
docker ps --format "table {{.Names}}\t{{.Status}}"

# 查看健康检查日志
docker inspect json-validator-standalone --format='{{json .State.Health}}'
```

### 日志管理

```bash
# 查看实时日志
docker logs -f json-validator-standalone

# 查看最近100行日志
docker logs --tail 100 json-validator-standalone

# 将日志保存到文件
docker logs json-validator-standalone > validator.log

# 持久化日志
docker run -d \
  --name json-validator-standalone \
  -p 8082:8082 \
  -v ./logs:/app/logs \
  ghcr.io/moderras/rustmcpservers:standalone-latest
```

## 🔒 安全

### 最佳实践

1. **使用非root用户**：镜像已配置为使用非root用户运行
2. **限制网络访问**：使用防火墙规则限制访问
3. **定期更新**：定期拉取最新镜像
4. **监控日志**：监控异常活动

### 安全配置

```bash
# 使用只读文件系统
docker run -d \
  --name json-validator-standalone \
  -p 8082:8082 \
  --read-only \
  --tmpfs /tmp \
  --tmpfs /app/logs \
  ghcr.io/moderras/rustmcpservers:standalone-latest

# 限制资源使用
docker run -d \
  --name json-validator-standalone \
  -p 8082:8082 \
  --memory=512m \
  --cpus=1.0 \
  ghcr.io/moderras/rustmcpservers:standalone-latest
```

## 🚀 部署

### Kubernetes部署

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: json-validator-standalone
spec:
  replicas: 3
  selector:
    matchLabels:
      app: json-validator-standalone
  template:
    metadata:
      labels:
        app: json-validator-standalone
    spec:
      containers:
      - name: json-validator-standalone
        image: ghcr.io/moderras/rustmcpservers:standalone-latest
        ports:
        - containerPort: 8082
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            memory: "512Mi"
            cpu: "500m"
          requests:
            memory: "256Mi"
            cpu: "250m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8082
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8082
          initialDelaySeconds: 5
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: json-validator-service
spec:
  selector:
    app: json-validator-standalone
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8082
  type: LoadBalancer
```

### Docker Swarm部署

```yaml
version: '3.8'
services:
  json-validator:
    image: ghcr.io/moderras/rustmcpservers:standalone-latest
    ports:
      - "8082:8082"
    environment:
      - RUST_LOG=info
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        delay: 10s
      restart_policy:
        condition: on-failure
        max_attempts: 3
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8082/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

## 📈 性能优化

### 缓存策略

Docker镜像使用多阶段构建和分层缓存：

```dockerfile
# 依赖项缓存层
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# 源代码层
COPY src ./src/
RUN cargo build --release
```

### 资源限制

```bash
# 生产环境资源限制
docker run -d \
  --name json-validator-standalone \
  -p 8082:8082 \
  --memory=1g \
  --cpus=2.0 \
  --memory-swap=2g \
  ghcr.io/moderras/rustmcpservers:standalone-latest
```

## 🔍 故障排除

### 常见问题

1. **容器启动失败**
   ```bash
   # 查看详细错误
   docker logs json-validator-standalone
   
   # 检查端口冲突
   netstat -tlnp | grep 8082
   ```

2. **健康检查失败**
   ```bash
   # 手动测试健康端点
   curl http://localhost:8082/health
   
   # 检查网络连接
   docker exec json-validator-standalone curl http://localhost:8082/health
   ```

3. **性能问题**
   ```bash
   # 查看资源使用情况
   docker stats json-validator-standalone
   
   # 查看容器详细信息
   docker inspect json-validator-standalone
   ```

### 调试模式

```bash
# 启用调试日志
docker run -d \
  --name json-validator-standalone \
  -p 8082:8082 \
  -e RUST_LOG=debug \
  -e RUST_BACKTRACE=1 \
  ghcr.io/moderras/rustmcpservers:standalone-latest

# 交互式调试
docker run -it \
  --entrypoint /bin/sh \
  ghcr.io/moderras/rustmcpservers:standalone-latest
```

## 📝 更新日志

### v1.0.0 (2024-01-XX)
- 🎉 初始版本发布
- ✅ 完整的JSON验证功能
- ✅ HTTP协议支持
- ✅ Docker容器化
- ✅ 自动化CI/CD
- ✅ 健康检查和监控

## 🤝 贡献

欢迎提交Issue和Pull Request！

## 📄 许可证

MIT License