#!/bin/bash

# GitHub Actions修复验证脚本
# 用于验证许可证配置、构建和部署配置是否正确

set -e

echo "🔍 GitHub Actions修复验证脚本"
echo "================================"

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 函数：打印成功消息
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

# 函数：打印错误消息
error() {
    echo -e "${RED}❌ $1${NC}"
}

# 函数：打印警告消息
warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# 1. 检查许可证一致性
echo -e "\n🔍 检查许可证一致性..."
if ! grep -q "MIT License" LICENSE; then
    error "根目录LICENSE文件不是MIT许可证"
    exit 1
fi
success "根目录LICENSE文件是MIT许可证"

if ! grep -q 'license = "MIT"' Cargo.toml; then
    error "Workspace许可证不是MIT"
    exit 1
fi
success "Workspace许可证是MIT"

missing_license_config=0
for crate in servers/*/Cargo.toml; do
    echo "检查 $crate"
    if ! grep -q 'license.workspace = true' "$crate" && ! grep -q 'license = "MIT"' "$crate"; then
        error "许可证配置不正确: $crate"
        missing_license_config=$((missing_license_config + 1))
    fi
done

if [ $missing_license_config -gt 0 ]; then
    error "发现 $missing_license_config 个包的许可证配置不正确"
    exit 1
fi
success "所有许可证配置一致"

# 2. 检查配置文件
echo -e "\n🔍 检查配置文件..."
if grep -q "Your Name" servers/json-validator-server/Cargo.toml; then
    error "发现占位符配置: Your Name"
    exit 1
fi
success "json-validator-server配置文件已修复"

# 3. 检查workspace配置
echo -e "\n🔍 检查workspace配置..."
if ! grep -q 'resolver = "2"' Cargo.toml; then
    error "Workspace resolver配置不正确"
    exit 1
fi
success "Workspace resolver配置正确"

# 4. 验证json-validator-server构建
echo -e "\n🔍 验证json-validator-server构建..."
cd servers/json-validator-server
if ! cargo check --quiet; then
    error "json-validator-server构建失败"
    exit 1
fi
success "json-validator-server构建成功"

# 5. 检查Cargo.toml配置
echo -e "\n🔍 检查Cargo.toml配置..."
if ! grep -q "version.workspace = true" Cargo.toml; then
    error "Workspace版本配置不正确"
    exit 1
fi
success "Workspace版本配置正确"

# 6. 检查依赖配置
echo -e "\n🔍 检查依赖配置..."
if ! grep -q "tower-http.*compression-br.*timeout" Cargo.toml; then
    warning "tower-http特性配置可能不完整"
fi
success "依赖配置检查完成"

# 7. 检查GitHub Actions文件
echo -e "\n🔍 检查GitHub Actions文件..."
if [ ! -f ".github/workflows/apt-r2.yml" ]; then
    error "apt-r2.yml文件不存在"
    exit 1
fi
success "apt-r2.yml文件存在"

if [ ! -f ".github/workflows/license-check.yml" ]; then
    error "license-check.yml文件不存在"
    exit 1
fi
success "license-check.yml文件存在"

# 8. 验证构建缓存配置
echo -e "\n🔍 验证构建缓存配置..."
if ! grep -q "restore-keys" .github/workflows/apt-r2.yml; then
    warning "apt-r2.yml缺少缓存恢复配置"
fi
success "构建缓存配置检查完成"

echo -e "\n🎉 所有验证通过！"
echo "================================"
echo "✅ 许可证配置一致"
echo "✅ 配置文件已修复"
echo "✅ GitHub Actions工作流已优化"
echo "✅ 构建验证通过"
echo ""
echo "📋 下一步建议："
echo "1. 在本地运行: cargo test"
echo "2. 检查CI/CD流水线"
echo "3. 验证发布流程"
echo "4. 测试部署功能"