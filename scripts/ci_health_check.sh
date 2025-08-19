#!/bin/bash

# CI健康检查脚本
# 用于检查GitHub Actions工作流的状态和健康度

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

# 检查GitHub Actions工作流文件
check_workflow_files() {
    log_info "Checking GitHub Actions workflow files..."
    
    local workflows_dir=".github/workflows"
    local required_workflows=("ci.yml" "release.yml" "test-suite.yml")
    
    for workflow in "${required_workflows[@]}"; do
        if [ ! -f "$workflows_dir/$workflow" ]; then
            log_error "Workflow file '$workflow' not found"
            return 1
        fi
        log_success "Found workflow: $workflow"
    done
    
    return 0
}

# 检查工作流语法
check_workflow_syntax() {
    log_info "Checking workflow syntax..."
    
    # 检查是否有yamllint工具
    if command -v yamllint &> /dev/null; then
        log_info "Using yamllint to validate workflow syntax..."
        
        local workflow_files=$(find .github/workflows -name "*.yml" -o -name "*.yaml")
        
        for file in $workflow_files; do
            if yamllint "$file"; then
                log_success "Workflow syntax valid: $file"
            else
                log_error "Workflow syntax invalid: $file"
                return 1
            fi
        done
    else
        log_warning "yamllint not found, skipping syntax validation"
    fi
    
    return 0
}

# 检查工作流依赖关系
check_workflow_dependencies() {
    log_info "Checking workflow dependencies..."
    
    # 检查test-suite.yml中的依赖关系
    local test_suite_file=".github/workflows/test-suite.yml"
    
    if [ -f "$test_suite_file" ]; then
        # 检查是否有循环依赖
        if grep -q "needs.*unit-tests.*integration-tests.*e2e-tests" "$test_suite_file" && \
           grep -q "needs.*integration-tests.*unit-tests" "$test_suite_file"; then
            log_error "Circular dependency detected in test-suite.yml"
            return 1
        fi
        
        log_success "Workflow dependencies are valid"
    fi
    
    return 0
}

# 检查缓存配置
check_cache_configuration() {
    log_info "Checking cache configuration..."
    
    local workflow_files=$(find .github/workflows -name "*.yml" -o -name "*.yaml")
    
    for file in $workflow_files; do
        if grep -q "actions/cache" "$file"; then
            log_info "Checking cache configuration in $file"
            
            # 检查缓存键是否包含hashFiles
            if grep -A 10 "actions/cache" "$file" | grep -q "hashFiles"; then
                log_success "Cache key properly configured in $file"
            else
                log_warning "Cache key might not be optimal in $file"
            fi
            
            # 检查是否有restore-keys
            if grep -A 15 "actions/cache" "$file" | grep -q "restore-keys"; then
                log_success "Cache restore-keys configured in $file"
            else
                log_warning "Cache restore-keys not configured in $file"
            fi
        fi
    done
    
    return 0
}

# 检查超时配置
check_timeout_configuration() {
    log_info "Checking timeout configuration..."
    
    local workflow_files=$(find .github/workflows -name "*.yml" -o -name "*.yaml")
    
    for file in $workflow_files; do
        if grep -q "timeout-minutes" "$file"; then
            log_info "Timeout configuration found in $file"
            
            # 检查超时是否合理
            local timeout_values=$(grep -o "timeout-minutes: [0-9]*" "$file" | cut -d' ' -f2)
            
            for timeout in $timeout_values; do
                if [ "$timeout" -gt 60 ]; then
                    log_warning "Long timeout detected: $timeout minutes in $file"
                elif [ "$timeout" -lt 5 ]; then
                    log_warning "Short timeout detected: $timeout minutes in $file"
                else
                    log_success "Reasonable timeout: $timeout minutes in $file"
                fi
            done
        else
            log_warning "No timeout configuration found in $file"
        fi
    done
    
    return 0
}

# 检查错误处理
check_error_handling() {
    log_info "Checking error handling configuration..."
    
    local workflow_files=$(find .github/workflows -name "*.yml" -o -name "*.yaml")
    
    for file in $workflow_files; do
        if grep -q "if: always()" "$file"; then
            log_success "Error handling with 'if: always()' found in $file"
        fi
        
        if grep -q "continue-on-error" "$file"; then
            log_info "Continue-on-error configuration found in $file"
        fi
    done
    
    return 0
}

# 生成CI健康报告
generate_ci_health_report() {
    log_info "Generating CI health report..."
    
    local report_file="ci_health_report.md"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    
    cat > "$report_file" << EOF
# CI Health Report

**Generated:** $timestamp  
**Repository:** $(git remote get-url origin 2>/dev/null || echo "unknown")  

## Workflow Files Status

| Workflow File | Status | Notes |
|---------------|--------|-------|
EOF

    # 添加工作流文件状态
    local workflow_files=$(find .github/workflows -name "*.yml" -o -name "*.yaml")
    for file in $workflow_files; do
        local status="✅ OK"
        local notes=""
        
        if [ -f "$file" ]; then
            # 检查语法
            if command -v yamllint &> /dev/null; then
                if ! yamllint "$file" >/dev/null 2>&1; then
                    status="❌ Syntax Error"
                    notes="YAML syntax validation failed"
                fi
            fi
            
            # 检查缓存
            if grep -q "actions/cache" "$file"; then
                if ! grep -A 10 "actions/cache" "$file" | grep -q "restore-keys"; then
                    notes="${notes} Missing restore-keys;"
                fi
            fi
            
            # 检查超时
            if ! grep -q "timeout-minutes" "$file"; then
                notes="${notes} No timeout configured;"
            fi
        else
            status="❌ Missing"
            notes="File not found"
        fi
        
        echo "| $file | $status | $notes |" >> "$report_file"
    done
    
    cat >> "$report_file" << EOF

## Cache Configuration

| Workflow | Cache Status | Restore Keys |
|----------|--------------|---------------|
EOF

    for file in $workflow_files; do
        if [ -f "$file" ] && grep -q "actions/cache" "$file"; then
            local cache_status="✅ Configured"
            local restore_keys="❌ Missing"
            
            if grep -A 15 "actions/cache" "$file" | grep -q "restore-keys"; then
                restore_keys="✅ Configured"
            fi
            
            echo "| $file | $cache_status | $restore_keys |" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

## Timeout Configuration

| Workflow | Timeout Status | Duration |
|----------|----------------|----------|
EOF

    for file in $workflow_files; do
        if [ -f "$file" ] && grep -q "timeout-minutes" "$file"; then
            local timeout_values=$(grep -o "timeout-minutes: [0-9]*" "$file" | cut -d' ' -f2)
            local timeout_status="✅ Configured"
            
            for timeout in $timeout_values; do
                echo "| $file | $timeout_status | $timeout minutes |" >> "$report_file"
            done
        else
            echo "| $file | ❌ Not configured | N/A |" >> "$report_file"
        fi
    done
    
    cat >> "$report_file" << EOF

## Recommendations

1. **Cache Optimization**: Ensure all workflows use appropriate cache keys and restore-keys
2. **Timeout Management**: Set reasonable timeouts for all build and test steps
3. **Error Handling**: Use \`if: always()\` for steps that should run even if previous steps fail
4. **Parallel Execution**: Optimize workflow dependencies to maximize parallel execution
5. **Resource Management**: Monitor and optimize resource usage to avoid timeouts

## Next Steps

- Review and implement the recommendations above
- Regularly monitor CI performance and reliability
- Consider implementing CI/CD metrics collection
- Set up alerts for repeated failures

---

*Report generated by CI health check script*
EOF

    log_success "CI health report generated: $report_file"
    return 0
}

# 主函数
main() {
    log_info "Starting CI health check..."
    
    # 检查工作流文件
    if ! check_workflow_files; then
        exit 1
    fi
    
    # 检查工作流语法
    if ! check_workflow_syntax; then
        exit 1
    fi
    
    # 检查工作流依赖关系
    if ! check_workflow_dependencies; then
        exit 1
    fi
    
    # 检查缓存配置
    if ! check_cache_configuration; then
        exit 1
    fi
    
    # 检查超时配置
    if ! check_timeout_configuration; then
        exit 1
    fi
    
    # 检查错误处理
    if ! check_error_handling; then
        exit 1
    fi
    
    # 生成健康报告
    if ! generate_ci_health_report; then
        exit 1
    fi
    
    log_success "CI health check completed successfully"
    exit 0
}

# 运行主函数
main "$@"