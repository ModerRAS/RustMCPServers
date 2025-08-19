# 多阶段构建
FROM rust:1.70-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制依赖文件
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY config ./config

# 构建优化
ENV RUSTFLAGS="-C target-cpu=native"
RUN cargo build --release

# 运行时镜像
FROM debian:bullseye-slim

# 安装必要的运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -s /bin/false -m -d /app appuser

# 切换用户
USER appuser

# 设置工作目录
WORKDIR /app

# 复制二进制文件
COPY --from=builder /app/target/release/task-orchestrator /usr/local/bin/
COPY --from=builder /app/target/release/build/*/out/migrations ./migrations/
COPY --from=builder /app/config ./config

# 创建数据目录
RUN mkdir -p /data /logs && chown appuser:appuser /data /logs

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# 暴露端口
EXPOSE 8080

# 启动命令
CMD ["task-orchestrator"]