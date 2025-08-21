use thiserror::Error;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::domain::{TaskId, TaskStatus, TaskPriority, TaskIdError, TaskTagError, WorkerIdError, WorkDirectoryError, PromptError};

/// 应用错误类型
#[derive(Debug, Error)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Task not found: {0}")]
    TaskNotFound(TaskId),
    
    #[error("Task already acquired by another worker")]
    TaskAlreadyAcquired,
    
    #[error("Concurrency conflict")]
    ConcurrencyConflict,
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Configuration error: {0}")]
    Configuration(#[from] config::ConfigError),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Invalid task ID: {0}")]
    InvalidTaskId(#[from] TaskIdError),

    #[error("Date parsing error: {0}")]
    DateParseError(#[from] chrono::ParseError),

    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    #[error("Task status error: {0}")]
    TaskStatus(#[from] crate::domain::TaskStatusError),

    #[error("Task priority error: {0}")]
    TaskPriority(#[from] crate::domain::TaskPriorityError),

    #[error("Task error: {0}")]
    Task(#[from] crate::domain::TaskError),

    #[error("Task tag error: {0}")]
    TaskTag(#[from] TaskTagError),

    #[error("Worker ID error: {0}")]
    WorkerId(#[from] WorkerIdError),

    #[error("Work directory error: {0}")]
    WorkDirectory(#[from] WorkDirectoryError),

    #[error("Prompt error: {0}")]
    Prompt(#[from] PromptError),

    #[error("Database migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

/// 验证错误
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid work directory: {0}")]
    InvalidWorkDirectory(String),
    
    #[error("Invalid prompt: {0}")]
    InvalidPrompt(String),
    
    #[error("Invalid priority: {0}")]
    InvalidPriority(String),
    
    #[error("Invalid tags: {0}")]
    InvalidTags(String),
    
    #[error("Invalid worker ID: {0}")]
    InvalidWorkerId(String),
    
    #[error("Invalid validation: {0}")]
    InvalidValidation(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Field too long: {0}")]
    FieldTooLong(String),
    
    #[error("Invalid status transition: {from} -> {to}")]
    InvalidStatusTransition {
        from: TaskStatus,
        to: TaskStatus,
    },
    
    #[error("Max retries exceeded: {current}/{max}")]
    MaxRetriesExceeded {
        current: u32,
        max: u32,
    },
}

/// API错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl ApiError {
    pub fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
        }
    }

    pub fn with_details(mut self, details: HashMap<String, serde_json::Value>) -> Self {
        self.details = Some(details);
        self
    }

    pub fn validation(message: String) -> Self {
        Self::new("VALIDATION_ERROR".to_string(), message)
    }

    pub fn not_found(message: String) -> Self {
        Self::new("NOT_FOUND".to_string(), message)
    }

    pub fn conflict(message: String) -> Self {
        Self::new("CONFLICT".to_string(), message)
    }

    pub fn unauthorized(message: String) -> Self {
        Self::new("UNAUTHORIZED".to_string(), message)
    }

    pub fn forbidden(message: String) -> Self {
        Self::new("FORBIDDEN".to_string(), message)
    }

    pub fn internal_error(message: String) -> Self {
        Self::new("INTERNAL_ERROR".to_string(), message)
    }

    pub fn rate_limit_exceeded() -> Self {
        Self::new("RATE_LIMIT_EXCEEDED".to_string(), "Rate limit exceeded".to_string())
    }

    pub fn service_unavailable(message: String) -> Self {
        Self::new("SERVICE_UNAVAILABLE".to_string(), message)
    }
}

/// API响应结构
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, api_error) = match self {
            AppError::Validation(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(err.to_string()),
            ),
            AppError::TaskNotFound(task_id) => (
                StatusCode::NOT_FOUND,
                ApiError::not_found(format!("Task not found: {}", task_id)),
            ),
            AppError::TaskAlreadyAcquired => (
                StatusCode::CONFLICT,
                ApiError::conflict("Task already acquired by another worker".to_string()),
            ),
            AppError::ConcurrencyConflict => (
                StatusCode::CONFLICT,
                ApiError::conflict("Concurrency conflict".to_string()),
            ),
            AppError::Database(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::internal_error(format!("Database error: {}", err)),
            ),
            AppError::Configuration(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::internal_error(format!("Configuration error: {}", err)),
            ),
            AppError::Authentication(err) => (
                StatusCode::UNAUTHORIZED,
                ApiError::unauthorized(err),
            ),
            AppError::Authorization(err) => (
                StatusCode::FORBIDDEN,
                ApiError::forbidden(err),
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ApiError::rate_limit_exceeded(),
            ),
            AppError::ServiceUnavailable(err) => (
                StatusCode::SERVICE_UNAVAILABLE,
                ApiError::service_unavailable(err),
            ),
            AppError::Internal(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::internal_error(err),
            ),
            AppError::InvalidTaskId(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Invalid task ID: {}", err)),
            ),
            AppError::DateParseError(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Date parsing error: {}", err)),
            ),
            AppError::Anyhow(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::internal_error(format!("Internal error: {}", err)),
            ),
            AppError::TaskStatus(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Invalid task status: {}", err)),
            ),
            AppError::TaskPriority(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Invalid task priority: {}", err)),
            ),
            AppError::Task(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Task error: {}", err)),
            ),
            AppError::TaskTag(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Task tag error: {}", err)),
            ),
            AppError::WorkerId(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Worker ID error: {}", err)),
            ),
            AppError::WorkDirectory(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Work directory error: {}", err)),
            ),
            AppError::Prompt(err) => (
                StatusCode::BAD_REQUEST,
                ApiError::validation(format!("Prompt error: {}", err)),
            ),
            AppError::Migration(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::internal_error(format!("Database migration error: {}", err)),
            ),
        };

        let response = ApiResponse::<()>::error(api_error);
        (status, Json(response)).into_response()
    }
}

/// 结果类型别名
pub type AppResult<T> = Result<T, AppError>;

/// 从validator::ValidationErrors转换
impl From<validator::ValidationErrors> for ValidationError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.as_ref().unwrap_or(&"invalid".into()))
                })
            })
            .collect();

        ValidationError::InvalidValidation(messages.join(", "))
    }
}

/// 验证错误详情
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub message: String,
    pub constraints: Option<HashMap<String, String>>,
}

impl ValidationError {
    pub fn invalid_validation(messages: String) -> Self {
        ValidationError::InvalidValidation(messages)
    }

    pub fn invalid_work_directory(path: String) -> Self {
        ValidationError::InvalidWorkDirectory(path)
    }

    pub fn invalid_prompt(prompt: String) -> Self {
        ValidationError::InvalidPrompt(prompt)
    }

    pub fn invalid_priority(priority: String) -> Self {
        ValidationError::InvalidPriority(priority)
    }

    pub fn invalid_tags(tags: String) -> Self {
        ValidationError::InvalidTags(tags)
    }

    pub fn invalid_worker_id(worker_id: String) -> Self {
        ValidationError::InvalidWorkerId(worker_id)
    }

    pub fn missing_field(field: String) -> Self {
        ValidationError::MissingField(field)
    }

    pub fn field_too_long(field: String) -> Self {
        ValidationError::FieldTooLong(field)
    }

    pub fn invalid_status_transition(from: TaskStatus, to: TaskStatus) -> Self {
        ValidationError::InvalidStatusTransition { from, to }
    }

    pub fn max_retries_exceeded(current: u32, max: u32) -> Self {
        ValidationError::MaxRetriesExceeded { current, max }
    }
}

/// HTTP请求错误
#[derive(Debug, Error)]
pub enum HttpError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<HttpError> for AppError {
    fn from(err: HttpError) -> Self {
        match err {
            HttpError::RequestFailed(reqwest_err) => {
                if reqwest_err.is_timeout() {
                    AppError::ServiceUnavailable("Request timeout".to_string())
                } else if reqwest_err.is_connect() {
                    AppError::ServiceUnavailable("Connection failed".to_string())
                } else {
                    AppError::Internal(format!("HTTP request failed: {}", reqwest_err))
                }
            }
            HttpError::InvalidResponse(msg) => {
                AppError::Internal(format!("Invalid response: {}", msg))
            }
            HttpError::Timeout(msg) => {
                AppError::ServiceUnavailable(format!("Timeout: {}", msg))
            }
        }
    }
}

/// 任务队列错误
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("Queue is full")]
    QueueFull,
    
    #[error("Queue is empty")]
    QueueEmpty,
    
    #[error("Task not found in queue")]
    TaskNotFound,
    
    #[error("Invalid task state")]
    InvalidTaskState,
}

impl From<QueueError> for AppError {
    fn from(err: QueueError) -> Self {
        match err {
            QueueError::QueueFull => {
                AppError::ServiceUnavailable("Task queue is full".to_string())
            }
            QueueError::QueueEmpty => {
                AppError::TaskNotFound(TaskId::default())
            }
            QueueError::TaskNotFound => {
                AppError::TaskNotFound(TaskId::default())
            }
            QueueError::InvalidTaskState => {
                AppError::Internal("Invalid task state".to_string())
            }
        }
    }
}

/// 并发控制错误
#[derive(Debug, Error)]
pub enum ConcurrencyError {
    #[error("Lock acquisition failed")]
    LockAcquisitionFailed,
    
    #[error("Lock not found")]
    LockNotFound,
    
    #[error("Lock expired")]
    LockExpired,
    
    #[error("Lock already held by another worker")]
    LockAlreadyHeld,
    
    #[error("Invalid lock owner")]
    InvalidLockOwner,
}

impl From<ConcurrencyError> for AppError {
    fn from(err: ConcurrencyError) -> Self {
        match err {
            ConcurrencyError::LockAcquisitionFailed | ConcurrencyError::LockAlreadyHeld => {
                AppError::TaskAlreadyAcquired
            }
            ConcurrencyError::LockNotFound | ConcurrencyError::LockExpired => {
                AppError::ConcurrencyConflict
            }
            ConcurrencyError::InvalidLockOwner => {
                AppError::Authorization("Invalid lock owner".to_string())
            }
        }
    }
}