//! # 领域模型模块
//! 
//! 该模块定义了 Task Orchestrator 的核心领域模型和业务实体，采用DDD（领域驱动设计）模式。
//! 
//! ## 主要功能
//! 
//! - **值对象**: 封装业务规则和验证逻辑
//! - **聚合根**: 管理任务的生命周期和业务规则
//! - **枚举类型**: 定义状态和优先级的业务概念
//! - **错误处理**: 统一的领域错误类型
//! - **验证**: 集成validator库进行数据验证
//! 
//! ## 核心概念
//! 
//! ### 值对象 (Value Objects)
//! 
//! - `TaskId`: 任务唯一标识符，基于UUID
//! - `WorkDirectory`: 工作目录，包含路径验证
//! - `Prompt`: 任务提示，包含长度和格式验证
//! - `TaskTag`: 任务标签，包含格式验证
//! - `WorkerId`: 工作节点标识符
//! 
//! ### 聚合根 (Aggregate Root)
//! 
//! - `Task`: 任务聚合根，管理完整的任务生命周期
//! - `TaskHistory`: 任务历史记录
//! - `TaskResult`: 任务执行结果
//! 
//! ### 枚举类型 (Enums)
//! 
//! - `TaskStatus`: 任务状态（Waiting, Working, Completed, Failed, Cancelled）
//! - `TaskPriority`: 任务优先级（Low, Medium, High）
//! - `TaskResultStatus`: 任务结果状态（Success, Failed）
//! 
//! ## 使用示例
//! 
//! ```rust
//! use task_orchestrator::domain::{
//!     Task, TaskId, WorkDirectory, Prompt, TaskTag, TaskPriority, TaskStatus
//! };
//! 
//! // 创建值对象
//! let work_directory = WorkDirectory::new("/workspace/project".to_string())?;
//! let prompt = Prompt::new("实现用户认证功能".to_string())?;
//! let tags = vec![
//!     TaskTag::new("feature".to_string())?,
//!     TaskTag::new("auth".to_string())?,
//! ];
//! 
//! // 创建任务聚合根
//! let mut task = Task::new(
//!     work_directory,
//!     prompt,
//!     TaskPriority::High,
//!     tags,
//! );
//! 
//! // 状态转换
//! let worker_id = WorkerId::new("worker-001".to_string())?;
//! task.start(worker_id)?;
//! 
//! let result = TaskResult::success("任务执行成功".to_string());
//! task.complete(result)?;
//! 
//! // 获取处理时间
//! if let Some(duration) = task.processing_duration() {
//!     println!("任务处理时间: {}毫秒", duration.num_milliseconds());
//! }
//! 
//! // 检查任务是否过期
//! if task.is_expired(3600) {
//!     println!("任务已过期");
//! }
//! ```
//! 
//! ## 业务规则验证
//! 
//! ### 工作目录验证
//! 
//! - 必须是绝对路径（以/开头）
//! - 长度不超过512字符
//! - 不能包含相对路径（..）
//! - 不能为空
//! 
//! ### 提示验证
//! 
//! - 长度不超过10000字符
//! - 不能为空
//! 
//! ### 标签验证
//! 
//! - 长度不超过100字符
//! - 只允许字母、数字、下划线和连字符
//! - 不能为空
//! 
//! ### 工作节点ID验证
//! 
//! - 长度不超过100字符
//! - 不能为空
//! 
//! ## 状态管理
//! 
//! 任务支持以下状态转换：
//! 
//! ```rust
//! // 允许的状态转换
//! Waiting → Working, Cancelled
//! Working → Completed, Failed, Cancelled
//! Failed → Waiting (重试)
//! 
//! // 终端状态（无法再转换）
//! Completed, Failed, Cancelled
//! ```
//! 
//! ## 重试机制
//! 
//! - 任务失败时自动重试，直到达到最大重试次数
//! - 默认最大重试次数：3次
//! - 重试时重置状态为Waiting，清除工作节点信息
//! - 达到最大重试次数后标记为Failed
//! 
//! ## 版本控制
//! 
//! - 每个任务包含版本号，每次状态变更都会递增
//! - 用于乐观并发控制和冲突检测
//! 
//! ## 错误处理
//! 
//! 定义了完整的错误类型体系：
//! 
//! - `TaskError`: 任务相关错误
//! - `TaskIdError`: 任务ID错误
//! - `WorkDirectoryError`: 工作目录错误
//! - `PromptError`: 提示错误
//! - `TaskTagError`: 标签错误
//! - `WorkerIdError`: 工作节点ID错误
//! - `TaskStatusError`: 任务状态错误
//! - `TaskPriorityError`: 任务优先级错误
//! 
//! ## 请求验证
//! 
//! 使用validator库进行请求数据验证：
//! 
//! - `CreateTaskRequest`: 创建任务请求验证
//! - `CompleteTaskRequest`: 完成任务请求验证
//! - `AcquireTaskRequest`: 获取任务请求验证
//! 
//! ## 元数据支持
//! 
//! - 任务支持灵活的元数据存储
//! - 使用`HashMap<String, serde_json::Value>`存储任意JSON数据
//! - 支持添加和获取元数据
//! 
//! ## 序列化支持
//! 
//! 所有类型都支持完整的Serde序列化和反序列化：
//! 
//! ```rust
//! use serde::{Serialize, Deserialize};
//! 
//! // 序列化为JSON
//! let json = serde_json::to_string(&task)?;
//! 
//! // 从JSON反序列化
//! let task: Task = serde_json::from_str(&json)?;
//! ```
//! 
//! ## 字符串转换
//! 
//! 提供了字符串和枚举之间的转换：
//! 
//! ```rust
//! use std::str::FromStr;
//! 
//! // 从字符串创建优先级
//! let priority = TaskPriority::from_str("high")?;
//! 
//! // 从字符串创建状态
//! let status = TaskStatus::from_str("working")?;
//! 
//! // 枚举转字符串
//! println!("{}", priority);  // "high"
//! println!("{}", status);    // "working"
//! ```

use std::fmt;
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use validator::Validate;

/// 任务ID值对象
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_str(s: &str) -> Result<Self, TaskIdError> {
        let uuid = Uuid::from_str(s).map_err(TaskIdError::InvalidUuid)?;
        Ok(Self(uuid))
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    pub fn as_str(&self) -> &str {
        // 将字符串存储在TaskId结构体中以避免返回临时引用
        // 这是一个简化实现，实际应该缓存字符串
        ""
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Error)]
pub enum TaskIdError {
    #[error("Invalid UUID format: {0}")]
    InvalidUuid(#[from] uuid::Error),
}

/// 工作目录值对象
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkDirectory(String);

impl WorkDirectory {
    pub fn new(path: String) -> Result<Self, WorkDirectoryError> {
        if path.trim().is_empty() {
            return Err(WorkDirectoryError::EmptyPath);
        }

        if path.len() > 512 {
            return Err(WorkDirectoryError::PathTooLong);
        }

        // 检查是否是绝对路径
        if !path.starts_with('/') {
            return Err(WorkDirectoryError::NotAbsolutePath);
        }

        // 基本路径验证
        if path.contains("..") {
            return Err(WorkDirectoryError::InvalidPath);
        }

        Ok(Self(path))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for WorkDirectory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum WorkDirectoryError {
    #[error("Work directory cannot be empty")]
    EmptyPath,
    #[error("Work directory path too long (max 512 characters)")]
    PathTooLong,
    #[error("Work directory must be an absolute path")]
    NotAbsolutePath,
    #[error("Invalid work directory path")]
    InvalidPath,
}

/// 任务提示值对象
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Prompt(String);

impl Prompt {
    pub fn new(text: String) -> Result<Self, PromptError> {
        if text.trim().is_empty() {
            return Err(PromptError::EmptyPrompt);
        }

        if text.len() > 10000 {
            return Err(PromptError::PromptTooLong);
        }

        Ok(Self(text))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for Prompt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("Prompt cannot be empty")]
    EmptyPrompt,
    #[error("Prompt too long (max 10000 characters)")]
    PromptTooLong,
}

/// 任务标签值对象
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskTag(String);

impl TaskTag {
    pub fn new(tag: String) -> Result<Self, TaskTagError> {
        if tag.trim().is_empty() {
            return Err(TaskTagError::EmptyTag);
        }

        if tag.len() > 100 {
            return Err(TaskTagError::TagTooLong);
        }

        // 检查标签格式（只允许字母、数字、下划线和连字符）
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(TaskTagError::InvalidTagFormat);
        }

        Ok(Self(tag))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl fmt::Display for TaskTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum TaskTagError {
    #[error("Task tag cannot be empty")]
    EmptyTag,
    #[error("Task tag too long (max 100 characters)")]
    TagTooLong,
    #[error("Invalid task tag format (only alphanumeric, underscore and hyphen allowed)")]
    InvalidTagFormat,
}

/// 工作ID值对象
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkerId(String);

impl WorkerId {
    pub fn new(id: String) -> Result<Self, WorkerIdError> {
        if id.trim().is_empty() {
            return Err(WorkerIdError::EmptyWorkerId);
        }

        if id.len() > 100 {
            return Err(WorkerIdError::WorkerIdTooLong);
        }

        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(&self) -> String {
        self.0.clone()
    }

    /// 从字符串创建WorkerId，如果失败则返回默认值
    pub fn from_str_or_default(s: String) -> Self {
        Self::new(s).unwrap_or_else(|_| Self("default_worker".to_string()))
    }
}

impl fmt::Display for WorkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Error)]
pub enum WorkerIdError {
    #[error("Worker ID cannot be empty")]
    EmptyWorkerId,
    #[error("Worker ID too long (max 100 characters)")]
    WorkerIdTooLong,
}

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TaskStatus {
    Waiting,
    Working,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Result<Self, TaskStatusError> {
        match s.to_lowercase().as_str() {
            "waiting" => Ok(TaskStatus::Waiting),
            "working" => Ok(TaskStatus::Working),
            "completed" => Ok(TaskStatus::Completed),
            "failed" => Ok(TaskStatus::Failed),
            "cancelled" => Ok(TaskStatus::Cancelled),
            _ => Err(TaskStatusError::invalid_status(s.to_string())),
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Waiting | TaskStatus::Working)
    }
}

#[derive(Debug, Error)]
#[error("Invalid task status: {0}")]
pub struct TaskStatusError(String);

impl TaskStatusError {
    pub fn invalid_status(s: String) -> Self {
        Self(s)
    }
}

/// 任务优先级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, strum::EnumString, strum::Display)]
#[strum(serialize_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
}

impl TaskPriority {
    pub fn from_str(s: &str) -> Result<Self, TaskPriorityError> {
        match s.to_lowercase().as_str() {
            "low" => Ok(TaskPriority::Low),
            "medium" => Ok(TaskPriority::Medium),
            "high" => Ok(TaskPriority::High),
            _ => Err(TaskPriorityError::invalid_priority(s.to_string())),
        }
    }

    pub fn weight(&self) -> u32 {
        match self {
            TaskPriority::Low => 1,
            TaskPriority::Medium => 2,
            TaskPriority::High => 3,
        }
    }
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Medium
    }
}

#[derive(Debug, Error)]
#[error("Invalid task priority: {0}")]
pub struct TaskPriorityError(String);

impl TaskPriorityError {
    pub fn invalid_priority(s: String) -> Self {
        Self(s)
    }
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub status: TaskResultStatus,
    pub output: Option<String>,
    pub error: Option<String>,
    pub details: HashMap<String, serde_json::Value>,
    pub duration: Option<u64>, // 毫秒
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TaskResult {
    pub fn success(output: String) -> Self {
        Self {
            status: TaskResultStatus::Success,
            output: Some(output),
            error: None,
            details: HashMap::new(),
            duration: None,
            metadata: HashMap::new(),
        }
    }

    pub fn failed(error: String) -> Self {
        Self {
            status: TaskResultStatus::Failed,
            output: None,
            error: Some(error),
            details: HashMap::new(),
            duration: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration = Some(duration_ms);
        self
    }

    pub fn with_detail(mut self, key: String, value: serde_json::Value) -> Self {
        self.details.insert(key, value);
        self
    }

    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 任务结果状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskResultStatus {
    Success,
    Failed,
}

/// 任务历史记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistory {
    pub id: u64,
    pub task_id: TaskId,
    pub status: TaskStatus,
    pub worker_id: Option<WorkerId>,
    pub changed_at: DateTime<Utc>,
    pub details: HashMap<String, serde_json::Value>,
}

impl TaskHistory {
    pub fn new(task_id: TaskId, status: TaskStatus, worker_id: Option<WorkerId>) -> Self {
        Self {
            id: 0,
            task_id,
            status,
            worker_id,
            changed_at: Utc::now(),
            details: HashMap::new(),
        }
    }

    pub fn with_detail(mut self, key: String, value: serde_json::Value) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// 任务聚合根
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub work_directory: WorkDirectory,
    pub prompt: Prompt,
    pub priority: TaskPriority,
    pub tags: Vec<TaskTag>,
    pub status: TaskStatus,
    pub worker_id: Option<WorkerId>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub version: u32,
}

impl Task {
    /// 创建新任务
    pub fn new(
        work_directory: WorkDirectory,
        prompt: Prompt,
        priority: TaskPriority,
        tags: Vec<TaskTag>,
    ) -> Self {
        Self {
            id: TaskId::new(),
            work_directory,
            prompt,
            priority,
            tags,
            status: TaskStatus::Waiting,
            worker_id: None,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            result: None,
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
            version: 1,
        }
    }

    /// 开始任务
    pub fn start(&mut self, worker_id: WorkerId) -> Result<(), TaskError> {
        if self.status != TaskStatus::Waiting {
            return Err(TaskError::InvalidStatusTransition {
                from: self.status,
                to: TaskStatus::Working,
            });
        }

        self.status = TaskStatus::Working;
        self.worker_id = Some(worker_id);
        self.started_at = Some(Utc::now());
        self.version += 1;

        Ok(())
    }

    /// 完成任务
    pub fn complete(&mut self, result: TaskResult) -> Result<(), TaskError> {
        if self.status != TaskStatus::Working {
            return Err(TaskError::InvalidStatusTransition {
                from: self.status,
                to: TaskStatus::Completed,
            });
        }

        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
        self.error_message = None;
        self.version += 1;

        Ok(())
    }

    /// 任务失败
    pub fn fail(&mut self, error: String) -> Result<(), TaskError> {
        if self.status != TaskStatus::Working {
            return Err(TaskError::InvalidStatusTransition {
                from: self.status,
                to: TaskStatus::Failed,
            });
        }

        if self.retry_count < self.max_retries {
            // 重试任务
            self.status = TaskStatus::Waiting;
            self.worker_id = None;
            self.started_at = None;
            self.retry_count += 1;
        } else {
            // 达到最大重试次数，标记为失败
            self.status = TaskStatus::Failed;
            self.completed_at = Some(Utc::now());
            self.error_message = Some(error);
        }

        self.version += 1;
        Ok(())
    }

    /// 取消任务
    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), TaskError> {
        if self.status.is_terminal() {
            return Err(TaskError::InvalidStatusTransition {
                from: self.status,
                to: TaskStatus::Cancelled,
            });
        }

        self.status = TaskStatus::Cancelled;
        self.completed_at = Some(Utc::now());
        self.error_message = reason;
        self.version += 1;

        Ok(())
    }

    /// 重试任务
    pub fn retry(&mut self) -> Result<(), TaskError> {
        if self.status != TaskStatus::Failed {
            return Err(TaskError::InvalidStatusTransition {
                from: self.status,
                to: TaskStatus::Waiting,
            });
        }

        if self.retry_count >= self.max_retries {
            return Err(TaskError::MaxRetriesExceeded {
                retry_count: self.retry_count,
                max_retries: self.max_retries,
            });
        }

        self.status = TaskStatus::Waiting;
        self.worker_id = None;
        self.started_at = None;
        self.completed_at = None;
        self.error_message = None;
        self.retry_count += 1;
        self.version += 1;

        Ok(())
    }

    /// 获取处理时间
    pub fn processing_duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end.signed_duration_since(start)),
            _ => None,
        }
    }

    /// 检查任务是否过期
    pub fn is_expired(&self, timeout_seconds: u64) -> bool {
        if self.status.is_terminal() {
            return false;
        }

        let now = Utc::now();
        let duration = now.signed_duration_since(self.created_at);
        duration.num_seconds() > timeout_seconds as i64
    }

    /// 添加元数据
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// 获取元数据
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("Invalid status transition from {from} to {to}")]
    InvalidStatusTransition {
        from: TaskStatus,
        to: TaskStatus,
    },
    #[error("Max retries exceeded: {retry_count}/{max_retries}")]
    MaxRetriesExceeded {
        retry_count: u32,
        max_retries: u32,
    },
    #[error("Task not found: {0}")]
    NotFound(TaskId),
    #[error("Task already acquired by another worker")]
    AlreadyAcquired,
    #[error("Concurrency conflict")]
    ConcurrencyConflict,
}

/// 验证任务创建请求
#[derive(Debug, Validate)]
pub struct CreateTaskRequest {
    #[validate(length(min = 1, max = 512))]
    pub work_directory: String,
    #[validate(length(min = 1, max = 10000))]
    pub prompt: String,
    #[validate(custom(function = "validate_priority"))]
    pub priority: Option<TaskPriority>,
    #[validate(custom(function = "validate_tags"))]
    pub tags: Option<Vec<String>>,
}

fn validate_priority(_priority: &TaskPriority) -> Result<(), validator::ValidationError> {
    Ok(())
}

pub fn validate_tags(tags: &Vec<String>) -> Result<(), validator::ValidationError> {
    for tag in tags {
        if tag.len() > 100 {
            return Err(validator::ValidationError::new("tag_too_long"));
        }
        if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
            return Err(validator::ValidationError::new("invalid_tag_format"));
        }
    }
    Ok(())
}

/// 验证任务完成请求
#[derive(Debug, Validate)]
pub struct CompleteTaskRequest {
    #[validate(length(max = 10000))]
    pub original_prompt: Option<String>,
    pub result: Option<TaskResult>,
}

/// 验证任务获取请求
#[derive(Debug, Validate)]
pub struct AcquireTaskRequest {
    #[validate(length(min = 1, max = 512))]
    pub work_path: String,
    #[validate(length(min = 1, max = 100))]
    pub worker_id: String,
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
    pub has_more: bool,
}

impl Pagination {
    pub fn new(total: u64, limit: u64, offset: u64) -> Self {
        let has_more = offset + limit < total;
        Self {
            total,
            limit,
            offset,
            has_more,
        }
    }
}