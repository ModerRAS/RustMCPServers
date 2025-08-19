#!/bin/bash

# 构建状态监控脚本
# 用于监控和报告GitHub Actions构建状态

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查命令是否存在
check_command() {
    if ! command -v $1 &> /dev/null; then
        log_error "Command '$1' not found"
        return 1
    fi
    return 0
}

# 检查文件是否存在
check_file() {
    if [ ! -f "$1" ]; then
        log_error "File '$1' not found"
        return 1
    fi
    return 0
}

# 检查目录是否存在
check_directory() {
    if [ ! -d "$1" ]; then
        log_error "Directory '$1' not found"
        return 1
    fi
    return 0
}

# 检查Rust环境
check_rust_environment() {
    log_info "Checking Rust environment..."
    
    if ! check_command "cargo"; then
        log_error "Cargo not found"
        return 1
    fi
    
    local rust_version=$(cargo --version | cut -d' ' -f2)
    log_success "Rust version: $rust_version"
    
    # 检查工具链
    if ! cargo check --quiet 2>/dev/null; then
        log_warning "Cargo check failed, but continuing..."
    fi
    
    return 0
}

# 检查项目结构
check_project_structure() {
    log_info "Checking project structure..."
    
    local required_files=("Cargo.toml" "tests/Cargo.toml")
    local required_dirs=("servers" "tests")
    
    for file in "${required_files[@]}"; do
        if ! check_file "$file"; then
            return 1
        fi
    done
    
    for dir in "${required_dirs[@]}"; do
        if ! check_directory "$dir"; then
            return 1
        fi
    done
    
    log_success "Project structure is valid"
    return 0
}

# 检查依赖一致性
check_dependency_consistency() {
    log_info "Checking dependency consistency..."
    
    # 检查workspace依赖
    if ! cargo check --workspace --quiet 2>/dev/null; then
        log_error "Dependency check failed"
        return 1
    fi
    
    log_success "Dependencies are consistent"
    return 0
}

# 运行测试并监控结果
run_tests_with_monitoring() {
    log_info "Running tests with monitoring..."
    
    local test_start_time=$(date +%s)
    local test_results=()
    
    # 定义要运行的测试
    local test_types=("unit" "integration" "e2e")
    
    for test_type in "${test_types[@]}"; do
        log_info "Running $test_type tests..."
        
        local test_start=$(date +%s)
        
        case $test_type in
            "unit")
                if cargo test --lib --quiet --no-fail-fast; then
                    test_results+=("$test_type:PASS")
                    log_success "Unit tests passed"
                else
                    test_results+=("$test_type:FAIL")
                    log_error "Unit tests failed"
                fi
                ;;
            "integration")
                if cargo test --test integration_tests --quiet --no-fail-fast 2>/dev/null; then
                    test_results+=("$test_type:PASS")
                    log_success "Integration tests passed"
                else
                    test_results+=("$test_type:FAIL")
                    log_warning "Integration tests failed or not found"
                fi
                ;;
            "e2e")
                if cargo test --test e2e_tests --quiet --no-fail-fast 2>/dev/null; then
                    test_results+=("$test_type:PASS")
                    log_success "E2E tests passed"
                else
                    test_results+=("$test_type:FAIL")
                    log_warning "E2E tests failed or not found"
                fi
                ;;
        esac
        
        local test_end=$(date +%s)
        local test_duration=$((test_end - test_start))
        log_info "$test_type tests completed in ${test_duration}s"
    done
    
    local test_end_time=$(date +%s)
    local total_duration=$((test_end_time - test_start_time))
    
    log_info "Total test execution time: ${total_duration}s"
    
    # 输出测试结果摘要
    log_info "Test Results Summary:"
    for result in "${test_results[@]}"; do
        echo "  - $result"
    done
    
    return 0
}

# 生成构建报告
generate_build_report() {
    log_info "Generating build report..."
    
    local report_file="build_report.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    local commit_hash=$(git rev-parse HEAD 2>/dev/null || echo "unknown")
    local branch_name=$(git branch --show-current 2>/dev/null || echo "unknown")
    
    cat > "$report_file" << EOF
# Build Report

**Generated:** $timestamp  
**Commit:** $commit_hash  
**Branch:** $branch_name  

## Environment Information

- **OS:** $(uname -s)
- **Architecture:** $(uname -m)
- **Rust Version:** $(cargo --version 2>/dev/null || echo "not found")

## Build Status

- **Workspace Check:** $(cargo check --workspace --quiet 2>/dev/null && echo "✅ PASS" || echo "❌ FAIL")
- **Unit Tests:** $(cargo test --lib --quiet --no-fail-fast 2>/dev/null && echo "✅ PASS" || echo "❌ FAIL")
- **Integration Tests:** $(cargo test --test integration_tests --quiet --no-fail-fast 2>/dev/null && echo "✅ PASS" || echo "❌ FAIL")
- **E2E Tests:** $(cargo test --test e2e_tests --quiet --no-fail-fast 2>/dev/null && echo "✅ PASS" || echo "❌ FAIL")

## Dependencies

### Workspace Dependencies
\`\`\`toml
$(cat Cargo.toml | grep -A 50 '\[workspace.dependencies\]' | head -n 30)
\`\`\`

### Test Dependencies
\`\`\`toml
$(cat tests/Cargo.toml | grep -A 20 '\[dependencies\]' | head -n 20)
\`\`\`

## Notes

This report was generated automatically by the build monitoring script.
EOF

    log_success "Build report generated: $report_file"
    return 0
}

# 主函数
main() {
    log_info "Starting build monitoring..."
    
    # 检查环境
    if ! check_rust_environment; then
        exit 1
    fi
    
    # 检查项目结构
    if ! check_project_structure; then
        exit 1
    fi
    
    # 检查依赖一致性
    if ! check_dependency_consistency; then
        exit 1
    fi
    
    # 运行测试
    if ! run_tests_with_monitoring; then
        log_error "Test execution failed"
        exit 1
    fi
    
    # 生成报告
    if ! generate_build_report; then
        log_error "Report generation failed"
        exit 1
    fi
    
    log_success "Build monitoring completed successfully"
    exit 0
}

# 运行主函数
main "$@"