#!/bin/bash

# Simple Task Orchestrator MCP Server - Claude Code 演示脚本
# 这个脚本展示了如何使用Claude Code执行功能

set -e

echo "🚀 Simple Task Orchestrator MCP Server - Claude Code 演示"
echo "============================================================"

# 检查服务器是否运行
if ! pgrep -f "simple-task-orchestrator" > /dev/null; then
    echo "❌ 服务器未运行，请先启动服务器："
    echo "   cargo run --bin simple-task-orchestrator"
    exit 1
fi

# 基础URL
BASE_URL="http://localhost:8080"

echo "📍 服务器地址: $BASE_URL"
echo ""

# 1. 创建Claude Code任务
echo "1️⃣ 创建Claude Code任务..."
TASK_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/tasks" \
    -H "Content-Type: application/json" \
    -d '{
        "work_directory": "/tmp/demo-project",
        "prompt": "创建一个简单的Rust Hello World程序，包括Cargo.toml和src/main.rs",
        "execution_mode": "ClaudeCode",
        "priority": "High",
        "tags": ["demo", "rust", "hello-world"]
    }')

# 提取任务ID
TASK_ID=$(echo $TASK_RESPONSE | jq -r '.data.id')
echo "✅ 任务创建成功: $TASK_ID"
echo ""

# 2. 获取任务到工作状态
echo "2️⃣ 获取任务到工作状态..."
WORKER_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/tasks/next?work_path=/tmp/demo-project&worker_id=claude-worker")
echo "✅ 任务已分配给工作器"
echo ""

# 3. 等待一下
echo "3️⃣ 等待任务准备..."
sleep 2
echo ""

# 4. 执行任务
echo "4️⃣ 执行Claude Code任务..."
EXECUTE_RESPONSE=$(curl -s -X POST "$BASE_URL/api/v1/tasks/$TASK_ID/execute")
echo "✅ 任务执行完成"
echo ""

# 5. 查看执行结果
echo "5️⃣ 查看任务结果..."
RESULT_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/tasks/$TASK_ID")

# 提取结果信息
TASK_STATUS=$(echo $RESULT_RESPONSE | jq -r '.data.status')
RESULT_OUTPUT=$(echo $RESULT_RESPONSE | jq -r '.data.result.output // "无输出"')
RESULT_DURATION=$(echo $RESULT_RESPONSE | jq -r '.data.result.duration_ms // 0')

echo "📊 任务状态: $TASK_STATUS"
echo "⏱️  执行时间: ${RESULT_DURATION}ms"
echo ""
echo "📝 执行结果:"
echo "----------------------------------------"
echo "$RESULT_OUTPUT"
echo "----------------------------------------"
echo ""

# 6. 查看生成的文件
echo "6️⃣ 检查生成的文件..."
if [ -d "/tmp/demo-project" ]; then
    echo "📁 项目目录存在:"
    ls -la /tmp/demo-project/
    echo ""
    
    if [ -f "/tmp/demo-project/Cargo.toml" ]; then
        echo "📄 Cargo.toml 内容:"
        cat /tmp/demo-project/Cargo.toml
        echo ""
    fi
    
    if [ -f "/tmp/demo-project/src/main.rs" ]; then
        echo "📄 src/main.rs 内容:"
        cat /tmp/demo-project/src/main.rs
        echo ""
    fi
else
    echo "❌ 项目目录不存在"
fi

# 7. 清理
echo "7️⃣ 清理演示文件..."
rm -rf /tmp/demo-project
echo "✅ 清理完成"
echo ""

# 8. 显示统计信息
echo "8️⃣ 系统统计信息..."
STATS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/v1/statistics")
TOTAL_TASKS=$(echo $STATS_RESPONSE | jq -r '.data.total_tasks')
COMPLETED_TASKS=$(echo $STATS_RESPONSE | jq -r '.data.completed_tasks')

echo "📈 总任务数: $TOTAL_TASKS"
echo "✅ 已完成任务: $COMPLETED_TASKS"
echo ""

echo "🎉 演示完成！"
echo ""
echo "💡 提示："
echo "   - 查看完整日志: tail -f target/debug/simple-task-orchestrator.log"
echo "   - 健康检查: curl $BASE_URL/health"
echo "   - 查看所有任务: curl $BASE_URL/api/v1/tasks"