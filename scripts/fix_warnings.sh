#!/bin/bash

# 修复未使用的导入和变量警告的脚本

echo "🔧 开始修复代码警告..."

# 1. 修复 json-validator-http 中的警告
echo "修复 json-validator-http 警告..."
cd servers/json-validator-http

# 使用 cargo fix 自动修复可以自动修复的问题
cargo fix --lib --allow-dirty

cd ../..

# 2. 修复 tests 中的警告
echo "修复 tests 中的警告..."

# 使用 sed 修复一些常见的未使用变量问题
find tests/ -name "*.rs" -exec sed -i 's/\bworkspace_root:\s*&Path\b/_workspace_root: \&Path/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bworkspace_root:\s&Path\b/_workspace_root: \&Path/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bworkflow_path:\s&str\b/_workflow_path: \&str/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bbranch:\s&str\b/_branch: \&str/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\boptions:\s&ValidationOptions\b/_options: \&ValidationOptions/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bcontent:\s&str\b/_content: \&str/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bcoverage_file:\s&str\b/_coverage_file: \&str/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bbuild_id:\s&str\b/_build_id: \&str/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bcargo_toml_path:\s&Path\b/_cargo_toml_path: \&Path/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bcargo_lock_path:\s&Path\b/_cargo_lock_path: \&Path/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bworkspace_path:\s&Path\b/_workspace_path: \&Path/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bmember_paths:\s&\[\&Path\]\b/_member_paths: \&\[\&Path\]/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bworkspace_toml_path:\s&Path\b/_workspace_toml_path: \&Path/g' {} \;
find tests/ -name "*.rs" -exec sed -i 's/\bjob_count:\susize\b/_job_count: usize/g' {} \;

# 修复未使用的导入
find tests/ -name "*.rs" -exec sed -i '/^use std::path::Path;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use std::process::Command;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use std::collections::HashMap;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use tempfile::NamedTempFile;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use tempfile::TempDir;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use crate::test_utils;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use crate::workflow_validator::WorkflowValidator;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use serde_yaml::Value;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use DateTime;$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use std::sync::{Arc, Mutex};$/d' {} \;
find tests/ -name "*.rs" -exec sed -i '/^use std::thread;$/d' {} \;

# 修复 mut 变量
find tests/ -name "*.rs" -exec sed -i 's/|mut entries|/|entries|/g' {} \;

echo "✅ 警告修复完成"
echo ""
echo "🚀 运行测试检查修复结果..."

# 运行测试检查结果
timeout 120 cargo test 2>&1 | grep -E "(warning|error|failed|FAILED|panicked)" | head -20

echo ""
echo "📊 测试完成！"