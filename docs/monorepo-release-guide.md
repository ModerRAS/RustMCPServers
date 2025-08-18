# Monorepo 独立构建 Release 指南

## 概述

在单个 monorepo 仓库中实现多个 MCP 服务器的独立构建和发布，确保每个服务器只发布自己的二进制文件，不会包含其他无关的代码或可执行文件。

## 核心思路

使用**前缀 tag + 单服务器 CI**的方式，在逻辑上隔离不同服务器的构建流程：

- 打 tag 时只触发对应服务器的 CI
- Release 中只上传对应服务器的二进制文件
- 各服务器之间互不干扰

## 目录结构约定

```
mcp-servers/
 ├─ servers/
 │   ├─ json-validator-server/
 │   │   ├─ Cargo.toml
 │   │   └─ src/
 │   └─ task-orchestrator/
 └─ .github/workflows/release.yml
```

## Tag 命名规则

每个服务器使用独立的 tag，通过前缀区分：

```
mcp-json-validator-v1.0.0
mcp-task-orchestrator-v1.0.0
```

格式：`mcp-{server-name}-v{version}`

## GitHub Actions 配置

### 完整工作流 (`.github/workflows/release.yml`)

```yaml
name: release

on:
  push:
    tags:
      - 'mcp-*-v*'   # 只监听带前缀的 tag

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        server: [json-validator, task-orchestrator]
        include:
          # JSON Validator Server
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            bin: json-validator
            server: json-validator
            server_dir: json-validator-server
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            bin: json-validator.exe
            server: json-validator
            server_dir: json-validator-server
          - os: macos-latest
            target: aarch64-apple-darwin
            bin: json-validator
            server: json-validator
            server_dir: json-validator-server
          # Task Orchestrator Server
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            bin: task-orchestrator
            server: task-orchestrator
            server_dir: task-orchestrator
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            bin: task-orchestrator.exe
            server: task-orchestrator
            server_dir: task-orchestrator
          - os: macos-latest
            target: aarch64-apple-darwin
            bin: task-orchestrator
            server: task-orchestrator
            server_dir: task-orchestrator

    steps:
      - uses: actions/checkout@v4

      # 1. 解析 tag 前缀，得到服务器信息
      - name: parse tag
        id: vars
        run: |
          TAG=${GITHUB_REF#refs/tags/}
          SERVER=${TAG%%-v*}
          VERSION=${TAG##*-v}
          
          # 检查是否匹配当前服务器
          if [[ "$SERVER" == "mcp-${{ matrix.server }}" ]]; then
            echo "should_build=true" >> $GITHUB_OUTPUT
            echo "server=$SERVER" >> $GITHUB_OUTPUT
            echo "server_dir=${{ matrix.server_dir }}" >> $GITHUB_OUTPUT
            echo "version=$VERSION" >> $GITHUB_OUTPUT
            echo "bin=${{ matrix.bin }}" >> $GITHUB_OUTPUT
          else
            echo "should_build=false" >> $GITHUB_OUTPUT
          fi

      # 2. 只编译匹配的服务器
      - uses: dtolnay/rust-toolchain@stable
        if: steps.vars.outputs.should_build == 'true'
        with:
          targets: ${{ matrix.target }}
      - run: |
          cd servers/${{ steps.vars.outputs.server_dir }}
          cargo build --release --bin ${{ steps.vars.outputs.bin }} --target ${{ matrix.target }}
        if: steps.vars.outputs.should_build == 'true'

      # 3. 上传 Release（只含单个文件）
      - name: upload
        uses: softprops/action-gh-release@v2
        if: steps.vars.outputs.should_build == 'true'
        with:
          tag_name: ${{ github.ref_name }}
          files: |
            servers/${{ steps.vars.outputs.server_dir }}/target/${{ matrix.target }}/release/${{ steps.vars.outputs.bin }}
```

## 服务器配置

### Cargo.toml 配置

每个服务器的 `Cargo.toml` 需要定义对应的 bin target：

```toml
[package]
name = "json-validator-server"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "json-validator"
path = "src/main.rs"
```

## 使用方法

### 发布新版本

1. 创建对应的 tag：
   ```bash
   git tag mcp-json-validator-v1.0.0
   ```
2. 推送 tag 到远程仓库：
   ```bash
   git push origin mcp-json-validator-v1.0.0
   ```

### 效果说明

- `git tag mcp-json-validator-v1.0.0` → 只有 `json-validator-server` 的 CI 触发，Release 里只出现 `json-validator` 二进制
- `git tag mcp-task-orchestrator-v1.0.0` → 只有 `task-orchestrator` 的 CI 触发，Release 里只出现 `task-orchestrator` 二进制

### 添加新服务器

要添加新的服务器到发布系统，需要：

1. **更新 matrix 配置**：在 `.github/workflows/release.yml` 的 `matrix.server` 中添加新服务器
2. **添加 include 配置**：为每个平台添加对应的配置条目
3. **确保服务器目录结构正确**：服务器必须位于 `servers/{server_dir}` 目录下
4. **配置二进制名称**：确保服务器的 `Cargo.toml` 中定义了正确的 `[[bin]]` 条目

示例：添加 `new-server` 
```yaml
matrix:
  server: [json-validator, task-orchestrator, new-server]
include:
  # ... 现有配置 ...
  # New Server
  - os: ubuntu-latest
    target: x86_64-unknown-linux-musl
    bin: new-server
    server: new-server
    server_dir: new-server
  # ... 其他平台配置 ...
```

## 本地测试

使用 `act` 工具本地测试 GitHub Actions：

```bash
# 安装 act
brew install act

# 测试工作流
act -j build -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:act-latest
```

## 注意事项

1. **GitHub Release 本身不提供过滤功能**，需要通过 CI 逻辑来实现
2. **Tag 命名必须严格遵循约定**，否则 CI 无法正确解析
3. **每个服务器的二进制名称需要预先定义**在 matrix.include 中
4. **确保每个服务器的 Cargo.toml 中正确定义了对应的 bin target**
5. **tag 解析逻辑需要根据实际项目结构调整**，特别是服务器名称与目录名的映射关系

## 扩展说明

这种方法可以扩展到任何支持标签触发 CI 的平台，不仅仅是 GitHub Actions。核心思想是：

- 通过命名约定区分不同服务
- 在 CI 中解析标签以确定目标服务
- 只构建和发布对应服务的产物

## 总结

GitHub Release 本身不帮你筛选，但用「前缀 tag + 单服务器 CI」就能在逻辑上隔离，让你在一个 monorepo 里安心地"只发需要的二进制"。