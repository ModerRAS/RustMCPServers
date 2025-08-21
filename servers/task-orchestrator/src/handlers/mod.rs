use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::domain::{TaskId, TaskStatus, TaskPriority};
use crate::services::TaskService;
use crate::domain::{CreateTaskRequest, CompleteTaskRequest, AcquireTaskRequest};
use crate::models::TaskFilter;
use crate::errors::{AppError, AppResult, ApiResponse};
use crate::utils::logging::StructuredLogger;

/// API处理器状态
#[derive(Clone)]
pub struct ApiState {
    pub task_service: Arc<TaskService>,
    pub logger: StructuredLogger,
}

/// 任务创建请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ApiCreateTaskRequest {
    #[validate(length(min = 1, max = 512))]
    pub work_directory: String,
    
    #[validate(length(min = 1, max = 10000))]
    pub prompt: String,
    
    #[validate(custom(function = "validate_priority_string"))]
    pub priority: Option<String>,
    
    #[validate(custom(function = "crate::domain::validate_tags"))]
    pub tags: Option<Vec<String>>,
}

/// 任务创建响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiCreateTaskResponse {
    pub task_id: String,
    pub status: String,
    pub priority: String,
    pub work_directory: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

/// 任务获取请求
#[derive(Debug, Deserialize)]
pub struct ApiGetTaskRequest {
    pub work_path: String,
    pub worker_id: String,
}

/// 任务获取响应
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ApiGetTaskResponse {
    pub task_id: String,
    pub prompt: String,
    pub work_directory: String,
    pub priority: String,
    pub tags: Vec<String>,
}

/// 任务完成请求
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ApiCompleteTaskRequest {
    #[validate(length(max = 10000))]
    pub original_prompt: Option<String>,
    pub result: Option<ApiTaskResult>,
}

/// 任务结果
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTaskResult {
    pub status: String,
    pub output: Option<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub details: serde_json::Value,
    #[serde(default)]
    pub duration: Option<u64>,
}

/// 任务详情响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTaskDetail {
    pub task_id: String,
    pub work_directory: String,
    pub prompt: String,
    pub priority: String,
    pub tags: Vec<String>,
    pub status: String,
    pub worker_id: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub result: Option<ApiTaskResult>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: serde_json::Value,
}

/// 任务列表查询参数
#[derive(Debug, Deserialize)]
pub struct ApiTaskListQuery {
    pub status: Option<String>,
    pub work_directory: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// 任务列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiTaskListResponse {
    pub tasks: Vec<ApiTaskDetail>,
    pub pagination: ApiPagination,
}

/// 分页信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiPagination {
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
    pub has_more: bool,
}

/// 任务取消请求
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiCancelTaskRequest {
    pub reason: Option<String>,
}

/// 任务取消响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiCancelTaskResponse {
    pub task_id: String,
    pub status: String,
    pub cancelled_at: String,
    pub reason: Option<String>,
}

/// 任务重试响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiRetryTaskResponse {
    pub task_id: String,
    pub status: String,
    pub retry_count: u32,
    pub max_retries: u32,
    pub last_retry_at: String,
}

/// 健康检查响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub uptime: String,
    pub components: serde_json::Value,
    pub metrics: serde_json::Value,
}

/// 统计信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsResponse {
    pub overview: serde_json::Value,
    pub status_distribution: serde_json::Value,
    pub priority_distribution: serde_json::Value,
    pub performance_metrics: serde_json::Value,
    pub time_series: Vec<serde_json::Value>,
}


/// 创建任务处理器
pub async fn create_task_handler(
    State(state): State<ApiState>,
    Json(request): Json<ApiCreateTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证请求
    request.validate().map_err(|e| {
        AppError::Validation(crate::errors::ValidationError::invalid_validation(e.to_string()))
    })?;

    // 转换优先级
    let priority = if let Some(p_str) = &request.priority {
        TaskPriority::from_str(p_str).map_err(|_| AppError::Validation(crate::errors::ValidationError::invalid_priority(p_str.clone())))?
    } else {
        TaskPriority::default()
    };

    // 转换标签
    let tags = request.tags
        .unwrap_or_default()
        .into_iter()
        .map(|t| crate::domain::TaskTag::new(t))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::Validation(crate::errors::ValidationError::invalid_tags(e.to_string())))?;

    // 创建任务请求
    let create_request = CreateTaskRequest {
        work_directory: request.work_directory,
        prompt: request.prompt,
        priority: Some(priority),
        tags: Some(tags.into_iter().map(|t| t.to_string()).collect()),
    };

    // 创建任务
    let task = state.task_service.create_task(create_request).await?;

    // 记录日志
    state.logger.log_task_created(
        &task.id.to_string(),
        task.work_directory.as_str(),
        &task.priority.to_string(),
        task.tags.len(),
        &task.status.to_string(),
    );

    // 构建响应
    let response = ApiCreateTaskResponse {
        task_id: task.id.to_string(),
        status: task.status.to_string(),
        priority: task.priority.to_string(),
        work_directory: task.work_directory.to_string(),
        tags: task.tags.iter().map(|t| t.to_string()).collect(),
        created_at: task.created_at.to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 获取下一个任务处理器
pub async fn get_next_task_handler(
    State(state): State<ApiState>,
    Query(params): Query<ApiGetTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证请求
    if params.work_path.is_empty() {
        return Err(AppError::Validation(crate::errors::ValidationError::missing_field("work_path".to_string())));
    }

    if params.worker_id.is_empty() {
        return Err(AppError::Validation(crate::errors::ValidationError::missing_field("worker_id".to_string())));
    }

    let acquire_request = AcquireTaskRequest {
        work_path: params.work_path,
        worker_id: params.worker_id,
    };

    // 获取任务
    let task = state.task_service.acquire_task(acquire_request).await?;

    match task {
        Some(task) => {
            // 记录日志
            state.logger.log_task_acquired(
                &task.id.to_string(),
                &task.worker_id.as_ref().unwrap().to_string(),
                task.work_directory.as_str(),
                &task.priority.to_string(),
                0, // 获取时间
            );

            let response = ApiGetTaskResponse {
                task_id: task.id.to_string(),
                prompt: task.prompt.to_string(),
                work_directory: task.work_directory.to_string(),
                priority: task.priority.to_string(),
                tags: task.tags.iter().map(|t| t.to_string()).collect(),
            };

            Ok(Json(ApiResponse::success(response)))
        }
        None => Ok(Json(ApiResponse::success(ApiGetTaskResponse::default()))),
    }
}

/// 完成任务处理器
pub async fn complete_task_handler(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
    Json(request): Json<ApiCompleteTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 验证请求
    request.validate().map_err(|e| {
        AppError::Validation(crate::errors::ValidationError::invalid_validation(e.to_string()))
    })?;

    let task_id = TaskId::from_str(&task_id)?;

    // 转换任务结果
    let result = request.result.map(|r| {
        let status = match r.status.as_str() {
            "success" => crate::domain::TaskResultStatus::Success,
            "failed" => crate::domain::TaskResultStatus::Failed,
            _ => crate::domain::TaskResultStatus::Success,
        };

        let mut task_result = crate::domain::TaskResult::success(r.output.unwrap_or_default());
        task_result.status = status;
        task_result.error = r.error;
        task_result.details = r.details.as_object()
            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        task_result.duration = r.duration;
        task_result
    });

    let complete_request = CompleteTaskRequest {
        original_prompt: request.original_prompt,
        result,
    };

    // 完成任务
    let task = state.task_service.complete_task(&task_id, complete_request).await?;

    // 记录日志
    if let Some(worker_id) = &task.worker_id {
        let processing_time = task.processing_duration()
            .map(|d| d.num_milliseconds() as u64)
            .unwrap_or(0);

        state.logger.log_task_completed(
            &task.id.to_string(),
            worker_id.as_str(),
            &task.status.to_string(),
            processing_time,
            "success",
            None,
        );
    }

    let response = serde_json::json!({
        "task_id": task.id.to_string(),
        "status": task.status.to_string(),
        "completed_at": task.completed_at.unwrap().to_rfc3339(),
        "worker_id": task.worker_id.as_ref().map(|w| w.to_string()),
    });

    Ok(Json(ApiResponse::success(response)))
}

/// 获取任务详情处理器
pub async fn get_task_handler(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let task_id = TaskId::from_str(&task_id)?;
    let task = state.task_service.get_task(&task_id).await?;

    let result = task.result.as_ref().map(|r| ApiTaskResult {
        status: match r.status {
            crate::domain::TaskResultStatus::Success => "success".to_string(),
            crate::domain::TaskResultStatus::Failed => "failed".to_string(),
        },
        output: r.output.clone(),
        error: r.error.clone(),
        details: serde_json::Value::Object(r.details.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        duration: r.duration,
    });

    let response = ApiTaskDetail {
        task_id: task.id.to_string(),
        work_directory: task.work_directory.to_string(),
        prompt: task.prompt.to_string(),
        priority: task.priority.to_string(),
        tags: task.tags.iter().map(|t| t.to_string()).collect(),
        status: task.status.to_string(),
        worker_id: task.worker_id.map(|w| w.to_string()),
        created_at: task.created_at.to_rfc3339(),
        started_at: task.started_at.map(|t| t.to_rfc3339()),
        completed_at: task.completed_at.map(|t| t.to_rfc3339()),
        result,
        error_message: task.error_message,
        retry_count: task.retry_count,
        max_retries: task.max_retries,
        metadata: serde_json::Value::Object(task.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 获取任务列表处理器
pub async fn list_tasks_handler(
    State(state): State<ApiState>,
    Query(params): Query<ApiTaskListQuery>,
) -> Result<impl IntoResponse, AppError> {
    // 构建过滤器
    let mut filter = TaskFilter::new();

    if let Some(status) = &params.status {
        filter = filter.with_status(TaskStatus::from_str(status)?);
    }

    if let Some(work_directory) = &params.work_directory {
        filter = filter.with_work_directory(work_directory.clone());
    }

    if let Some(priority) = &params.priority {
        filter = filter.with_priority(TaskPriority::from_str(priority)?);
    }

    if let Some(tags) = &params.tags {
        filter = filter.with_tags(tags.split(',').map(|s| s.trim().to_string()).collect());
    }

    if let Some(created_after) = &params.created_after {
        filter = filter.with_created_after(chrono::DateTime::parse_from_rfc3339(created_after)?.with_timezone(&chrono::Utc).into());
    }

    if let Some(created_before) = &params.created_before {
        filter = filter.with_created_before(chrono::DateTime::parse_from_rfc3339(created_before)?.with_timezone(&chrono::Utc).into());
    }

    if let Some(limit) = params.limit {
        filter = filter.with_limit(limit);
    }

    if let Some(offset) = params.offset {
        filter = filter.with_offset(offset);
    }

    if let Some(sort_by) = &params.sort_by {
        filter = filter.with_sort_by(sort_by.clone());
    }

    if let Some(sort_order) = &params.sort_order {
        filter = filter.with_sort_order(sort_order.clone());
    }

    // 获取任务列表
    let (tasks, total) = state.task_service.list_tasks(filter).await?;

    // 转换任务详情
    let task_details = tasks.into_iter().map(|task| {
        let result = task.result.as_ref().map(|r| ApiTaskResult {
            status: match r.status {
                crate::domain::TaskResultStatus::Success => "success".to_string(),
                crate::domain::TaskResultStatus::Failed => "failed".to_string(),
            },
            output: r.output.clone(),
            error: r.error.clone(),
            details: serde_json::Value::Object(r.details.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
            duration: r.duration,
        });

        ApiTaskDetail {
            task_id: task.id.to_string(),
            work_directory: task.work_directory.to_string(),
            prompt: task.prompt.to_string(),
            priority: task.priority.to_string(),
            tags: task.tags.iter().map(|t| t.to_string()).collect(),
            status: task.status.to_string(),
            worker_id: task.worker_id.map(|w| w.to_string()),
            created_at: task.created_at.to_rfc3339(),
            started_at: task.started_at.map(|t| t.to_rfc3339()),
            completed_at: task.completed_at.map(|t| t.to_rfc3339()),
            result,
            error_message: task.error_message,
            retry_count: task.retry_count,
            max_retries: task.max_retries,
            metadata: serde_json::Value::Object(task.metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
        }
    }).collect();

    let limit = params.limit.unwrap_or(100) as u64;
    let offset = params.offset.unwrap_or(0) as u64;

    let response = ApiTaskListResponse {
        tasks: task_details,
        pagination: ApiPagination {
            total,
            limit,
            offset,
            has_more: offset + limit < total,
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 取消任务处理器
pub async fn cancel_task_handler(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
    Json(request): Json<ApiCancelTaskRequest>,
) -> Result<impl IntoResponse, AppError> {
    let task_id = TaskId::from_str(&task_id)?;
    let reason = request.reason.clone();
    let task = state.task_service.cancel_task(&task_id, reason.clone()).await?;

    // 记录日志
    state.logger.log_task_cancelled(
        &task.id.to_string(),
        reason.as_deref(),
        None,
    );

    let response = ApiCancelTaskResponse {
        task_id: task.id.to_string(),
        status: task.status.to_string(),
        cancelled_at: task.completed_at.unwrap().to_rfc3339(),
        reason: task.error_message,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 重试任务处理器
pub async fn retry_task_handler(
    State(state): State<ApiState>,
    Path(task_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let task_id = TaskId::from_str(&task_id)?;
    let task = state.task_service.retry_task(&task_id).await?;

    let response = ApiRetryTaskResponse {
        task_id: task.id.to_string(),
        status: task.status.to_string(),
        retry_count: task.retry_count,
        max_retries: task.max_retries,
        last_retry_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 健康检查处理器
pub async fn health_check_handler(
    State(_state): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    use crate::utils::HealthChecker;
    
    let health_checker = HealthChecker::new();
    let health_status = health_checker.check_health().await;

    let response = HealthCheckResponse {
        status: health_status.status,
        timestamp: health_status.timestamp.to_rfc3339(),
        version: health_status.version,
        uptime: format!("{:?}", std::time::SystemTime::UNIX_EPOCH.elapsed().unwrap()),
        components: serde_json::to_value(health_status.components).unwrap(),
        metrics: serde_json::to_value(health_status.metrics).unwrap(),
    };

    Ok(Json(response))
}

/// 获取统计信息处理器
pub async fn get_statistics_handler(
    State(state): State<ApiState>,
) -> Result<impl IntoResponse, AppError> {
    let stats = state.task_service.get_statistics().await?;

    let response = StatisticsResponse {
        overview: serde_json::json!({
            "total_tasks": stats.total_tasks,
            "completed_tasks": stats.completed_tasks,
            "failed_tasks": stats.failed_tasks,
            "cancelled_tasks": stats.cancelled_tasks,
            "active_tasks": stats.active_tasks,
            "success_rate": stats.success_rate
        }),
        status_distribution: serde_json::json!({
            "waiting": stats.waiting_tasks,
            "working": stats.working_tasks,
            "completed": stats.completed_tasks,
            "failed": stats.failed_tasks,
            "cancelled": stats.cancelled_tasks
        }),
        priority_distribution: serde_json::json!({
            "low": 0, // 需要从数据库获取
            "medium": 0,
            "high": 0
        }),
        performance_metrics: serde_json::json!({
            "avg_processing_time": stats.avg_processing_time,
            "tasks_per_hour": stats.tasks_per_hour
        }),
        time_series: vec![],
    };

    Ok(Json(ApiResponse::success(response)))
}

/// 验证优先级字符串
fn validate_priority_string(priority: &str) -> Result<(), validator::ValidationError> {
    // 尝试解析为TaskPriority
    if priority.is_empty() {
        return Ok(());
    }
    
    TaskPriority::from_str(priority)
        .map(|_| ())
        .map_err(|_| validator::ValidationError::new("invalid_priority"))
}

/// 创建API路由
pub fn create_routes(state: ApiState) -> Router {
    Router::new()
        // 任务管理
        .route("/api/v1/tasks", post(create_task_handler).get(list_tasks_handler))
        .route("/api/v1/tasks/next", get(get_next_task_handler))
        .route("/api/v1/tasks/:task_id", get(get_task_handler))
        .route("/api/v1/tasks/:task_id/complete", post(complete_task_handler))
        .route("/api/v1/tasks/:task_id/cancel", post(cancel_task_handler))
        .route("/api/v1/tasks/:task_id/retry", post(retry_task_handler))
        // 系统管理
        .route("/health", get(health_check_handler))
        .route("/api/v1/statistics", get(get_statistics_handler))
        .with_state(state)
}