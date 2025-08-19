# Task Orchestrator API 示例

本文档提供了Task Orchestrator MCP Server的API使用示例。

## 基础设置

所有API请求都需要包含正确的Header：

```bash
BASE_URL="http://localhost:8080"
API_KEY="your-api-key-here"
```

## 1. 创建任务

### 请求示例

```bash
curl -X POST "$BASE_URL/api/v1/tasks" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "work_directory": "/home/user/projects/my-project",
    "prompt": "Analyze the codebase and identify potential security vulnerabilities",
    "priority": "high",
    "tags": ["security", "analysis", "urgent"]
  }'
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "waiting",
    "priority": "high",
    "work_directory": "/home/user/projects/my-project",
    "tags": ["security", "analysis", "urgent"],
    "created_at": "2024-01-01T10:00:00Z"
  },
  "error": null,
  "timestamp": "2024-01-01T10:00:00Z"
}
```

## 2. 获取下一个任务

### 请求示例

```bash
curl -X GET "$BASE_URL/api/v1/tasks/next?work_path=/home/user/projects/my-project&worker_id=worker-001" \
  -H "Authorization: Bearer $API_KEY"
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "prompt": "Analyze the codebase and identify potential security vulnerabilities",
    "work_directory": "/home/user/projects/my-project",
    "priority": "high",
    "tags": ["security", "analysis", "urgent"]
  },
  "error": null,
  "timestamp": "2024-01-01T10:00:05Z"
}
```

### 无任务时的响应

```json
{
  "success": true,
  "data": null,
  "error": null,
  "timestamp": "2024-01-01T10:00:05Z"
}
```

## 3. 完成任务

### 请求示例

```bash
curl -X POST "$BASE_URL/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000/complete" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "original_prompt": "Analyze the codebase and identify potential security vulnerabilities",
    "result": {
      "status": "success",
      "output": "Security analysis completed. Found 3 potential vulnerabilities:\n1. SQL injection in user-auth module\n2. XSS in comment system\n3. CSRF in profile update form",
      "duration": 4500,
      "details": {
        "vulnerabilities_found": 3,
        "files_analyzed": 127,
        "scan_duration": "4.5s"
      }
    }
  }'
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "completed",
    "completed_at": "2024-01-01T10:05:00Z",
    "worker_id": "worker-001"
  },
  "error": null,
  "timestamp": "2024-01-01T10:05:00Z"
}
```

## 4. 获取任务详情

### 请求示例

```bash
curl -X GET "$BASE_URL/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000" \
  -H "Authorization: Bearer $API_KEY"
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "work_directory": "/home/user/projects/my-project",
    "prompt": "Analyze the codebase and identify potential security vulnerabilities",
    "priority": "high",
    "tags": ["security", "analysis", "urgent"],
    "status": "completed",
    "worker_id": "worker-001",
    "created_at": "2024-01-01T10:00:00Z",
    "started_at": "2024-01-01T10:00:05Z",
    "completed_at": "2024-01-01T10:05:00Z",
    "result": {
      "status": "success",
      "output": "Security analysis completed. Found 3 potential vulnerabilities...",
      "duration": 4500,
      "details": {
        "vulnerabilities_found": 3,
        "files_analyzed": 127,
        "scan_duration": "4.5s"
      }
    },
    "retry_count": 0,
    "max_retries": 3,
    "metadata": {
      "complexity": "high",
      "estimated_duration": "5m"
    }
  },
  "error": null,
  "timestamp": "2024-01-01T10:05:00Z"
}
```

## 5. 列出任务

### 请求示例

```bash
curl -X GET "$BASE_URL/api/v1/tasks?status=completed&priority=high&limit=5&offset=0" \
  -H "Authorization: Bearer $API_KEY"
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "task_id": "550e8400-e29b-41d4-a716-446655440000",
        "work_directory": "/home/user/projects/my-project",
        "prompt": "Analyze the codebase and identify potential security vulnerabilities",
        "priority": "high",
        "tags": ["security", "analysis", "urgent"],
        "status": "completed",
        "worker_id": "worker-001",
        "created_at": "2024-01-01T10:00:00Z",
        "completed_at": "2024-01-01T10:05:00Z"
      }
    ],
    "pagination": {
      "total": 1,
      "limit": 5,
      "offset": 0,
      "has_more": false
    }
  },
  "error": null,
  "timestamp": "2024-01-01T10:05:00Z"
}
```

## 6. 取消任务

### 请求示例

```bash
curl -X POST "$BASE_URL/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000/cancel" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "reason": "Project cancelled by client"
  }'
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "cancelled",
    "cancelled_at": "2024-01-01T10:02:00Z",
    "reason": "Project cancelled by client"
  },
  "error": null,
  "timestamp": "2024-01-01T10:02:00Z"
}
```

## 7. 重试任务

### 请求示例

```bash
curl -X POST "$BASE_URL/api/v1/tasks/550e8400-e29b-41d4-a716-446655440000/retry" \
  -H "Authorization: Bearer $API_KEY"
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "task_id": "550e8400-e29b-41d4-a716-446655440000",
    "status": "waiting",
    "retry_count": 1,
    "max_retries": 3,
    "last_retry_at": "2024-01-01T10:03:00Z"
  },
  "error": null,
  "timestamp": "2024-01-01T10:03:00Z"
}
```

## 8. 健康检查

### 请求示例

```bash
curl -X GET "$BASE_URL/health"
```

### 响应示例

```json
{
  "status": "healthy",
  "timestamp": "2024-01-01T10:00:00Z",
  "version": "1.0.0",
  "uptime": "1h30m",
  "components": {
    "database": {
      "healthy": true,
      "response_time": 5.2,
      "last_checked": "2024-01-01T10:00:00Z"
    },
    "cache": {
      "healthy": true,
      "response_time": 1.1,
      "last_checked": "2024-01-01T10:00:00Z"
    },
    "external_services": {
      "healthy": true,
      "response_time": 15.3,
      "last_checked": "2024-01-01T10:00:00Z"
    }
  },
  "metrics": {
    "memory_usage": "78MB",
    "cpu_usage": 12.5,
    "active_connections": 15,
    "queue_size": 0
  }
}
```

## 9. 获取统计信息

### 请求示例

```bash
curl -X GET "$BASE_URL/api/v1/statistics" \
  -H "Authorization: Bearer $API_KEY"
```

### 响应示例

```json
{
  "success": true,
  "data": {
    "overview": {
      "total_tasks": 1250,
      "completed_tasks": 980,
      "failed_tasks": 120,
      "cancelled_tasks": 50,
      "active_tasks": 100,
      "success_rate": 0.85
    },
    "status_distribution": {
      "waiting": 45,
      "working": 55,
      "completed": 980,
      "failed": 120,
      "cancelled": 50
    },
    "priority_distribution": {
      "low": 300,
      "medium": 600,
      "high": 350
    },
    "performance_metrics": {
      "avg_processing_time": 2450.5,
      "tasks_per_hour": 85.3
    },
    "time_series": [
      {
        "timestamp": "2024-01-01T09:00:00Z",
        "completed": 15,
        "failed": 2
      },
      {
        "timestamp": "2024-01-01T10:00:00Z",
        "completed": 18,
        "failed": 1
      }
    ]
  },
  "error": null,
  "timestamp": "2024-01-01T10:00:00Z"
}
```

## 错误处理

### 验证错误

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input data",
    "details": {
      "field": "work_directory",
      "message": "Work directory must be an absolute path"
    }
  },
  "timestamp": "2024-01-01T10:00:00Z"
}
```

### 任务未找到

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "NOT_FOUND",
    "message": "Task not found: 550e8400-e29b-41d4-a716-446655440000"
  },
  "timestamp": "2024-01-01T10:00:00Z"
}
```

### 并发冲突

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "CONFLICT",
    "message": "Task already acquired by another worker"
  },
  "timestamp": "2024-01-01T10:00:00Z"
}
```

## Python 客户端示例

```python
import requests
import json
import time

class TaskOrchestratorClient:
    def __init__(self, base_url, api_key):
        self.base_url = base_url
        self.headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json"
        }
    
    def create_task(self, work_directory, prompt, priority="medium", tags=None):
        """创建新任务"""
        data = {
            "work_directory": work_directory,
            "prompt": prompt,
            "priority": priority,
            "tags": tags or []
        }
        
        response = requests.post(
            f"{self.base_url}/api/v1/tasks",
            headers=self.headers,
            json=data
        )
        
        return response.json()
    
    def get_next_task(self, work_path, worker_id):
        """获取下一个任务"""
        params = {
            "work_path": work_path,
            "worker_id": worker_id
        }
        
        response = requests.get(
            f"{self.base_url}/api/v1/tasks/next",
            headers=self.headers,
            params=params
        )
        
        return response.json()
    
    def complete_task(self, task_id, original_prompt, result):
        """完成任务"""
        data = {
            "original_prompt": original_prompt,
            "result": result
        }
        
        response = requests.post(
            f"{self.base_url}/api/v1/tasks/{task_id}/complete",
            headers=self.headers,
            json=data
        )
        
        return response.json()
    
    def get_task(self, task_id):
        """获取任务详情"""
        response = requests.get(
            f"{self.base_url}/api/v1/tasks/{task_id}",
            headers=self.headers
        )
        
        return response.json()

# 使用示例
if __name__ == "__main__":
    client = TaskOrchestratorClient(
        base_url="http://localhost:8080",
        api_key="your-api-key"
    )
    
    # 创建任务
    task = client.create_task(
        work_directory="/home/user/projects/my-project",
        prompt="Analyze the codebase for security issues",
        priority="high",
        tags=["security", "analysis"]
    )
    
    print("Created task:", task)
    
    # 获取任务
    next_task = client.get_next_task(
        work_path="/home/user/projects/my-project",
        worker_id="worker-001"
    )
    
    if next_task["data"]:
        print("Got task:", next_task)
        
        # 完成任务
        result = client.complete_task(
            task_id=next_task["data"]["task_id"],
            original_prompt=next_task["data"]["prompt"],
            result={
                "status": "success",
                "output": "Analysis completed successfully",
                "duration": 3000
            }
        )
        
        print("Completed task:", result)
    else:
        print("No tasks available")
```

## JavaScript 客户端示例

```javascript
class TaskOrchestratorClient {
    constructor(baseUrl, apiKey) {
        this.baseUrl = baseUrl;
        this.headers = {
            'Authorization': `Bearer ${apiKey}`,
            'Content-Type': 'application/json'
        };
    }

    async createTask(workDirectory, prompt, priority = 'medium', tags = []) {
        const data = {
            work_directory: workDirectory,
            prompt: prompt,
            priority: priority,
            tags: tags
        };

        const response = await fetch(`${this.baseUrl}/api/v1/tasks`, {
            method: 'POST',
            headers: this.headers,
            body: JSON.stringify(data)
        });

        return await response.json();
    }

    async getNextTask(workPath, workerId) {
        const params = new URLSearchParams({
            work_path: workPath,
            worker_id: workerId
        });

        const response = await fetch(`${this.baseUrl}/api/v1/tasks/next?${params}`, {
            method: 'GET',
            headers: this.headers
        });

        return await response.json();
    }

    async completeTask(taskId, originalPrompt, result) {
        const data = {
            original_prompt: originalPrompt,
            result: result
        };

        const response = await fetch(`${this.baseUrl}/api/v1/tasks/${taskId}/complete`, {
            method: 'POST',
            headers: this.headers,
            body: JSON.stringify(data)
        });

        return await response.json();
    }

    async getTask(taskId) {
        const response = await fetch(`${this.baseUrl}/api/v1/tasks/${taskId}`, {
            method: 'GET',
            headers: this.headers
        });

        return await response.json();
    }
}

// 使用示例
async function main() {
    const client = new TaskOrchestratorClient(
        'http://localhost:8080',
        'your-api-key'
    );

    try {
        // 创建任务
        const task = await client.createTask(
            '/home/user/projects/my-project',
            'Analyze the codebase for security issues',
            'high',
            ['security', 'analysis']
        );

        console.log('Created task:', task);

        // 获取任务
        const nextTask = await client.getNextTask(
            '/home/user/projects/my-project',
            'worker-001'
        );

        if (nextTask.data) {
            console.log('Got task:', nextTask);

            // 完成任务
            const result = await client.completeTask(
                nextTask.data.task_id,
                nextTask.data.prompt,
                {
                    status: 'success',
                    output: 'Analysis completed successfully',
                    duration: 3000
                }
            );

            console.log('Completed task:', result);
        } else {
            console.log('No tasks available');
        }
    } catch (error) {
        console.error('Error:', error);
    }
}

main();
```

这些示例展示了Task Orchestrator MCP Server的主要功能和使用方法。您可以根据实际需求调整参数和处理逻辑。