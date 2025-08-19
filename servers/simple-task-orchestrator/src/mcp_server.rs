use std::sync::Arc;
use std::future::Future;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::{ServerCapabilities, ServerInfo, CallToolResult, Content, Implementation, ErrorData},
    schemars, tool, tool_handler, tool_router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{TaskId, TaskStatus, TaskPriority, CreateTaskRequest, CompleteTaskRequest, AcquireTaskRequest, TaskResult, ExecutionMode};
use crate::services::{TaskService, TaskExecutionService};
use crate::domain::TaskFilter;

// 任务创建请求参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskParams {
    #[schemars(description = "工作目录")]
    pub work_directory: String,
    #[schemars(description = "任务提示")]
    pub prompt: String,
    #[schemars(description = "任务优先级 (Low, Medium, High)")]
    pub priority: Option<String>,
    #[schemars(description = "执行模式 (Standard, ClaudeCode)")]
    pub execution_mode: Option<String>,
    #[schemars(description = "任务标签")]
    pub tags: Option<Vec<String>>,
}

// 获取任务请求参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetTaskParams {
    #[schemars(description = "任务ID")]
    pub task_id: String,
}

// 获取任务请求参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AcquireTaskParams {
    #[schemars(description = "工作路径")]
    pub work_path: String,
    #[schemars(description = "工作节点ID")]
    pub worker_id: String,
}

// 执行任务请求参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExecuteTaskParams {
    #[schemars(description = "任务ID")]
    pub task_id: String,
}

// 完成任务请求参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CompleteTaskParams {
    #[schemars(description = "任务ID")]
    pub task_id: String,
    #[schemars(description = "执行结果")]
    pub result: TaskResultData,
}

// 任务结果数据
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TaskResultData {
    #[schemars(description = "状态")]
    pub status: String,
    #[schemars(description = "输出内容")]
    pub output: String,
    #[schemars(description = "执行时长（毫秒）")]
    pub duration_ms: u64,
}

// 列出任务请求参数
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListTasksParams {
    #[schemars(description = "任务状态过滤")]
    pub status: Option<String>,
    #[schemars(description = "优先级过滤")]
    pub priority: Option<String>,
    #[schemars(description = "限制数量")]
    pub limit: Option<u32>,
}

/// 任务编排MCP服务器
#[derive(Clone)]
pub struct TaskOrchestratorServer {
    task_service: Arc<TaskService>,
    execution_service: Arc<TaskExecutionService>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TaskOrchestratorServer {
    pub fn new(task_service: Arc<TaskService>, execution_service: Arc<TaskExecutionService>) -> Self {
        Self {
            task_service,
            execution_service,
            tool_router: Self::tool_router(),
        }
    }

    // 创建任务
    #[tool(description = "创建新任务")]
    async fn create_task(&self, Parameters(params): Parameters<CreateTaskParams>) -> Result<CallToolResult, ErrorData> {
        let priority = params.priority
            .map(|p| match p.as_str() {
                "Low" => TaskPriority::Low,
                "Medium" => TaskPriority::Medium,
                "High" => TaskPriority::High,
                _ => TaskPriority::Medium,
            })
            .unwrap_or(TaskPriority::Medium);

        let execution_mode = params.execution_mode
            .map(|m| match m.as_str() {
                "Standard" => ExecutionMode::Standard,
                "ClaudeCode" => ExecutionMode::ClaudeCode,
                _ => ExecutionMode::Standard,
            })
            .unwrap_or(ExecutionMode::Standard);

        let request = CreateTaskRequest {
            work_directory: params.work_directory,
            prompt: params.prompt,
            priority: Some(priority),
            execution_mode: Some(execution_mode),
            tags: params.tags,
        };

        match self.task_service.create_task(request).await {
            Ok(task) => {
                let result = serde_json::to_string_pretty(&task)
                    .unwrap_or_else(|_| "Task created".to_string());
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Failed to create task: {}", e))]))
            }
        }
    }

    // 获取任务
    #[tool(description = "获取任务信息")]
    async fn get_task(&self, Parameters(params): Parameters<GetTaskParams>) -> Result<CallToolResult, ErrorData> {
        let task_id = match Uuid::parse_str(&params.task_id) {
            Ok(uuid) => TaskId::from_uuid(uuid),
            Err(_) => {
                return Ok(CallToolResult::error(vec![Content::text("Invalid task ID format")]))
            }
        };

        match self.task_service.get_task(&task_id).await {
            Ok(task) => {
                let result = serde_json::to_string_pretty(&task)
                    .unwrap_or_else(|_| "Task found".to_string());
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Task not found: {}", e))]))
            }
        }
    }

    // 获取待处理任务
    #[tool(description = "获取待处理任务")]
    async fn acquire_task(&self, Parameters(params): Parameters<AcquireTaskParams>) -> Result<CallToolResult, ErrorData> {
        let request = AcquireTaskRequest {
            work_path: params.work_path,
            worker_id: params.worker_id,
        };

        match self.task_service.acquire_task(request).await {
            Ok(Some(task)) => {
                let result = serde_json::to_string_pretty(&task)
                    .unwrap_or_else(|_| "Task acquired".to_string());
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Ok(None) => {
                Ok(CallToolResult::success(vec![Content::text("No tasks available")]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Failed to acquire task: {}", e))]))
            }
        }
    }

    // 执行任务
    #[tool(description = "执行任务")]
    async fn execute_task(&self, Parameters(params): Parameters<ExecuteTaskParams>) -> Result<CallToolResult, ErrorData> {
        let task_id = match Uuid::parse_str(&params.task_id) {
            Ok(uuid) => TaskId::from_uuid(uuid),
            Err(_) => {
                return Ok(CallToolResult::error(vec![Content::text("Invalid task ID format")]))
            }
        };

        match self.execution_service.execute_task(&task_id).await {
            Ok(result) => {
                let response = serde_json::json!({
                    "task_id": task_id.to_string(),
                    "execution_result": result,
                    "executed_at": chrono::Utc::now()
                });
                let result_text = serde_json::to_string_pretty(&response)
                    .unwrap_or_else(|_| "Task executed".to_string());
                Ok(CallToolResult::success(vec![Content::text(result_text)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Failed to execute task: {}", e))]))
            }
        }
    }

    // 完成任务
    #[tool(description = "完成任务")]
    async fn complete_task(&self, Parameters(params): Parameters<CompleteTaskParams>) -> Result<CallToolResult, ErrorData> {
        let task_id = match Uuid::parse_str(&params.task_id) {
            Ok(uuid) => TaskId::from_uuid(uuid),
            Err(_) => {
                return Ok(CallToolResult::error(vec![Content::text("Invalid task ID format")]))
            }
        };

        let task_result = TaskResult {
            status: params.result.status,
            output: params.result.output.clone(),
            duration_ms: params.result.duration_ms,
            metadata: None,
        };

        let complete_request = CompleteTaskRequest {
            original_prompt: None,
            result: Some(task_result),
        };

        match self.task_service.complete_task(&task_id, complete_request).await {
            Ok(task) => {
                let result = serde_json::to_string_pretty(&task)
                    .unwrap_or_else(|_| "Task completed".to_string());
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Failed to complete task: {}", e))]))
            }
        }
    }

    // 列出任务
    #[tool(description = "列出任务")]
    async fn list_tasks(&self, Parameters(params): Parameters<ListTasksParams>) -> Result<CallToolResult, ErrorData> {
        let mut filter = TaskFilter::new();
        
        if let Some(status_str) = params.status {
            let status = match status_str.as_str() {
                "Waiting" => TaskStatus::Waiting,
                "Working" => TaskStatus::Working,
                "Completed" => TaskStatus::Completed,
                "Failed" => TaskStatus::Failed,
                "Cancelled" => TaskStatus::Cancelled,
                _ => {
                    return Ok(CallToolResult::error(vec![Content::text("Invalid status")]))
                }
            };
            filter = filter.with_status(status);
        }
        
        if let Some(priority_str) = params.priority {
            let priority = match priority_str.as_str() {
                "Low" => TaskPriority::Low,
                "Medium" => TaskPriority::Medium,
                "High" => TaskPriority::High,
                _ => {
                    return Ok(CallToolResult::error(vec![Content::text("Invalid priority")]))
                }
            };
            filter = filter.with_priority(priority);
        }
        
        if let Some(limit) = params.limit {
            filter = filter.with_limit(limit);
        }
        
        match self.task_service.list_tasks(filter).await {
            Ok((tasks, _total)) => {
                let result = serde_json::to_string_pretty(&tasks)
                    .unwrap_or_else(|_| "Tasks listed".to_string());
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Failed to list tasks: {}", e))]))
            }
        }
    }

    // 获取统计信息
    #[tool(description = "获取统计信息")]
    async fn get_statistics(&self) -> Result<CallToolResult, ErrorData> {
        match self.task_service.get_statistics().await {
            Ok(stats) => {
                let result = serde_json::to_string_pretty(&stats)
                    .unwrap_or_else(|_| "Statistics retrieved".to_string());
                Ok(CallToolResult::success(vec![Content::text(result)]))
            }
            Err(e) => {
                Ok(CallToolResult::error(vec![Content::text(format!("Failed to get statistics: {}", e))]))
            }
        }
    }
}

// ServerHandler实现
#[tool_handler]
impl ServerHandler for TaskOrchestratorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("任务编排MCP服务器 - 提供任务创建、执行、监控等功能".into()),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            ..Default::default()
        }
    }
}