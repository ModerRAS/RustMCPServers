use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::domain::{TaskStatus, TaskPriority, TaskId, WorkDirectory, Prompt, TaskTag, WorkerId};

/// 数据库任务记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TaskRecord {
    pub id: i32,
    pub task_id: String,
    pub work_directory: String,
    pub prompt: String,
    pub priority: String,
    pub tags: Option<String>,
    pub status: String,
    pub worker_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub max_retries: i32,
    pub metadata: Option<String>,
    pub version: i32,
    pub updated_at: DateTime<Utc>,
}

impl TaskRecord {
    /// 转换为领域模型
    pub fn to_domain(self) -> Result<crate::domain::Task, anyhow::Error> {
        let tags = self.tags
            .map(|tags| serde_json::from_str::<Vec<String>>(&tags))
            .transpose()?
            .unwrap_or_default()
            .into_iter()
            .map(TaskTag::new)
            .collect::<Result<Vec<_>, _>>()?;

        let metadata = self.metadata
            .map(|meta| serde_json::from_str::<HashMap<String, serde_json::Value>>(&meta))
            .transpose()?
            .unwrap_or_default();

        let result = self.result
            .map(|result| serde_json::from_str::<crate::domain::TaskResult>(&result))
            .transpose()?;

        Ok(crate::domain::Task {
            id: TaskId::from_str(&self.task_id)?,
            work_directory: WorkDirectory::new(self.work_directory)?,
            prompt: Prompt::new(self.prompt)?,
            priority: TaskPriority::from_str(&self.priority)?,
            tags,
            status: TaskStatus::from_str(&self.status)?,
            worker_id: self.worker_id.map(WorkerId::new),
            created_at: self.created_at,
            started_at: self.started_at,
            completed_at: self.completed_at,
            result,
            error_message: self.error_message,
            retry_count: self.retry_count as u32,
            max_retries: self.max_retries as u32,
            metadata,
            version: self.version as u32,
        })
    }

    /// 从领域模型创建记录
    pub fn from_domain(task: &crate::domain::Task) -> Result<Self, anyhow::Error> {
        let tags = if task.tags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&task.tags.iter().map(|t| t.as_str()).collect::<Vec<_>>())?)
        };

        let result = task.result.as_ref().map(|r| serde_json::to_string(r)).transpose()?;

        let metadata = if task.metadata.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&task.metadata)?)
        };

        Ok(Self {
            id: 0, // 数据库自动生成
            task_id: task.id.to_string(),
            work_directory: task.work_directory.as_str().to_string(),
            prompt: task.prompt.as_str().to_string(),
            priority: task.priority.to_string(),
            tags,
            status: task.status.to_string(),
            worker_id: task.worker_id.as_ref().map(|w| w.as_str().to_string()),
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
            result,
            error_message: task.error_message.clone(),
            retry_count: task.retry_count as i32,
            max_retries: task.max_retries as i32,
            metadata,
            version: task.version as i32,
            updated_at: Utc::now(),
        })
    }
}

/// 任务历史记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct TaskHistoryRecord {
    pub id: i32,
    pub task_id: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub details: Option<String>,
}

impl TaskHistoryRecord {
    /// 转换为领域模型
    pub fn to_domain(self) -> Result<crate::domain::TaskHistory, anyhow::Error> {
        let details = self.details
            .map(|details| serde_json::from_str::<HashMap<String, serde_json::Value>>(&details))
            .transpose()?
            .unwrap_or_default();

        Ok(crate::domain::TaskHistory {
            id: self.id as u64,
            task_id: TaskId::from_str(&self.task_id)?,
            status: TaskStatus::from_str(&self.status)?,
            worker_id: self.worker_id.map(WorkerId::new),
            changed_at: self.changed_at,
            details,
        })
    }

    /// 从领域模型创建记录
    pub fn from_domain(history: &crate::domain::TaskHistory) -> Result<Self, anyhow::Error> {
        let details = if history.details.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&history.details)?)
        };

        Ok(Self {
            id: history.id as i32,
            task_id: history.task_id.to_string(),
            status: history.status.to_string(),
            worker_id: history.worker_id.as_ref().map(|w| w.as_str().to_string()),
            changed_at: history.changed_at,
            details,
        })
    }
}

/// 锁记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct LockRecord {
    pub resource_id: String,
    pub owner_id: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl LockRecord {
    pub fn new(resource_id: String, owner_id: String, expires_at: DateTime<Utc>) -> Self {
        Self {
            resource_id,
            owner_id,
            expires_at,
            created_at: Utc::now(),
        }
    }
}

/// 系统配置记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SystemConfigRecord {
    pub key: String,
    pub value: String,
    pub description: Option<String>,
    pub updated_at: DateTime<Utc>,
}

/// 性能指标记录
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct PerformanceMetricRecord {
    pub id: i32,
    pub metric_name: String,
    pub metric_value: f64,
    pub timestamp: DateTime<Utc>,
    pub tags: Option<String>,
}

impl PerformanceMetricRecord {
    pub fn new(metric_name: String, metric_value: f64, tags: Option<HashMap<String, String>>) -> Result<Self, anyhow::Error> {
        let tags = tags.map(|t| serde_json::to_string(&t)).transpose()?;
        
        Ok(Self {
            id: 0,
            metric_name,
            metric_value,
            timestamp: Utc::now(),
            tags,
        })
    }
}

/// 任务查询过滤器
#[derive(Debug, Clone, Default)]
pub struct TaskFilter {
    pub status: Option<TaskStatus>,
    pub work_directory: Option<String>,
    pub priority: Option<TaskPriority>,
    pub tags: Option<Vec<String>>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub worker_id: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

impl TaskFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: TaskStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_work_directory(mut self, work_directory: String) -> Self {
        self.work_directory = Some(work_directory);
        self
    }

    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = Some(priority);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_created_after(mut self, after: DateTime<Utc>) -> Self {
        self.created_after = Some(after);
        self
    }

    pub fn with_created_before(mut self, before: DateTime<Utc>) -> Self {
        self.created_before = Some(before);
        self
    }

    pub fn with_worker_id(mut self, worker_id: String) -> Self {
        self.worker_id = Some(worker_id);
        self
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn with_sort_by(mut self, sort_by: String) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    pub fn with_sort_order(mut self, sort_order: String) -> Self {
        self.sort_order = Some(sort_order);
        self
    }
}

/// 任务统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub cancelled_tasks: u64,
    pub active_tasks: u64,
    pub waiting_tasks: u64,
    pub working_tasks: u64,
    pub success_rate: f64,
    pub avg_processing_time: f64,
    pub tasks_per_hour: f64,
}

impl TaskStatistics {
    pub fn new() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            cancelled_tasks: 0,
            active_tasks: 0,
            waiting_tasks: 0,
            working_tasks: 0,
            success_rate: 0.0,
            avg_processing_time: 0.0,
            tasks_per_hour: 0.0,
        }
    }
}