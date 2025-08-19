use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub work_directory: String,
    pub prompt: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub timeout_seconds: u32,
    pub worker_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub enum TaskPriority {
    #[schemars(description = "Low priority task")]
    Low = 1,
    #[schemars(description = "Medium priority task")]
    #[default]
    Medium = 2,
    #[schemars(description = "High priority task")]
    High = 3,
    #[schemars(description = "Urgent priority task")]
    Urgent = 4,
}

impl From<i32> for TaskPriority {
    fn from(value: i32) -> Self {
        match value {
            1 => TaskPriority::Low,
            2 => TaskPriority::Medium,
            3 => TaskPriority::High,
            4 => TaskPriority::Urgent,
            _ => TaskPriority::Medium,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub enum TaskStatus {
    #[schemars(description = "Task is pending execution")]
    #[default]
    Pending = 0,
    #[schemars(description = "Task is waiting for resources")]
    Waiting = 1,
    #[schemars(description = "Task is currently running")]
    Running = 2,
    #[schemars(description = "Task completed successfully")]
    Completed = 3,
    #[schemars(description = "Task failed")]
    Failed = 4,
    #[schemars(description = "Task was cancelled")]
    Cancelled = 5,
}

impl From<i32> for TaskStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => TaskStatus::Pending,
            1 => TaskStatus::Waiting,
            2 => TaskStatus::Running,
            3 => TaskStatus::Completed,
            4 => TaskStatus::Failed,
            5 => TaskStatus::Cancelled,
            _ => TaskStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub status: String,
    pub output: String,
    pub duration_ms: u64,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct CreateTaskRequest {
    #[schemars(description = "The working directory where the task should be executed")]
    pub work_directory: String,
    #[schemars(description = "The prompt or description of the task to execute")]
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Task priority level")]
    pub priority: Option<TaskPriority>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Optional tags for categorizing the task")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Maximum number of retry attempts")]
    pub max_retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(description = "Task timeout in seconds")]
    pub timeout_seconds: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    pub status: Option<TaskStatus>,
    pub worker_id: Option<String>,
    pub result: Option<TaskResult>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub worker_id: Option<String>,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistics {
    pub total_tasks: u64,
    pub pending_tasks: u64,
    pub waiting_tasks: u64,
    pub running_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub cancelled_tasks: u64,
    pub average_completion_time_ms: u64,
    pub success_rate: f64,
}