# JSON Validator MCP Server

这是一个用于验证 JSON 文件的 MCP (Model Context Protocol) 服务器。

## 功能特性

- 验证 JSON 文件格式
- 支持本地文件验证
- 提供详细的错误信息

## 安装

### 从源码构建
```bash
cargo build --release
```

### 使用 apt 安装（推荐）
```bash
echo "deb [trusted=yes] https://<R2-自定义域名>/ stable main" \
  | sudo tee /etc/apt/sources.list.d/mcp.list
sudo apt update && sudo apt install mcp-json-validator
```

## 使用方法

### 作为 MCP 服务器使用
在 Claude Code 中配置：
```bash
claude mcp add json-validator -- mcp-json-validator
```

### 独立使用
```bash
mcp-json-validator <file.json>
```

## 开发

### 运行测试
```bash
cargo test
```

### 构建 Debian 包
```bash
cargo install cargo-deb
cargo deb
```

## 发布

创建新的发布标签：
```bash
git tag mcp-json-validator-v1.0.0
git push origin mcp-json-validator-v1.0.0
```

## 许可证

本项目采用 MIT 许可证。