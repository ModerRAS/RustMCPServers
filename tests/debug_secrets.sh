#!/bin/bash

echo "🔍 Debugging secret detection..."
cd /root/WorkSpace/Rust/RustMCPServers/tests

# 创建测试文件
cat > test_secrets.yml << 'EOF'
name: Test Secrets
on: [push]
env:
  api_key: 'sk-1234567890abcdef1234567890abcdef'
  secret: 'super_secret_value_1234567890abcdef'
  password: 'mypassword123'
  token: 'github_token_1234567890abcdef1234567890'
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Testing secrets"
EOF

echo "📄 Created test file: test_secrets.yml"
echo "📄 File content:"
cat test_secrets.yml

# 运行cargo test来检查问题
echo ""
echo "🔍 Running test with debug output..."
RUST_BACKTRACE=1 cargo test test_secret_detection --lib -- --nocapture

# 清理
rm -f test_secrets.yml