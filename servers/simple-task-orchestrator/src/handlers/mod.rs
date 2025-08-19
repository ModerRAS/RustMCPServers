use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::domain::{
    TaskId, TaskStatus, TaskPriority, ExecutionMode,
    CreateTaskRequest, CompleteTaskRequest, AcquireTaskRequest,
    TaskFilter, ApiResponse, ApiError, Task, TaskStatistics, TaskResult,
};
use crate::services::{TaskService, TaskExecutionService};

/// API状态
#[derive(Clone)]
pub struct ApiState {
    pub task_service: Arc<TaskService>,
    pub execution_service: Arc<TaskExecutionService>,
}

/// 任务查询参数
#[derive(Deserialize)]
pub struct TaskQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub worker_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// 获取下一个任务查询参数
#[derive(Deserialize)]
pub struct NextTaskQuery {
    pub work_path: String,
    pub worker_id: String,
}

/// 创建API路由
pub fn create_routes() -> Router<ApiState> {
    Router::new()
        // 任务管理
        .route("/api/v1/tasks", post(create_task))
        .route("/api/v1/tasks", get(list_tasks))
        .route("/api/v1/tasks/next", get(get_next_task))
        .route("/api/v1/tasks/:id", get(get_task))
        .route("/api/v1/tasks/:id", delete(delete_task))
        .route("/api/v1/tasks/:id/complete", post(complete_task))
        .route("/api/v1/tasks/:id/cancel", post(cancel_task))
        .route("/api/v1/tasks/:id/retry", post(retry_task))
        .route("/api/v1/tasks/:id/execute", post(execute_task))
        .route("/api/v1/execute/directory/:work_directory", post(execute_tasks_in_directory))
        // 系统管理
        .route("/api/v1/statistics", get(get_statistics))
        .route("/health", get(health_check))
}

/// 创建任务
async fn create_task(
    State(state): State<ApiState>,
    Json(request): Json<CreateTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, StatusCode> {
    match state.task_service.create_task(request).await {
        Ok(task) => Ok(Json(ApiResponse::success(task))),
        Err(e) => {
            let error = ApiError::new("VALIDATION_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 获取任务
async fn get_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, StatusCode> {
    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => TaskId::from_uuid(uuid),
        Err(_) => return Ok(Json(ApiResponse::error(ApiError::new(
            "INVALID_ID".to_string(),
            "Invalid task ID format".to_string(),
        )))),
    };
    
    match state.task_service.get_task(&task_id).await {
        Ok(task) => Ok(Json(ApiResponse::success(task))),
        Err(e) => {
            let error = ApiError::new("NOT_FOUND".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 获取下一个任务
async fn get_next_task(
    State(state): State<ApiState>,
    Query(params): Query<NextTaskQuery>,
) -> Result<Json<ApiResponse<Task>>, StatusCode> {
    let request = AcquireTaskRequest {
        work_path: params.work_path,
        worker_id: params.worker_id,
    };
    
    match state.task_service.acquire_task(request).await {
        Ok(Some(task)) => Ok(Json(ApiResponse::success(task))),
        Ok(None) => {
            let response = ApiResponse::<Task> {
                success: true,
                data: None,
                error: None,
                timestamp: chrono::Utc::now(),
            };
            Ok(Json(response))
        },
        Err(e) => {
            let error = ApiError::new("ACQUIRE_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 完成任务
async fn complete_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<CompleteTaskRequest>,
) -> Result<Json<ApiResponse<Task>>, StatusCode> {
    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => TaskId::from_uuid(uuid),
        Err(_) => return Ok(Json(ApiResponse::error(ApiError::new(
            "INVALID_ID".to_string(),
            "Invalid task ID format".to_string(),
        )))),
    };
    
    match state.task_service.complete_task(&task_id, request).await {
        Ok(task) => Ok(Json(ApiResponse::success(task))),
        Err(e) => {
            let error = ApiError::new("COMPLETE_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 取消任务
async fn cancel_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Task>>, StatusCode> {
    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => TaskId::from_uuid(uuid),
        Err(_) => return Ok(Json(ApiResponse::error(ApiError::new(
            "INVALID_ID".to_string(),
            "Invalid task ID format".to_string(),
        )))),
    };
    
    let reason = request.get("reason").and_then(|v| v.as_str()).map(|s| s.to_string());
    
    match state.task_service.cancel_task(&task_id, reason).await {
        Ok(task) => Ok(Json(ApiResponse::success(task))),
        Err(e) => {
            let error = ApiError::new("CANCEL_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 重试任务
async fn retry_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Task>>, StatusCode> {
    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => TaskId::from_uuid(uuid),
        Err(_) => return Ok(Json(ApiResponse::error(ApiError::new(
            "INVALID_ID".to_string(),
            "Invalid task ID format".to_string(),
        )))),
    };
    
    match state.task_service.retry_task(&task_id).await {
        Ok(task) => Ok(Json(ApiResponse::success(task))),
        Err(e) => {
            let error = ApiError::new("RETRY_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 删除任务
async fn delete_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => TaskId::from_uuid(uuid),
        Err(_) => return Ok(Json(ApiResponse::error(ApiError::new(
            "INVALID_ID".to_string(),
            "Invalid task ID format".to_string(),
        )))),
    };
    
    // 首先获取任务
    match state.task_service.get_task(&task_id).await {
        Ok(_) => {
            // 如果任务存在，尝试取消它（如果正在运行）
            let _ = state.task_service.cancel_task(&task_id, Some("Task deleted".to_string())).await;
            
            // 返回成功响应
            Ok(Json(ApiResponse::success(())))
        }
        Err(e) => {
            let error = ApiError::new("NOT_FOUND".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 列出任务
async fn list_tasks(
    State(state): State<ApiState>,
    Query(params): Query<TaskQuery>,
) -> Result<Json<ApiResponse<Vec<Task>>>, StatusCode> {
    let mut filter = TaskFilter::new();
    
    if let Some(status_str) = &params.status {
        let status = match status_str.to_lowercase().as_str() {
            "waiting" => TaskStatus::Waiting,
            "working" => TaskStatus::Working,
            "completed" => TaskStatus::Completed,
            "failed" => TaskStatus::Failed,
            "cancelled" => TaskStatus::Cancelled,
            _ => return Ok(Json(ApiResponse::error(ApiError::new(
                "INVALID_STATUS".to_string(),
                "Invalid status parameter".to_string(),
            )))),
        };
        filter = filter.with_status(status);
    }
    
    if let Some(priority_str) = &params.priority {
        let priority = match priority_str.to_lowercase().as_str() {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            _ => return Ok(Json(ApiResponse::error(ApiError::new(
                "INVALID_PRIORITY".to_string(),
                "Invalid priority parameter".to_string(),
            )))),
        };
        filter = filter.with_priority(priority);
    }
    
    if let Some(worker_id) = &params.worker_id {
        filter = filter.with_worker_id(worker_id.clone());
    }
    
    if let Some(limit) = params.limit {
        filter = filter.with_limit(limit);
    }
    
    if let Some(offset) = params.offset {
        filter = filter.with_offset(offset);
    }
    
    match state.task_service.list_tasks(filter).await {
        Ok((tasks, _total)) => {
            // 在实际应用中，这里应该返回分页信息
            Ok(Json(ApiResponse::success(tasks)))
        }
        Err(e) => {
            let error = ApiError::new("LIST_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 获取统计信息
async fn get_statistics(
    State(state): State<ApiState>,
) -> Result<Json<ApiResponse<TaskStatistics>>, StatusCode> {
    match state.task_service.get_statistics().await {
        Ok(stats) => Ok(Json(ApiResponse::success(stats))),
        Err(e) => {
            let error = ApiError::new("STATS_ERROR".to_string(), e);
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 健康检查
async fn health_check() -> Json<ApiResponse<serde_json::Value>> {
    let health_data = serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now(),
        "version": "1.0.0",
        "uptime": "0s", // 简化实现
        "components": {
            "memory": {"healthy": true},
            "storage": {"healthy": true}
        }
    });
    
    Json(ApiResponse::success(health_data))
}

/// 执行单个任务
async fn execute_task(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    let task_id = match Uuid::parse_str(&id) {
        Ok(uuid) => TaskId::from_uuid(uuid),
        Err(_) => return Ok(Json(ApiResponse::error(ApiError::new(
            "INVALID_ID".to_string(),
            "Invalid task ID format".to_string(),
        )))),
    };
    
    match state.execution_service.execute_task(&task_id).await {
        Ok(result) => {
            let response_data = serde_json::json!({
                "task_id": task_id.to_string(),
                "execution_result": result,
                "executed_at": chrono::Utc::now()
            });
            Ok(Json(ApiResponse::success(response_data)))
        }
        Err(e) => {
            let error = ApiError::new("EXECUTION_ERROR".to_string(), e.to_string());
            Ok(Json(ApiResponse::error(error)))
        }
    }
}

/// 执行指定目录的所有任务
async fn execute_tasks_in_directory(
    State(state): State<ApiState>,
    Path(work_directory): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    match state.execution_service.execute_tasks_in_directory(&work_directory).await {
        Ok(results) => {
            // 转换Result类型以便序列化
            let serializable_results: Vec<(String, Result<TaskResult, String>)> = results
                .into_iter()
                .map(|(task_id, result)| {
                    (task_id.to_string(), result.map_err(|e| e.to_string()))
                })
                .collect();
            
            let response_data = serde_json::json!({
                "work_directory": work_directory,
                "execution_results": serializable_results,
                "executed_at": chrono::Utc::now()
            });
            Ok(Json(ApiResponse::success(response_data)))
        }
        Err(e) => {
            let error = ApiError::new("BATCH_EXECUTION_ERROR".to_string(), e.to_string());
            Ok(Json(ApiResponse::error(error)))
        }
    }
}