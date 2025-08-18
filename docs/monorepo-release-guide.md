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
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            bin: json-validator
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            bin: json-validator.exe
          - os: macos-latest
            target: aarch64-apple-darwin
            bin: json-validator

    steps:
      - uses: actions/checkout@v4

      # 1. 解析 tag 前缀，得到服务器目录
      - name: parse tag
        id: vars
        run: |
          TAG=${GITHUB_REF#refs/tags/}
          SERVER=${TAG%%-v*}
          # 将 mcp-json-validator 转换为 json-validator-server
          SERVER_DIR=${SERVER#mcp-}
          SERVER_DIR=${SERVER_DIR%-validator}
          SERVER_DIR="${SERVER_DIR}-validator-server"
          echo "server=$SERVER" >> $GITHUB_OUTPUT
          echo "server_dir=$SERVER_DIR" >> $GITHUB_OUTPUT
          echo "version=${TAG##*-v}" >> $GITHUB_OUTPUT

      # 2. 只编译对应那一个 bin
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - run: |
          cd servers/${{ steps.vars.outputs.server_dir }}
          cargo build --release --bin ${{ matrix.bin }} --target ${{ matrix.target }}

      # 3. 上传 Release（只含单个文件）
      - name: upload
        uses: softprops/action-gh-release@v2
        with:
          tag_name: ${{ github.ref_name }}
          files: |
            servers/${{ steps.vars.outputs.server_dir }}/target/${{ matrix.target }}/release/${{ matrix.bin }}
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