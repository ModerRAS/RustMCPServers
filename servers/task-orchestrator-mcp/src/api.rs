use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::storage::{InMemoryTaskRepository, TaskRepository};
use crate::models::{CreateTaskRequest, TaskFilter, TaskResult, TaskPriority, TaskStatus};

#[derive(Deserialize)]
pub struct CreateTaskParams {
    pub work_directory: String,
    pub prompt: String,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
    pub max_retries: Option<u32>,
    pub timeout_seconds: Option<u32>,
}

#[derive(Deserialize)]
pub struct CompleteTaskParams {
    pub status: String,
    pub output: String,
    pub duration_ms: Option<u64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Deserialize)]
pub struct ListTasksQuery {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub worker_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

pub fn create_api_routes(task_repository: Arc<InMemoryTaskRepository>) -> Router {
    Router::new()
        .route("/tasks", post(create_task).get(list_tasks))
        .route("/tasks/:id", get(get_task).post(complete_task))
        .route("/tasks/:id/retry", post(retry_task))
        .route("/tasks/:id/acquire", post(acquire_task))
        .route("/statistics", get(get_statistics))
        .with_state(task_repository)
}

async fn create_task(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
    Json(params): Json<CreateTaskParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_priority = match params.priority.as_deref() {
        Some("low") => TaskPriority::Low,
        Some("medium") => TaskPriority::Medium,
        Some("high") => TaskPriority::High,
        Some("urgent") => TaskPriority::Urgent,
        Some(_) | None => TaskPriority::Medium,
    };

    let request = CreateTaskRequest {
        work_directory: params.work_directory,
        prompt: params.prompt,
        priority: Some(task_priority),
        tags: params.tags,
        max_retries: params.max_retries,
        timeout_seconds: params.timeout_seconds,
    };

    match task_repository.create_task(request).await {
        Ok(task) => Ok(Json(serde_json::json!({
            "success": true,
            "task": task
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_task(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match task_repository.get_task(task_id).await {
        Ok(task) => Ok(Json(serde_json::json!({
            "success": true,
            "task": task
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn list_tasks(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
    Query(params): Query<ListTasksQuery>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut filter = TaskFilter {
        status: None,
        priority: None,
        worker_id: params.worker_id,
        limit: params.limit.unwrap_or(50),
        offset: params.offset.unwrap_or(0),
    };

    if let Some(status_str) = params.status {
        filter.status = match status_str.as_str() {
            "pending" => Some(TaskStatus::Pending),
            "waiting" => Some(TaskStatus::Waiting),
            "running" => Some(TaskStatus::Running),
            "completed" => Some(TaskStatus::Completed),
            "failed" => Some(TaskStatus::Failed),
            "cancelled" => Some(TaskStatus::Cancelled),
            _ => None,
        };
    }

    if let Some(priority_str) = params.priority {
        filter.priority = match priority_str.as_str() {
            "low" => Some(TaskPriority::Low),
            "medium" => Some(TaskPriority::Medium),
            "high" => Some(TaskPriority::High),
            "urgent" => Some(TaskPriority::Urgent),
            _ => None,
        };
    }

    match task_repository.list_tasks(filter).await {
        Ok(tasks) => Ok(Json(serde_json::json!({
            "success": true,
            "tasks": tasks,
            "count": tasks.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn complete_task(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
    Path(id): Path<String>,
    Json(params): Json<CompleteTaskParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    let result = TaskResult {
        status: params.status.clone(),
        output: params.output.clone(),
        duration_ms: params.duration_ms.unwrap_or(0),
        metadata: params.metadata.map(|m| serde_json::from_value(m).unwrap_or_default()).unwrap_or_default(),
    };

    let result = match params.status.as_str() {
        "success" => task_repository.complete_task(task_id, result).await,
        "failed" => task_repository.fail_task(task_id, params.output).await,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match result {
        Ok(task) => Ok(Json(serde_json::json!({
            "success": true,
            "task": task
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn acquire_task(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let worker_id = id;
    let work_directory = "/tmp".to_string(); // 简化版本，使用固定目录

    match task_repository.acquire_task(worker_id, work_directory).await {
        Ok(Some(task)) => Ok(Json(serde_json::json!({
            "success": true,
            "task": task
        }))),
        Ok(None) => Ok(Json(serde_json::json!({
            "success": false,
            "message": "No tasks available"
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn retry_task(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let task_id = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    match task_repository.retry_task(task_id).await {
        Ok(task) => Ok(Json(serde_json::json!({
            "success": true,
            "task": task
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_statistics(
    State(task_repository): State<Arc<InMemoryTaskRepository>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match task_repository.get_statistics().await {
        Ok(stats) => Ok(Json(serde_json::json!({
            "success": true,
            "statistics": stats
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}