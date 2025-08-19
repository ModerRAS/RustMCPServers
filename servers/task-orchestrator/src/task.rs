use anyhow::Result;
use rmcp::{RoleServer, Service, RequestId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub dependencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub max_concurrent_tasks: usize,
    pub default_timeout_seconds: u64,
    pub retry_attempts: u32,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub name: String,
    pub description: String,
    pub priority: TaskPriority,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub result: Option<TaskResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task: Task,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListResponse {
    pub tasks: Vec<Task>,
}

pub struct TaskOrchestrator {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
    config: TaskConfig,
}

impl TaskOrchestrator {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            config: TaskConfig {
                max_concurrent_tasks: 5,
                default_timeout_seconds: 3600,
                retry_attempts: 3,
                log_level: "INFO".to_string(),
            },
        }
    }

    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task> {
        let task = Task {
            id: Uuid::new_v4().to_string(),
            name: request.name,
            description: request.description,
            status: TaskStatus::Pending,
            priority: request.priority,
            dependencies: request.dependencies,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());

        Ok(task)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(task_id).cloned())
    }

    pub async fn update_task(&self, task_id: &str, request: UpdateTaskRequest) -> Result<Option<Task>> {
        let mut tasks = self.tasks.write().await;
        
        if let Some(task) = tasks.get_mut(task_id) {
            if let Some(status) = request.status {
                task.status = status.clone();
                if status == TaskStatus::Running {
                    task.started_at = Some(Utc::now());
                } else if status == TaskStatus::Completed || status == TaskStatus::Failed {
                    task.completed_at = Some(Utc::now());
                }
            }
            
            if let Some(priority) = request.priority {
                task.priority = priority;
            }
            
            if let Some(result) = request.result {
                task.result = Some(result);
            }
            
            Ok(Some(task.clone()))
        } else {
            Ok(None)
        }
    }

    pub async fn list_tasks(&self) -> Result<Vec<Task>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.values().cloned().collect())
    }

    pub async fn delete_task(&self, task_id: &str) -> Result<bool> {
        let mut tasks = self.tasks.write().await;
        Ok(tasks.remove(task_id).is_some())
    }

    pub async fn get_config(&self) -> Result<TaskConfig> {
        Ok(self.config.clone())
    }

    pub async fn update_config(&self, config: TaskConfig) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        // 在实际实现中，这里应该有配置验证逻辑
        drop(tasks);
        
        // 更新配置
        // 注意：这里简化了实现，实际应该有更好的配置管理
        Ok(())
    }
}

#[rmcp::tool]
impl TaskOrchestrator {
    #[rmcp::tool(description = "Create a new task")]
    pub async fn create_task(
        &self,
        #[rmcp::arg(description = "Task name")] name: String,
        #[rmcp::arg(description = "Task description")] description: String,
        #[rmcp::arg(description = "Task priority")] priority: TaskPriority,
        #[rmcp::arg(description = "Task dependencies")] dependencies: Vec<String>,
    ) -> Result<TaskResponse> {
        let request = CreateTaskRequest {
            name,
            description,
            priority,
            dependencies,
        };

        let task = self.create_task(request).await?;
        Ok(TaskResponse { task })
    }

    #[rmcp::tool(description = "Get a task by ID")]
    pub async fn get_task(
        &self,
        #[rmcp::arg(description = "Task ID")] task_id: String,
    ) -> Result<Option<TaskResponse>> {
        let task = self.get_task(&task_id).await?;
        Ok(task.map(|task| TaskResponse { task }))
    }

    #[rmcp::tool(description = "Update a task")]
    pub async fn update_task(
        &self,
        #[rmcp::arg(description = "Task ID")] task_id: String,
        #[rmcp::arg(description = "Task status")] status: Option<TaskStatus>,
        #[rmcp::arg(description = "Task priority")] priority: Option<TaskPriority>,
        #[rmcp::arg(description = "Task result")] result: Option<TaskResult>,
    ) -> Result<Option<TaskResponse>> {
        let request = UpdateTaskRequest {
            status,
            priority,
            result,
        };

        let task = self.update_task(&task_id, request).await?;
        Ok(task.map(|task| TaskResponse { task }))
    }

    #[rmcp::tool(description = "List all tasks")]
    pub async fn list_tasks(&self) -> Result<TaskListResponse> {
        let tasks = self.list_tasks().await?;
        Ok(TaskListResponse { tasks })
    }

    #[rmcp::tool(description = "Delete a task")]
    pub async fn delete_task(
        &self,
        #[rmcp::arg(description = "Task ID")] task_id: String,
    ) -> Result<bool> {
        self.delete_task(&task_id).await
    }

    #[rmcp::tool(description = "Get task configuration")]
    pub async fn get_config(&self) -> Result<TaskConfig> {
        self.get_config().await
    }

    #[rmcp::tool(description = "Update task configuration")]
    pub async fn update_config(
        &self,
        #[rmcp::arg(description = "Maximum concurrent tasks")] max_concurrent_tasks: usize,
        #[rmcp::arg(description = "Default timeout in seconds")] default_timeout_seconds: u64,
        #[rmcp::arg(description = "Retry attempts")] retry_attempts: u32,
        #[rmcp::arg(description = "Log level")] log_level: String,
    ) -> Result<()> {
        let config = TaskConfig {
            max_concurrent_tasks,
            default_timeout_seconds,
            retry_attempts,
            log_level,
        };

        self.update_config(config).await
    }
}

#[rmcp::service]
impl Service for TaskOrchestrator {
    async fn handle_request(&self, request: rmcp::Request) -> Result<rmcp::Response> {
        // 这里简化了请求处理，实际应该根据工具名称路由到相应的处理函数
        // 由于我们使用了工具宏，所以这里可以简化处理
        Err(anyhow::anyhow!("Not implemented"))
    }
}