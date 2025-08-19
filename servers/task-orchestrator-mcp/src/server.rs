use std::sync::Arc;
use rmcp::{
    ServerHandler,
    model::{ServerInfo, ServerCapabilities},
    transport::streamable_http_server::{StreamableHttpService, StreamableHttpServerConfig},
    transport::streamable_http_server::session::local::LocalSessionManager,
};
use serde_json::json;
use uuid::Uuid;

use crate::storage::{InMemoryTaskRepository, TaskRepository, RepositoryError};
use crate::models::{CreateTaskRequest, TaskFilter, TaskResult};

#[derive(Debug, Clone)]
pub struct TaskOrchestratorServer {
    #[allow(dead_code)] // Field is used by MCP tools but compiler doesn't detect it
    task_repository: Arc<InMemoryTaskRepository>,
}

impl TaskOrchestratorServer {
    pub fn new(task_repository: Arc<InMemoryTaskRepository>) -> Self {
        Self { task_repository }
    }

    #[allow(dead_code)] // Reserved for future MCP HTTP service integration
    pub fn create_http_service(&self) -> StreamableHttpService<Self, LocalSessionManager> {
        let config = StreamableHttpServerConfig {
            sse_keep_alive: Some(std::time::Duration::from_secs(30)),
            stateful_mode: true,
        };

        let session_manager = Arc::new(LocalSessionManager::default());
        let server = self.clone();

        StreamableHttpService::new(
            move || Ok(server.clone()),
            session_manager,
            config,
        )
    }

    // Tool implementations - these methods are used by MCP tools
    #[allow(dead_code)] // Used by MCP tools
    pub async fn create_task(
        &self,
        work_directory: String,
        prompt: String,
        priority: Option<String>,
        tags: Option<Vec<String>>,
        max_retries: Option<u32>,
        timeout_seconds: Option<u32>,
    ) -> Result<serde_json::Value, String> {
        let task_priority = match priority.as_deref() {
            Some("low") => crate::models::TaskPriority::Low,
            Some("medium") => crate::models::TaskPriority::Medium,
            Some("high") => crate::models::TaskPriority::High,
            Some("urgent") => crate::models::TaskPriority::Urgent,
            Some(_) | None => crate::models::TaskPriority::Medium,
        };

        let request = CreateTaskRequest {
            work_directory,
            prompt,
            priority: Some(task_priority),
            tags,
            max_retries,
            timeout_seconds,
        };

        match self.task_repository.create_task(request).await {
            Ok(task) => Ok(json!({
                "task_id": task.id,
                "status": format!("{:?}", task.status),
                "message": "Task created successfully"
            })),
            Err(e) => Err(format!("Failed to create task: {e}")),
        }
    }

    #[allow(dead_code)] // Used by MCP tools
    pub async fn get_task(&self, task_id: String) -> Result<serde_json::Value, String> {
        let task_id = Uuid::parse_str(&task_id).map_err(|e| format!("Invalid task ID: {e}"))?;

        match self.task_repository.get_task(task_id).await {
            Ok(task) => Ok(serde_json::to_value(task).unwrap()),
            Err(RepositoryError::TaskNotFound(_)) => Err("Task not found".to_string()),
            Err(e) => Err(format!("Failed to get task: {e}")),
        }
    }

    #[allow(dead_code)] // Used by MCP tools
    pub async fn acquire_task(&self, worker_id: String, work_directory: String) -> Result<serde_json::Value, String> {
        match self.task_repository.acquire_task(worker_id, work_directory).await {
            Ok(Some(task)) => Ok(serde_json::to_value(task).unwrap()),
            Ok(None) => Ok(json!({
                "message": "No tasks available for acquisition",
                "task": null
            })),
            Err(e) => Err(format!("Failed to acquire task: {e}")),
        }
    }

    #[allow(dead_code)] // Used by MCP tools
    pub async fn complete_task(
        &self,
        task_id: String,
        status: String,
        output: String,
        duration_ms: Option<u64>,
        metadata: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let task_id = Uuid::parse_str(&task_id).map_err(|e| format!("Invalid task ID: {e}"))?;

        let result = TaskResult {
            status: status.clone(),
            output: output.clone(),
            duration_ms: duration_ms.unwrap_or(0),
            metadata: metadata.map(|m| serde_json::from_value(m).unwrap_or_default()).unwrap_or_default(),
        };

        match status.as_str() {
            "success" => {
                match self.task_repository.complete_task(task_id, result).await {
                    Ok(task) => Ok(json!({
                        "task_id": task.id,
                        "status": format!("{:?}", task.status),
                        "message": "Task completed successfully"
                    })),
                    Err(e) => Err(format!("Failed to complete task: {e}")),
                }
            }
            "failed" => {
                match self.task_repository.fail_task(task_id, output).await {
                    Ok(task) => Ok(json!({
                        "task_id": task.id,
                        "status": format!("{:?}", task.status),
                        "message": "Task marked as failed"
                    })),
                    Err(e) => Err(format!("Failed to mark task as failed: {e}")),
                }
            }
            _ => Err("Invalid status. Must be 'success' or 'failed'".to_string()),
        }
    }

    #[allow(dead_code)] // Used by MCP tools
    pub async fn list_tasks(
        &self,
        status: Option<String>,
        priority: Option<String>,
        worker_id: Option<String>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<serde_json::Value, String> {
        let mut filter = TaskFilter {
            status: None,
            priority: None,
            worker_id,
            limit: limit.unwrap_or(50),
            offset: offset.unwrap_or(0),
        };

        if let Some(status_str) = status {
            filter.status = match status_str.as_str() {
                "pending" => Some(crate::models::TaskStatus::Pending),
                "waiting" => Some(crate::models::TaskStatus::Waiting),
                "running" => Some(crate::models::TaskStatus::Running),
                "completed" => Some(crate::models::TaskStatus::Completed),
                "failed" => Some(crate::models::TaskStatus::Failed),
                "cancelled" => Some(crate::models::TaskStatus::Cancelled),
                _ => None,
            };
        }

        if let Some(priority_str) = priority {
            filter.priority = match priority_str.as_str() {
                "low" => Some(crate::models::TaskPriority::Low),
                "medium" => Some(crate::models::TaskPriority::Medium),
                "high" => Some(crate::models::TaskPriority::High),
                "urgent" => Some(crate::models::TaskPriority::Urgent),
                _ => None,
            };
        }

        match self.task_repository.list_tasks(filter).await {
            Ok(tasks) => Ok(serde_json::to_value(tasks).unwrap()),
            Err(e) => Err(format!("Failed to list tasks: {e}")),
        }
    }

    #[allow(dead_code)] // Used by MCP tools
    pub async fn get_statistics(&self) -> Result<serde_json::Value, String> {
        match self.task_repository.get_statistics().await {
            Ok(stats) => Ok(serde_json::to_value(stats).unwrap()),
            Err(e) => Err(format!("Failed to get statistics: {e}")),
        }
    }

    #[allow(dead_code)] // Used by MCP tools
    pub async fn retry_task(&self, task_id: String) -> Result<serde_json::Value, String> {
        let task_id = Uuid::parse_str(&task_id).map_err(|e| format!("Invalid task ID: {e}"))?;

        match self.task_repository.retry_task(task_id).await {
            Ok(task) => Ok(json!({
                "task_id": task.id,
                "status": format!("{:?}", task.status),
                "message": "Task retry initiated successfully"
            })),
            Err(e) => Err(format!("Failed to retry task: {e}")),
        }
    }
}

impl ServerHandler for TaskOrchestratorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: rmcp::model::Implementation {
                name: "task-orchestrator-mcp".to_string(),
                version: "0.1.0".to_string(),
            },
            instructions: Some("A task orchestrator MCP server for managing and executing tasks".to_string()),
        }
    }
}