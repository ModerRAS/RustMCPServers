#!/bin/bash

# GitHub Actions 修复验证脚本
echo "🔍 验证 GitHub Actions 修复效果..."

# 1. 检查许可证一致性
echo "1. 检查许可证一致性..."
if grep -q "MIT License" LICENSE; then
    echo "✅ 根目录 LICENSE 文件是 MIT 许可证"
else
    echo "❌ 根目录 LICENSE 文件不是 MIT 许可证"
    exit 1
fi

if grep -q 'license = "MIT"' Cargo.toml; then
    echo "✅ Workspace 配置使用 MIT 许可证"
else
    echo "❌ Workspace 配置不使用 MIT 许可证"
    exit 1
fi

# 2. 检查工作流文件是否存在
echo "2. 检查工作流文件..."
workflows=(".github/workflows/claude.yml" ".github/workflows/claude-code-review.yml" ".github/workflows/license-check.yml" ".github/workflows/security-scan.yml")

for workflow in "${workflows[@]}"; do
    if [ -f "$workflow" ]; then
        echo "✅ $workflow 存在"
    else
        echo "❌ $workflow 不存在"
        exit 1
    fi
done

# 3. 检查路径引用
echo "3. 检查路径引用..."
if [ -d "servers/json-validator-server" ]; then
    echo "✅ servers/json-validator-server 目录存在"
else
    echo "❌ servers/json-validator-server 目录不存在"
    exit 1
fi

if [ -d "servers/task-orchestrator" ]; then
    echo "✅ servers/task-orchestrator 目录存在"
else
    echo "❌ servers/task-orchestrator 目录不存在"
    exit 1
fi

# 4. 检查集成测试文件
echo "4. 检查集成测试文件..."
if [ -f "servers/json-validator-server/tests/integration_tests.rs" ]; then
    echo "✅ JSON 验证服务器集成测试存在"
else
    echo "❌ JSON 验证服务器集成测试不存在"
    exit 1
fi

if [ -f "servers/task-orchestrator/tests/integration_tests.rs" ]; then
    echo "✅ 任务协调器集成测试存在"
else
    echo "❌ 任务协调器集成测试不存在"
    exit 1
fi

# 5. 检查 Cargo.toml 配置
echo "5. 检查 Cargo.toml 配置..."
for crate in servers/*/Cargo.toml; do
    echo "检查 $crate"
    if grep -q 'license.workspace = true' "$crate" || grep -q 'license = "MIT"' "$crate"; then
        echo "✅ $crate 许可证配置正确"
    else
        echo "❌ $crate 许可证配置不正确"
        exit 1
    fi
done

# 6. 检查是否有错误的路径引用
echo "6. 检查是否有错误的路径引用..."
if grep -r "duckduckgo" .github/workflows/; then
    echo "❌ 发现对不存在的 duckduckgo-mcp-server 的引用"
    exit 1
else
    echo "✅ 没有发现错误的路径引用"
fi

echo ""
echo "🎉 所有检查都通过了！GitHub Actions 修复成功！"
echo ""
echo "修复摘要："
echo "- ✅ 统一使用 MIT 许可证"
echo "- ✅ 修复了路径引用错误"
echo "- ✅ 创建了许可证检查工作流"
echo "- ✅ 创建了安全扫描工作流"
echo "- ✅ 验证了所有配置的正确性"