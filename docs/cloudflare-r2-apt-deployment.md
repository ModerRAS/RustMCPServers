# 使用 Cloudflare R2 + GitHub Actions 自动发布 apt 源

本文档介绍如何使用 Cloudflare R2 和 GitHub Actions 自动发布 Rust MCP 服务器的 Debian 包。

## 架构优势

- **成本更低**：R2 前 10 GB 免费，超出后 0.015 USD/GB·月
- **简单配置**：仅需 10 行 YAML 即可运行
- **自动发布**：通过 Git tag 触发自动构建和发布

## 1. R2 准备工作

### 创建 R2 桶
1. 登录 Cloudflare 后台
2. 进入 R2 服务
3. 创建新桶（如 `my-apt-repo`）

### 获取必要信息
- Account ID（仪表盘右侧）
- 创建 R2 API Token（权限：Object Read & Write，绑定到这个桶）

## 2. GitHub 仓库配置

### 添加 Secrets
在 GitHub 仓库设置中添加以下 Secrets：

| Secret 名 | 内容 |
|---|---|
| R2_ACCOUNT_ID | Cloudflare Account ID |
| R2_BUCKET_NAME | R2 桶名 |
| R2_ACCESS_KEY_ID | R2 API Token Access Key |
| R2_SECRET_ACCESS_KEY | R2 API Token Secret Key |

## 3. GitHub Actions 工作流

创建 `.github/workflows/apt-r2.yml`：

```yaml
name: apt-r2

on:
  push:
    tags:
      - 'mcp-*-v*'   # 只匹配带前缀的 tag

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # 1. 打包 deb
      - run: |
          cd servers/${GITHUB_REF_NAME%%-v*}
          cargo install cargo-deb
          cargo deb

      # 2. 上传
      - uses: unfor19/install-aws-cli-action@v1
        with:
          version: 2.22.35   # 兼容 R2 的稳定版
      - run: |
          aws configure set region auto
          aws s3 cp servers/${GITHUB_REF_NAME%%-v*}/target/debian/*.deb \
             s3://${{ secrets.R2_BUCKET_NAME }}/pool/main/ \
             --endpoint-url https://${{ secrets.R2_ACCOUNT_ID }}.r2.cloudflarestorage.com \
             --checksum-algorithm CRC32
```

## 4. 发布流程

### 创建发布标签
```bash
git tag mcp-foo-v1.3.0 && git push origin mcp-foo-v1.3.0
```

### 用户安装
```bash
echo "deb [trusted=yes] https://<R2-自定义域名>/ stable main" \
  | sudo tee /etc/apt/sources.list.d/mcp.list
sudo apt update && sudo apt install mcp-foo
```

## 5. 成本优化

### 自动清理旧包
在 R2 桶设置中添加生命周期规则：
- 30 天未访问的对象自动删除
- 可根据需要调整时间

### 费用估算
- deb 包通常几 MB
- R2 前 10 GB 免费
- 年成本几乎为 0

## 6. 自定义域名配置（可选）

为 R2 桶配置自定义域名以获得更短的 URL：

1. 在 Cloudflare 中为 R2 桶配置自定义域名
2. 用户安装命令变为：
```bash
echo "deb [trusted=yes] https://apt.yourdomain.com/ stable main" \
  | sudo tee /etc/apt/sources.list.d/mcp.list
```

## 7. 注意事项

- 确保 tag 格式正确：`mcp-{server-name}-v{version}`
- deb 包会自动上传到 `pool/main/` 目录
- AWS CLI 版本需要兼容 R2
- 建议定期清理旧版本包以节省存储空间