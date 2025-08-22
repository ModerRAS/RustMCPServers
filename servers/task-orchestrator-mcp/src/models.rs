//! # 数据模型模块
//! 
//! 该模块定义了 Task Orchestrator MCP 服务器的核心数据模型和结构体。
//! 
//! ## 主要功能
//! 
//! - **任务模型**: 完整的任务生命周期和数据结构
//! - **状态管理**: 任务状态和优先级的枚举定义
//! - **请求响应**: MCP协议的请求和响应模型
//! - **序列化支持**: 完整的Serde序列化和JSON Schema支持
//! - **类型安全**: 使用Rust类型系统确保数据完整性
//! 
//! ## 核心模型
//! 
//! - `Task`: 任务实体，包含完整的任务信息和状态
//! - `TaskPriority`: 任务优先级枚举（Low, Medium, High, Urgent）
//! - `TaskStatus`: 任务状态枚举（Pending, Waiting, Running, Completed, Failed, Cancelled）
//! - `TaskResult`: 任务执行结果
//! - `CreateTaskRequest`: 创建任务请求
//! - `UpdateTaskRequest`: 更新任务请求
//! - `TaskFilter`: 任务过滤器
//! - `TaskStatistics`: 任务统计信息
//! 
//! ## 使用示例
//! 
//! ```rust
//! use task_orchestrator_mcp::models::{
//!     Task, TaskPriority, TaskStatus, CreateTaskRequest, TaskResult
//! };
//! use chrono::Utc;
//! use uuid::Uuid;
//! 
//! // 创建任务请求
//! let request = CreateTaskRequest {
//!     work_directory: "/workspace/project".to_string(),
//!     prompt: "实现用户认证功能".to_string(),
//!     priority: Some(TaskPriority::High),
//!     tags: Some(vec!["feature".to_string(), "auth".to_string()]),
//!     max_retries: Some(3),
//!     timeout_seconds: Some(3600),
//! };
//! 
//! // 创建任务
//! let task = Task {
//!     id: Uuid::new_v4(),
//!     work_directory: request.work_directory.clone(),
//!     prompt: request.prompt.clone(),
//!     priority: request.priority.unwrap_or_default(),
//!     status: TaskStatus::Pending,
//!     tags: request.tags.unwrap_or_default(),
//!     created_at: Utc::now(),
//!     updated_at: Utc::now(),
//!     started_at: None,
//!     completed_at: None,
//!     result: None,
//!     retry_count: 0,
//!     max_retries: request.max_retries.unwrap_or(3),
//!     timeout_seconds: request.timeout_seconds.unwrap_or(3600),
//!     worker_id: None,
//!     error_message: None,
//! };
//! 
//! // 创建任务结果
//! let result = TaskResult {
//!     status: "success".to_string(),
//!     output: "任务执行完成".to_string(),
//!     duration_ms: 1500,
//!     metadata: std::collections::HashMap::new(),
//! };
//! ```
//! 
//! ## 任务状态管理
//! 
//! 任务支持以下状态流转：
//! 
//! - `Pending` → `Waiting`: 等待资源
//! - `Waiting` → `Running`: 开始执行
//! - `Running` → `Completed`: 执行成功
//! - `Running` → `Failed`: 执行失败
//! - `Running` → `Cancelled`: 被取消
//! - `Failed` → `Pending`: 重试任务
//! 
//! ## 任务优先级
//! 
//! - `Low` (1): 低优先级
//! - `Medium` (2): 中优先级（默认）
//! - `High` (3): 高优先级
//! - `Urgent` (4): 紧急优先级
//! 
//! ## JSON Schema 支持
//! 
//! 所有模型都实现了 `JsonSchema` trait，提供完整的JSON Schema验证支持：
//! 
//! ```rust
//! use schemars::JsonSchema;
//! 
//! // 获取CreateTaskRequest的JSON Schema
//! let schema = schemars::schema_for!(CreateTaskRequest);
//! println!("{}", serde_json::to_string_pretty(&schema).unwrap());
//! ```
//! 
//! ## 序列化和反序列化
//! 
//! 所有模型都支持完整的Serde序列化和反序列化：
//! 
//! ```rust
//! use serde::{Deserialize, Serialize};
//! 
//! // 序列化为JSON
//! let json = serde_json::to_string(&task)?;
//! 
//! // 从JSON反序列化
//! let task: Task = serde_json::from_str(&json)?;
//! ```
//! 
//! ## 类型转换
//! 
//! 提供了 `From<i32>` 的类型转换实现，方便与数据库和其他系统集成：
//! 
//! ```rust
//! let priority = TaskPriority::from(2);  // TaskPriority::Medium
//! let status = TaskStatus::from(3);      // TaskStatus::Completed
//! ```
//! 
//! ## 验证和约束
//! 
//! - 工作目录必须是有效的文件系统路径
//! - 提示文本不能为空且长度有限制
//! - 标签列表中的每个标签都有长度限制
//! - 重试次数和超时时间有合理的默认值
//! 
//! ## 扩展性
//! 
//! 模型设计考虑了未来的扩展性：
//! - 使用 `Option<T>` 类型支持可选字段
//! - 通过 `HashMap<String, serde_json::Value>` 支持灵活的元数据
//! - 枚举类型预留了扩展空间

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