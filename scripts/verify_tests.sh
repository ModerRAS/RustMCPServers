#!/bin/bash

echo "🧪 运行完整的测试套件..."

# 运行测试并捕获结果
timeout 180 cargo test 2>&1 > test_results.txt

# 检查测试结果
if [ $? -eq 0 ]; then
    echo "✅ 所有测试通过！"
    
    # 统计测试结果
    echo ""
    echo "📊 测试统计:"
    grep -E "test result.*passed.*failed" test_results.txt | tail -1
    
    # 计算通过率
    passed=$(grep -o "test result:.*passed.*failed" test_results.txt | tail -1 | grep -o '[0-9]\+ passed' | grep -o '[0-9]\+' || echo "0")
    failed=$(grep -o "test result:.*passed.*failed" test_results.txt | tail -1 | grep -o '[0-9]\+ failed' | grep -o '[0-9]\+' || echo "0")
    
    if [ -n "$passed" ] && [ -n "$failed" ]; then
        total=$((passed + failed))
        if [ $total -gt 0 ]; then
            success_rate=$(echo "scale=1; $passed * 100 / $total" | bc -l)
            echo "📈 通过率: $success_rate%"
        fi
    fi
    
    echo ""
    echo "🎯 项目状态: 高质量项目 - 测试全部通过！"
    
else
    echo "❌ 测试失败，请检查错误"
    echo ""
    echo "🔍 错误详情:"
    grep -E "(fail|error|FAILED|panicked)" test_results.txt | head -10
fi

# 清理
rm -f test_results.txt