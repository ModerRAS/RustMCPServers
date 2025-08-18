# MCP服务器GitHub Actions配置指南

本文档为开发者提供在添加新的MCP服务器时配置GitHub Actions自动发布的完整指南。

## 目录

1. [快速开始](#快速开始)
2. [配置步骤](#配置步骤)
3. [通用模板](#通用模板)
4. [检查清单](#检查清单)
5. [常见问题](#常见问题)
6. [故障排除](#故障排除)

## 快速开始

### 1. 前置条件
- 确保已配置好Cloudflare R2（参考 `docs/cloudflare-r2-apt-deployment.md`）
- 在GitHub仓库中已添加R2相关的Secrets

### 2. 基本流程
1. 在 `servers/` 目录下创建新的Rust项目
2. 配置项目的 `Cargo.toml` 文件
3. 创建GitHub Actions工作流
4. 测试构建和发布

## 配置步骤

### 步骤1：项目结构

在 `servers/` 目录下创建新项目：
```
servers/
├── your-server-name/
│   ├── Cargo.toml
│   ├── src/
│   │   └── main.rs
│   └── README.md
```

### 步骤2：配置Cargo.toml

```toml
[package]
name = "your-server-name"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
description = "A brief description of your MCP server"

[[bin]]
name = "your-binary-name"
path = "src/main.rs"

[dependencies]
# 添加你的依赖

[dev-dependencies]
# 添加开发依赖

[package.metadata.deb]
name = "mcp-your-server-name"
maintainer = "Your Name <your.email@example.com>"
copyright = "2024, Your Name"
license-file = ["../../LICENSE", "0"]
depends = "$auto"
section = "utils"
priority = "optional"
assets = [
    ["target/release/your-binary-name", "usr/bin/", "755"],
    ["README.md", "usr/share/doc/mcp-your-server-name/", "644"],
]
```

### 步骤3：创建GitHub Actions工作流

在 `.github/workflows/` 目录下创建 `apt-r2.yml` 文件：

```yaml
name: apt-r2

on:
  push:
    tags:
      - 'mcp-your-server-name-v*'   # 匹配你的服务器名称

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # 1. 打包 deb
      - run: |
          cd servers/your-server-name
          cargo install cargo-deb
          cargo deb

      # 2. 上传
      - uses: unfor19/install-aws-cli-action@v1
        with:
          version: 2.22.35
      - run: |
          aws configure set region auto
          aws s3 cp servers/your-server-name/target/debian/*.deb \
             s3://${{ secrets.R2_BUCKET_NAME }}/pool/main/ \
             --endpoint-url https://${{ secrets.R2_ACCOUNT_ID }}.r2.cloudflarestorage.com \
             --checksum-algorithm CRC32
```

## 通用模板

### 模板1：标准MCP服务器
```yaml
name: apt-r2

on:
  push:
    tags:
      - 'mcp-SERVER_NAME-v*'  # 替换SERVER_NAME为你的服务器名称

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build deb package
        run: |
          cd servers/SERVER_NAME
          cargo install cargo-deb
          cargo deb

      - name: Setup AWS CLI
        uses: unfor19/install-aws-cli-action@v1
        with:
          version: 2.22.35

      - name: Upload to R2
        run: |
          aws configure set region auto
          aws s3 cp servers/SERVER_NAME/target/debian/*.deb \
             s3://${{ secrets.R2_BUCKET_NAME }}/pool/main/ \
             --endpoint-url https://${{ secrets.R2_ACCOUNT_ID }}.r2.cloudflarestorage.com \
             --checksum-algorithm CRC32
```

### 模板2：带测试的MCP服务器
```yaml
name: apt-r2

on:
  push:
    tags:
      - 'mcp-SERVER_NAME-v*'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: |
          cd servers/SERVER_NAME
          cargo test

  deploy:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build deb package
        run: |
          cd servers/SERVER_NAME
          cargo install cargo-deb
          cargo deb

      - name: Setup AWS CLI
        uses: unfor19/install-aws-cli-action@v1
        with:
          version: 2.22.35

      - name: Upload to R2
        run: |
          aws configure set region auto
          aws s3 cp servers/SERVER_NAME/target/debian/*.deb \
             s3://${{ secrets.R2_BUCKET_NAME }}/pool/main/ \
             --endpoint-url https://${{ secrets.R2_ACCOUNT_ID }}.r2.cloudflarestorage.com \
             --checksum-algorithm CRC32
```

## 检查清单

### ✅ 项目配置检查
- [ ] 项目目录结构正确
- [ ] `Cargo.toml` 包含所有必需字段
- [ ] `description` 字段已添加
- [ ] `license-file` 路径正确（相对于项目目录）
- [ ] 二进制文件名称正确
- [ ] deb包配置中的assets路径正确

### ✅ GitHub Actions检查
- [ ] 工作流文件位置正确：`.github/workflows/apt-r2.yml`
- [ ] tag匹配模式正确：`mcp-SERVER_NAME-v*`
- [ ] 项目目录路径正确：`cd servers/SERVER_NAME`
- [ ] deb包构建命令正确
- [ ] R2上传配置正确

### ✅ 发布流程检查
- [ ] tag格式正确：`mcp-SERVER_NAME-vX.Y.Z`
- [ ] GitHub Secrets已配置
- [ ] R2桶权限正确
- [ ] 版本号符合语义化版本规范

## 常见问题

### Q1: 如何命名我的服务器？
A1: 使用小写字母、数字和连字符，例如：
- `mcp-json-validator-v1.0.0`
- `mcp-task-orchestrator-v2.1.0`

### Q2: 如何设置版本号？
A2: 使用语义化版本规范（Semantic Versioning）：
- `v1.0.0` - 主版本号.次版本号.修订号
- 主版本号：不兼容的API修改
- 次版本号：向下兼容的功能性新增
- 修订号：向下兼容的问题修正

### Q3: 如何测试构建过程？
A3: 在本地运行：
```bash
cd servers/your-server-name
cargo install cargo-deb
cargo deb
```

### Q4: 如何添加额外的构建步骤？
A4: 在GitHub Actions工作流中添加新的step：
```yaml
- name: Additional build step
  run: |
    cd servers/your-server-name
    # 你的命令
```

## 故障排除

### 问题1：deb包构建失败
**错误信息：** `unable to read license file: LICENSE`

**解决方案：**
- 检查 `license-file` 路径是否正确
- 确保LICENSE文件存在于根目录
- 使用相对路径：`"../../LICENSE"`

### 问题2：tag匹配失败
**错误信息：** 工作流没有触发

**解决方案：**
- 检查tag格式是否正确
- 确认工作流中的匹配模式
- 使用 `git tag -l` 查看本地tags

### 问题3：R2上传失败
**错误信息：** AWS凭证错误

**解决方案：**
- 检查GitHub Secrets配置
- 确认R2 API Token权限
- 验证R2桶名称和Account ID

### 问题4：二进制文件路径错误
**错误信息：** 文件不存在

**解决方案：**
- 确认 `[[bin]]` 配置正确
- 检查二进制文件是否在 `target/release/` 目录
- 验证deb包assets配置

## 最佳实践

1. **版本管理**
   - 使用Git tag管理版本
   - 遵循语义化版本规范
   - 在发布前进行充分测试

2. **文档维护**
   - 为每个服务器创建README.md
   - 更新CHANGELOG.md
   - 记录配置变更

3. **质量保证**
   - 在工作流中添加测试步骤
   - 使用cargo clippy进行代码检查
   - 确保构建可重复

4. **发布流程**
   - 使用GitHub Actions自动发布
   - 定期清理旧版本包
   - 监控存储使用情况

## 参考资源

- [cargo-deb文档](https://github.com/kornelski/cargo-deb)
- [GitHub Actions文档](https://docs.github.com/en/actions)
- [Cloudflare R2文档](https://developers.cloudflare.com/r2/)
- [语义化版本规范](https://semver.org/)