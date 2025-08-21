#!/bin/bash

# 运行测试并提取结果统计
echo "Running cargo test..."
timeout 180 cargo test 2>&1 > test_output.txt

if [ $? -eq 0 ]; then
    echo "Test completed successfully"
else
    echo "Test was interrupted or failed"
fi

# 查找测试结果行
echo ""
echo "=== Test Results ==="
grep -E "test result.*passed.*failed" test_output.txt | tail -1

echo ""
echo "=== Test Summary ==="
# 计算通过的测试数
passed=$(grep -o "test result:.*passed.*failed" test_output.txt | tail -1 | grep -o '[0-9]\+ passed' | grep -o '[0-9]\+')
# 计算失败的测试数
failed=$(grep -o "test result:.*passed.*failed" test_output.txt | tail -1 | grep -o '[0-9]\+ failed' | grep -o '[0-9]\+')

if [ -n "$passed" ] && [ -n "$failed" ]; then
    total=$((passed + failed))
    echo "Total tests: $total"
    echo "Passed: $passed"
    echo "Failed: $failed"
    if [ $total -gt 0 ]; then
        success_rate=$(echo "scale=1; $passed * 100 / $total" | bc -l)
        echo "Success rate: $success_rate%"
    fi
else
    echo "Could not extract test results"
fi

# 清理
rm -f test_output.txt