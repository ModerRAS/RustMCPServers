use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// 任务ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 工作节点ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerId(String);

impl WorkerId {
    pub fn new(id: String) -> Result<Self, String> {
        if id.is_empty() || id.len() > 100 {
            return Err("Worker ID must be between 1 and 100 characters".to_string());
        }
        Ok(Self(id))
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn to_string(&self) -> String {
        self.0.clone()
    }
}

impl std::fmt::Display for WorkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Waiting,
    Working,
    Completed,
    Failed,
    Cancelled,
}

impl TaskStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
    }
    
    pub fn can_transition_to(&self, new_status: &TaskStatus) -> bool {
        use TaskStatus::*;
        match (self, new_status) {
            (Waiting, Working) => true,
            (Waiting, Cancelled) => true,
            (Working, Completed) => true,
            (Working, Failed) => true,
            (Working, Cancelled) => true,
            (Failed, Waiting) => true, // 重试
            _ => false,
        }
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Waiting
    }
}

/// 任务优先级
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
}

impl TaskPriority {
    pub fn as_i32(&self) -> i32 {
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

/// 任务执行方式
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// 标准执行模式
    Standard,
    /// 使用Claude Code执行
    ClaudeCode,
    /// 自定义执行器
    Custom(String),
}

impl Default for ExecutionMode {
    fn default() -> Self {
        ExecutionMode::Standard
    }
}

/// 任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub status: String,
    pub output: String,
    pub duration_ms: u64,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl TaskResult {
    pub fn success(output: String) -> Self {
        Self {
            status: "success".to_string(),
            output,
            duration_ms: 0,
            metadata: None,
        }
    }
    
    pub fn failure(error: String) -> Self {
        Self {
            status: "failure".to_string(),
            output: error,
            duration_ms: 0,
            metadata: None,
        }
    }
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub work_directory: String,
    pub prompt: String,
    pub priority: TaskPriority,
    pub execution_mode: ExecutionMode,
    pub tags: Vec<String>,
    pub status: TaskStatus,
    pub worker_id: Option<WorkerId>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Task {
    pub fn new(work_directory: String, prompt: String, priority: TaskPriority, tags: Vec<String>) -> Self {
        Self {
            id: TaskId::new(),
            work_directory,
            prompt,
            priority,
            execution_mode: ExecutionMode::Standard,
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
            metadata: None,
        }
    }

    pub fn with_execution_mode(mut self, execution_mode: ExecutionMode) -> Self {
        self.execution_mode = execution_mode;
        self
    }
    
    pub fn start(&mut self, worker_id: WorkerId) -> Result<(), String> {
        if !self.status.can_transition_to(&TaskStatus::Working) {
            return Err(format!("Cannot transition from {:?} to Working", self.status));
        }
        
        self.status = TaskStatus::Working;
        self.worker_id = Some(worker_id);
        self.started_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn complete(&mut self, result: TaskResult) -> Result<(), String> {
        if !self.status.can_transition_to(&TaskStatus::Completed) {
            return Err(format!("Cannot transition from {:?} to Completed", self.status));
        }
        
        self.status = TaskStatus::Completed;
        self.result = Some(result);
        self.completed_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn fail(&mut self, error: String) -> Result<(), String> {
        if !self.status.can_transition_to(&TaskStatus::Failed) {
            return Err(format!("Cannot transition from {:?} to Failed", self.status));
        }
        
        self.status = TaskStatus::Failed;
        self.error_message = Some(error);
        self.completed_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn cancel(&mut self, reason: Option<String>) -> Result<(), String> {
        if !self.status.can_transition_to(&TaskStatus::Cancelled) {
            return Err(format!("Cannot transition from {:?} to Cancelled", self.status));
        }
        
        self.status = TaskStatus::Cancelled;
        self.error_message = reason;
        self.completed_at = Some(Utc::now());
        Ok(())
    }
    
    pub fn retry(&mut self) -> Result<(), String> {
        if self.status != TaskStatus::Failed {
            return Err("Only failed tasks can be retried".to_string());
        }
        
        if self.retry_count >= self.max_retries {
            return Err("Maximum retry count exceeded".to_string());
        }
        
        self.status = TaskStatus::Waiting;
        self.worker_id = None;
        self.started_at = None;
        self.completed_at = None;
        self.error_message = None;
        self.retry_count += 1;
        Ok(())
    }
    
    pub fn is_expired(&self, timeout_seconds: u64) -> bool {
        if let Some(started_at) = self.started_at {
            Utc::now().signed_duration_since(started_at).num_seconds() > timeout_seconds as i64
        } else {
            false
        }
    }
}

/// 创建任务请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTaskRequest {
    pub work_directory: String,
    pub prompt: String,
    pub priority: Option<TaskPriority>,
    pub execution_mode: Option<ExecutionMode>,
    pub tags: Option<Vec<String>>,
}

/// 获取任务请求
#[derive(Debug, Serialize, Deserialize)]
pub struct AcquireTaskRequest {
    pub work_path: String,
    pub worker_id: String,
}

/// 完成任务请求
#[derive(Debug, Serialize, Deserialize)]
pub struct CompleteTaskRequest {
    pub original_prompt: Option<String>,
    pub result: Option<TaskResult>,
}

/// 任务过滤器
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub worker_id: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

impl TaskFilter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }
    
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }
    
    pub fn with_worker_id(mut self, worker_id: String) -> Self {
        self.worker_id = Some(worker_id);
        self
    }
    
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
    
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// 任务统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub cancelled_tasks: u64,
    pub active_tasks: u64,
    pub success_rate: f64,
}

impl TaskStatistics {
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            cancelled_tasks: 0,
            active_tasks: 0,
            success_rate: 0.0,
        }
    }
}

/// 分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
    pub has_more: bool,
}

/// API响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub timestamp: DateTime<Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: Utc::now(),
        }
    }
    
    pub fn error(error: ApiError) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            timestamp: Utc::now(),
        }
    }
}

/// API错误
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    pub fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
        }
    }
    
    pub fn with_details(code: String, message: String, details: serde_json::Value) -> Self {
        Self {
            code,
            message,
            details: Some(details),
        }
    }
}