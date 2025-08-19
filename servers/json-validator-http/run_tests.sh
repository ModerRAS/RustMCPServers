#!/bin/bash

# 测试运行脚本
set -e

echo "运行HTTP协议JSON验证MCP服务器测试..."

# 切换到项目目录
cd "$(dirname "$0")"

echo "🧪 运行单元测试..."
cargo test --lib -- --test-threads=1

echo "🧪 运行集成测试..."
cargo test --test integration_tests -- --test-threads=1

echo "🧪 运行性能测试..."
cargo test --test performance_tests -- --test-threads=1

echo "🧪 运行文档测试..."
cargo test --doc

echo "🧪 运行基准测试..."
cargo bench --no-run

echo "✅ 所有测试完成!"

# 生成测试覆盖率报告
if command -v cargo-tarpaulin &> /dev/null; then
    echo "📊 生成测试覆盖率报告..."
    cargo tarpaulin --out Html --output-dir coverage/
    echo "📊 测试覆盖率报告已生成到 coverage/ 目录"
fi

echo "🎉 测试运行完成!"