#!/bin/bash

# GitHub Actions 测试自动化脚本
# 这个脚本会运行完整的测试套件来验证GitHub Actions工作流

set -e

echo "🚀 Starting GitHub Actions Test Suite"
echo "=================================="

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

# 获取脚本所在目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# 设置工作目录
cd "$PROJECT_ROOT"

log_info "Working directory: $(pwd)"

# 检查依赖
check_dependencies() {
    log_info "Checking dependencies..."
    
    # 检查Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed"
        exit 1
    fi
    
    # 检查Git
    if ! command -v git &> /dev/null; then
        log_error "Git is not installed"
        exit 1
    fi
    
    log_success "All dependencies are available"
}

# 运行单元测试
run_unit_tests() {
    log_info "Running unit tests..."
    
    cd "$PROJECT_ROOT/tests"
    
    if cargo test --lib -- --nocapture; then
        log_success "Unit tests passed"
        return 0
    else
        log_error "Unit tests failed"
        return 1
    fi
}

# 运行集成测试
run_integration_tests() {
    log_info "Running integration tests..."
    
    cd "$PROJECT_ROOT/tests"
    
    if cargo test --test integration_tests -- --nocapture; then
        log_success "Integration tests passed"
        return 0
    else
        log_error "Integration tests failed"
        return 1
    fi
}

# 运行端到端测试
run_e2e_tests() {
    log_info "Running end-to-end tests..."
    
    cd "$PROJECT_ROOT/tests"
    
    if cargo test --test e2e_tests -- --nocapture; then
        log_success "E2E tests passed"
        return 0
    else
        log_error "E2E tests failed"
        return 1
    fi
}

# 运行性能测试
run_performance_tests() {
    log_info "Running performance tests..."
    
    cd "$PROJECT_ROOT/tests"
    
    # 运行基准测试
    if cargo bench; then
        log_success "Performance benchmarks completed"
        return 0
    else
        log_error "Performance benchmarks failed"
        return 1
    fi
}

# 验证GitHub Actions工作流
validate_workflows() {
    log_info "Validating GitHub Actions workflows..."
    
    local workflow_dir="$PROJECT_ROOT/.github/workflows"
    local failed=0
    
    # 检查工作流目录是否存在
    if [[ ! -d "$workflow_dir" ]]; then
        log_error "Workflows directory not found: $workflow_dir"
        return 1
    fi
    
    # 验证每个工作流文件
    for workflow_file in "$workflow_dir"/*.yml; do
        if [[ -f "$workflow_file" ]]; then
            log_info "Validating workflow: $(basename "$workflow_file")"
            
            # 使用我们的验证器进行验证
            cd "$PROJECT_ROOT/tests"
            
            if cargo run --bin validate_workflow -- "$workflow_file"; then
                log_success "✓ $(basename "$workflow_file") is valid"
            else
                log_error "✗ $(basename "$workflow_file") is invalid"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        log_success "All workflows are valid"
        return 0
    else
        log_error "$failed workflows are invalid"
        return 1
    fi
}

# 运行安全测试
run_security_tests() {
    log_info "Running security tests..."
    
    local workflow_dir="$PROJECT_ROOT/.github/workflows"
    local failed=0
    
    for workflow_file in "$workflow_dir"/*.yml; do
        if [[ -f "$workflow_file" ]]; then
            log_info "Testing security of: $(basename "$workflow_file")"
            
            cd "$PROJECT_ROOT/tests"
            
            if cargo run --bin security_test -- "$workflow_file"; then
                log_success "✓ $(basename "$workflow_file") passed security tests"
            else
                log_error "✗ $(basename "$workflow_file") failed security tests"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        log_success "All workflows passed security tests"
        return 0
    else
        log_error "$failed workflows failed security tests"
        return 1
    fi
}

# 生成测试报告
generate_test_report() {
    log_info "Generating test report..."
    
    local report_dir="$PROJECT_ROOT/tests/reports"
    mkdir -p "$report_dir"
    
    local report_file="$report_dir/test_report_$(date +%Y%m%d_%H%M%S).md"
    
    cat > "$report_file" << EOF
# GitHub Actions Test Report

**Generated:** $(date)
**Repository:** $(basename "$PROJECT_ROOT)

## Test Results Summary

| Test Category | Status | Details |
|---------------|--------|---------|
| Unit Tests | $([ "$UNIT_TESTS_PASSED" = "true" ] && echo "✅ PASSED" || echo "❌ FAILED") | |
| Integration Tests | $([ "$INTEGRATION_TESTS_PASSED" = "true" ] && echo "✅ PASSED" || echo "❌ FAILED") | |
| E2E Tests | $([ "$E2E_TESTS_PASSED" = "true" ] && echo "✅ PASSED" || echo "❌ FAILED") | |
| Performance Tests | $([ "$PERFORMANCE_TESTS_PASSED" = "true" ] && echo "✅ PASSED" || echo "❌ FAILED") | |
| Workflow Validation | $([ "$WORKFLOW_VALIDATION_PASSED" = "true" ] && echo "✅ PASSED" || echo "❌ FAILED") | |
| Security Tests | $([ "$SECURITY_TESTS_PASSED" = "true" ] && echo "✅ PASSED" || echo "❌ FAILED") | |

## Overall Status

$([ "$OVERALL_SUCCESS" = "true" ] && echo "🎉 **ALL TESTS PASSED**" || echo "❌ **SOME TESTS FAILED**")

## Performance Metrics

- Total execution time: ${TOTAL_DURATION:-N/A}
- Memory usage: ${MEMORY_USAGE:-N/A}
- Test coverage: ${TEST_COVERAGE:-N/A}

## Recommendations

$(generate_recommendations)

## Next Steps

$(generate_next_steps)

EOF
    
    log_success "Test report generated: $report_file"
    echo "📊 Report: $report_file"
}

# 生成建议
generate_recommendations() {
    local recommendations=""
    
    if [[ "$UNIT_TESTS_PASSED" != "true" ]]; then
        recommendations+="- Fix failing unit tests\n"
    fi
    
    if [[ "$INTEGRATION_TESTS_PASSED" != "true" ]]; then
        recommendations+="- Fix failing integration tests\n"
    fi
    
    if [[ "$SECURITY_TESTS_PASSED" != "true" ]]; then
        recommendations+="- Address security vulnerabilities\n"
    fi
    
    if [[ -z "$recommendations" ]]; then
        recommendations="- All tests are passing! Consider adding more test coverage for edge cases.\n"
    fi
    
    echo -e "$recommendations"
}

# 生成后续步骤
generate_next_steps() {
    echo "- Review the detailed test results above"
    echo "- Address any failing tests or security issues"
    echo "- Consider running tests in your CI/CD pipeline"
    echo "- Update test cases as needed for new features"
}

# 清理函数
cleanup() {
    log_info "Cleaning up..."
    
    # 清理临时文件
    cd "$PROJECT_ROOT"
    cargo clean 2>/dev/null || true
    
    log_success "Cleanup completed"
}

# 主执行函数
main() {
    local start_time=$(date +%s)
    local overall_success=true
    
    # 设置trap进行清理
    trap cleanup EXIT
    
    log_info "Starting GitHub Actions test suite..."
    
    # 检查依赖
    check_dependencies
    
    # 运行测试
    log_info "Running test suite..."
    
    if run_unit_tests; then
        UNIT_TESTS_PASSED=true
    else
        UNIT_TESTS_PASSED=false
        overall_success=false
    fi
    
    if run_integration_tests; then
        INTEGRATION_TESTS_PASSED=true
    else
        INTEGRATION_TESTS_PASSED=false
        overall_success=false
    fi
    
    if run_e2e_tests; then
        E2E_TESTS_PASSED=true
    else
        E2E_TESTS_PASSED=false
        overall_success=false
    fi
    
    if run_performance_tests; then
        PERFORMANCE_TESTS_PASSED=true
    else
        PERFORMANCE_TESTS_PASSED=false
        overall_success=false
    fi
    
    if validate_workflows; then
        WORKFLOW_VALIDATION_PASSED=true
    else
        WORKFLOW_VALIDATION_PASSED=false
        overall_success=false
    fi
    
    if run_security_tests; then
        SECURITY_TESTS_PASSED=true
    else
        SECURITY_TESTS_PASSED=false
        overall_success=false
    fi
    
    # 计算总执行时间
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    TOTAL_DURATION="${duration}s"
    
    # 导出变量供报告使用
    export UNIT_TESTS_PASSED INTEGRATION_TESTS_PASSED E2E_TESTS_PASSED
    export PERFORMANCE_TESTS_PASSED WORKFLOW_VALIDATION_PASSED SECURITY_TESTS_PASSED
    export TOTAL_DURATION OVERALL_SUCCESS="$overall_success"
    
    # 生成报告
    generate_test_report
    
    # 输出最终结果
    echo ""
    echo "=================================="
    if [[ "$overall_success" == "true" ]]; then
        log_success "🎉 All tests passed successfully!"
        echo "⏱️  Total execution time: $TOTAL_DURATION"
        exit 0
    else
        log_error "❌ Some tests failed!"
        echo "⏱️  Total execution time: $TOTAL_DURATION"
        exit 1
    fi
}

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --help, -h          Show this help message"
            echo "  --unit-only         Run only unit tests"
            echo "  --integration-only  Run only integration tests"
            echo "  --e2e-only          Run only E2E tests"
            echo "  --performance-only  Run only performance tests"
            echo "  --security-only     Run only security tests"
            echo "  --validation-only   Run only workflow validation"
            echo ""
            exit 0
            ;;
        --unit-only)
            run_unit_tests
            exit $?
            ;;
        --integration-only)
            run_integration_tests
            exit $?
            ;;
        --e2e-only)
            run_e2e_tests
            exit $?
            ;;
        --performance-only)
            run_performance_tests
            exit $?
            ;;
        --security-only)
            run_security_tests
            exit $?
            ;;
        --validation-only)
            validate_workflows
            exit $?
            ;;
        *)
            log_error "Unknown option: $1"
            exit 1
            ;;
    esac
    shift
done

# 运行主函数
main "$@"