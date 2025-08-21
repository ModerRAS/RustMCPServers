#!/bin/bash

# 进入项目根目录
cd /root/WorkSpace/Rust/RustMCPServers

echo "=== 运行测试套件验证优化效果 ==="
echo "当前目录: $(pwd)"
echo ""

# 首先检查项目结构
echo "=== 检查项目结构 ==="
ls -la
echo ""

# 进入测试目录
echo "=== 进入测试目录 ==="
cd tests
ls -la
echo ""

# 运行测试
echo "=== 运行 cargo test --lib ==="
cargo test --lib --verbose
echo ""

# 运行 clippy 检查
echo "=== 运行 cargo clippy 检查代码质量 ==="
cargo clippy --all-targets --all-features -- -D warnings
echo ""

echo "=== 测试完成 ==="