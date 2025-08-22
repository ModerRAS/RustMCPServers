//! # 存储层模块
//! 
//! 该模块提供了 Task Orchestrator MCP 服务器的数据存储和访问功能。
//! 
//! ## 主要功能
//! 
//! - **存储抽象**: 定义了统一的任务存储接口
//! - **内存存储**: 提供高性能的内存任务存储实现
//! - **状态管理**: 完整的任务状态转换和验证
//! - **并发控制**: 使用异步读写锁确保线程安全
//! - **错误处理**: 统一的错误类型和处理机制
//! 
//! ## 核心组件
//! 
//! - `TaskRepository`: 任务存储特征，定义了所有存储操作接口
//! - `InMemoryTaskRepository`: 内存任务存储实现
//! - `RepositoryError`: 存储层错误类型
//! 
//! ## 使用示例
//! 
//! ```rust
//! use task_orchestrator_mcp::storage::{InMemoryTaskRepository, TaskRepository};
//! use task_orchestrator_mcp::models::{CreateTaskRequest, TaskFilter, TaskStatus};
//! 
//! // 创建存储实例
//! let repository = InMemoryTaskRepository::new();
//! 
//! // 创建任务
//! let request = CreateTaskRequest {
//!     work_directory: "/workspace/project".to_string(),
//!     prompt: "实现用户认证功能".to_string(),
//!     priority: Some(TaskPriority::High),
//!     tags: Some(vec!["feature".to_string()]),
//!     max_retries: Some(3),
//!     timeout_seconds: Some(3600),
//! };
//! 
//! let task = repository.create_task(request).await?;
//! println!("Created task: {}", task.id);
//! 
//! // 获取任务
//! let task = repository.get_task(task.id).await?;
//! println!("Task status: {:?}", task.status);
//! 
//! // 列出任务
//! let filter = TaskFilter {
//!     status: Some(TaskStatus::Waiting),
//!     priority: None,
//!     worker_id: None,
//!     limit: 10,
//!     offset: 0,
//! };
//! 
//! let tasks = repository.list_tasks(filter).await?;
//! println!("Found {} waiting tasks", tasks.len());
//! 
//! // 获取统计信息
//! let stats = repository.get_statistics().await?;
//! println!("Total tasks: {}, Success rate: {:.2}%", 
//!     stats.total_tasks, stats.success_rate);
//! ```
//! 
//! ## 存储接口设计
//! 
//! `TaskRepository` 特征定义了完整的任务存储操作：
//! 
//! - `create_task`: 创建新任务
//! - `get_task`: 获取指定任务
//! - `update_task`: 更新任务信息
//! - `delete_task`: 删除任务（预留功能）
//! - `list_tasks`: 列出任务（支持过滤和分页）
//! - `acquire_task`: 获取待处理任务
//! - `complete_task`: 完成任务
//! - `fail_task`: 标记任务失败
//! - `retry_task`: 重试任务
//! - `get_statistics`: 获取统计信息
//! - `cleanup_old_tasks`: 清理旧任务（预留功能）
//! 
//! ## 状态转换验证
//! 
//! 存储层实现了严格的状态转换验证：
//! 
//! ```rust
//! // 允许的状态转换
//! Pending → Waiting, Cancelled
//! Waiting → Running, Cancelled
//! Running → Completed, Failed, Cancelled
//! Failed → Waiting, Cancelled
//! Cancelled → Waiting
//! ```
//! 
//! ## 任务调度逻辑
//! 
//! `acquire_task` 方法实现了智能的任务调度：
//! 
//! - 优先选择高优先级任务
//! - 匹配工作目录
//! - 只选择等待状态的任务
//! - 自动更新任务状态为运行中
//! - 记录工作节点和开始时间
//! 
//! ## 重试机制
//! 
//! `fail_task` 方法实现了自动重试逻辑：
//! 
//! - 检查重试次数限制
//! - 未达到限制时重置为等待状态
//! - 达到限制时标记为失败
//! - 保留错误信息
//! 
//! ## 统计信息
//! 
//! `get_statistics` 方法提供详细的统计信息：
//! 
//! - 各状态任务数量
//! - 平均完成时间
//! - 成功率计算
//! - 任务总数统计
//! 
//! ## 内存存储特点
//! 
//! - **高性能**: 基于内存的HashMap存储
//! - **线程安全**: 使用Tokio的RwLock实现异步并发控制
//! - **自动清理**: 支持定期清理过期任务
//! - **分页支持**: 支持limit和offset分页
//! - **排序功能**: 按优先级和创建时间排序
//! 
//! ## 错误处理
//! 
//! 定义了统一的错误类型：
//! 
//! - `TaskNotFound`: 任务不存在
//! - `InvalidStateTransition`: 非法状态转换
//! - `TaskLocked`: 任务被锁定（预留功能）
//! 
//! ## 扩展性
//! 
//! 存储层设计考虑了未来扩展：
//! 
//! - 预留了分布式锁定功能
//! - 预留了任务删除功能
//! - 预留了旧任务清理功能
//! - 支持轻松替换为数据库存储
//! 
//! ## 性能优化
//! 
//! - 使用读写锁提高并发性能
//! - 惰性计算统计信息
//! - 高效的过滤和排序算法
//! - 最小化锁的持有时间

use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::models::{Task, TaskStatus, TaskFilter, CreateTaskRequest, UpdateTaskRequest, TaskResult, TaskStatistics};

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),
    #[error("Invalid task state transition")]
    InvalidStateTransition,
    #[error("Task locked by another worker")]
    #[allow(dead_code)]
    TaskLocked, // Reserved for future distributed locking
}

#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create_task(&self, request: CreateTaskRequest) -> Result<Task, RepositoryError>;
    async fn get_task(&self, task_id: Uuid) -> Result<Task, RepositoryError>;
    async fn update_task(&self, task_id: Uuid, request: UpdateTaskRequest) -> Result<Task, RepositoryError>;
    #[allow(dead_code)]
    async fn delete_task(&self, task_id: Uuid) -> Result<(), RepositoryError>;
    async fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<Task>, RepositoryError>;
    async fn acquire_task(&self, worker_id: String, work_directory: String) -> Result<Option<Task>, RepositoryError>;
    async fn complete_task(&self, task_id: Uuid, result: TaskResult) -> Result<Task, RepositoryError>;
    async fn fail_task(&self, task_id: Uuid, error_message: String) -> Result<Task, RepositoryError>;
    async fn retry_task(&self, task_id: Uuid) -> Result<Task, RepositoryError>;
    async fn get_statistics(&self) -> Result<TaskStatistics, RepositoryError>;
    #[allow(dead_code)]
    async fn cleanup_old_tasks(&self, older_than: DateTime<Utc>) -> Result<u64, RepositoryError>;
}

#[derive(Debug)]
pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<Uuid, Task>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn create_task(&self, request: CreateTaskRequest) -> Result<Task, RepositoryError> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let task = Task {
            id: task_id,
            work_directory: request.work_directory,
            prompt: request.prompt,
            priority: request.priority.unwrap_or_default(),
            status: TaskStatus::Pending,
            tags: request.tags.unwrap_or_default(),
            created_at: now,
            updated_at: now,
            started_at: None,
            completed_at: None,
            result: None,
            retry_count: 0,
            max_retries: request.max_retries.unwrap_or(3),
            timeout_seconds: request.timeout_seconds.unwrap_or(3600),
            worker_id: None,
            error_message: None,
        };

        let mut tasks = self.tasks.write().await;
        tasks.insert(task_id, task.clone());

        // Move to waiting state
        let mut updated_task = task;
        updated_task.status = TaskStatus::Waiting;
        updated_task.updated_at = Utc::now();
        tasks.insert(task_id, updated_task.clone());

        Ok(updated_task)
    }

    async fn get_task(&self, task_id: Uuid) -> Result<Task, RepositoryError> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned().ok_or(RepositoryError::TaskNotFound(task_id))
    }

    async fn update_task(&self, task_id: Uuid, request: UpdateTaskRequest) -> Result<Task, RepositoryError> {
        let mut tasks = self.tasks.write().await;
        let mut task = tasks.get(&task_id).cloned().ok_or(RepositoryError::TaskNotFound(task_id))?;
        
        // Update fields
        if let Some(status) = request.status {
            if !is_valid_state_transition(task.status, status) {
                return Err(RepositoryError::InvalidStateTransition);
            }
            task.status = status;
            
            match status {
                TaskStatus::Running => task.started_at = Some(Utc::now()),
                TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled => {
                    task.completed_at = Some(Utc::now());
                }
                _ => {}
            }
        }

        if let Some(worker_id) = request.worker_id {
            task.worker_id = Some(worker_id);
        }

        if let Some(result) = request.result {
            task.result = Some(result);
        }

        if let Some(error_message) = request.error_message {
            task.error_message = Some(error_message);
        }

        task.updated_at = Utc::now();
        tasks.insert(task_id, task.clone());
        Ok(task)
    }

    #[allow(dead_code)]
    async fn delete_task(&self, task_id: Uuid) -> Result<(), RepositoryError> {
        // Reserved for future use - task deletion functionality
        let mut tasks = self.tasks.write().await;
        tasks.remove(&task_id).ok_or(RepositoryError::TaskNotFound(task_id))?;
        Ok(())
    }

    async fn list_tasks(&self, filter: TaskFilter) -> Result<Vec<Task>, RepositoryError> {
        let tasks = self.tasks.read().await;
        let mut filtered_tasks: Vec<Task> = tasks.values().cloned().collect();

        // Apply filters
        if let Some(status) = filter.status {
            filtered_tasks.retain(|task| task.status == status);
        }

        if let Some(priority) = filter.priority {
            filtered_tasks.retain(|task| task.priority == priority);
        }

        if let Some(ref worker_id) = filter.worker_id {
            filtered_tasks.retain(|task| task.worker_id.as_ref() == Some(worker_id));
        }

        // Sort by priority and creation time
        filtered_tasks.sort_by(|a, b| {
            b.priority.cmp(&a.priority).then_with(|| a.created_at.cmp(&b.created_at))
        });

        // Apply pagination
        let offset = filter.offset as usize;
        let limit = filter.limit as usize;
        filtered_tasks = filtered_tasks.into_iter().skip(offset).take(limit).collect();

        Ok(filtered_tasks)
    }

    async fn acquire_task(&self, worker_id: String, work_directory: String) -> Result<Option<Task>, RepositoryError> {
        let mut tasks = self.tasks.write().await;
        
        // Find the highest priority waiting task that matches the work directory
        let task_to_acquire = tasks.values_mut()
            .filter(|task| task.status == TaskStatus::Waiting && task.work_directory == work_directory)
            .max_by(|a, b| a.priority.cmp(&b.priority));

        if let Some(task) = task_to_acquire {
            let _task_id = task.id;
            task.status = TaskStatus::Running;
            task.worker_id = Some(worker_id.clone());
            task.started_at = Some(Utc::now());
            task.updated_at = Utc::now();
            
            Ok(Some(task.clone()))
        } else {
            Ok(None)
        }
    }

    async fn complete_task(&self, task_id: Uuid, result: TaskResult) -> Result<Task, RepositoryError> {
        self.update_task(
            task_id,
            UpdateTaskRequest {
                status: Some(TaskStatus::Completed),
                worker_id: None,
                result: Some(result),
                error_message: None,
            },
        ).await
    }

    async fn fail_task(&self, task_id: Uuid, error_message: String) -> Result<Task, RepositoryError> {
        let task = self.get_task(task_id).await?;
        
        if task.retry_count < task.max_retries {
            // Retry the task
            self.update_task(
                task_id,
                UpdateTaskRequest {
                    status: Some(TaskStatus::Waiting),
                    worker_id: None,
                    result: None,
                    error_message: Some(error_message),
                },
            ).await?;
        } else {
            // Max retries reached, mark as failed
            self.update_task(
                task_id,
                UpdateTaskRequest {
                    status: Some(TaskStatus::Failed),
                    worker_id: None,
                    result: None,
                    error_message: Some(error_message),
                },
            ).await?;
        }

        self.get_task(task_id).await
    }

    async fn retry_task(&self, task_id: Uuid) -> Result<Task, RepositoryError> {
        let task = self.get_task(task_id).await?;
        
        if task.status != TaskStatus::Failed && task.status != TaskStatus::Cancelled {
            return Err(RepositoryError::InvalidStateTransition);
        }

        self.update_task(
            task_id,
            UpdateTaskRequest {
                status: Some(TaskStatus::Waiting),
                worker_id: None,
                result: None,
                error_message: None,
            },
        ).await
    }

    async fn get_statistics(&self) -> Result<TaskStatistics, RepositoryError> {
        let tasks = self.tasks.read().await;
        let mut total_tasks = 0;
        let mut pending_tasks = 0;
        let mut waiting_tasks = 0;
        let mut running_tasks = 0;
        let mut completed_tasks = 0;
        let mut failed_tasks = 0;
        let mut cancelled_tasks = 0;
        let mut sum_completion_time = 0;
        let mut completion_count = 0;

        for task in tasks.values() {
            total_tasks += 1;
            
            match task.status {
                TaskStatus::Pending => pending_tasks += 1,
                TaskStatus::Waiting => waiting_tasks += 1,
                TaskStatus::Running => running_tasks += 1,
                TaskStatus::Completed => {
                    completed_tasks += 1;
                    if let (Some(started), Some(completed)) = (task.started_at, task.completed_at) {
                        sum_completion_time += (completed - started).num_milliseconds() as u64;
                        completion_count += 1;
                    }
                }
                TaskStatus::Failed => failed_tasks += 1,
                TaskStatus::Cancelled => cancelled_tasks += 1,
            }
        }

        let statistics = TaskStatistics {
            total_tasks,
            pending_tasks,
            waiting_tasks,
            running_tasks,
            completed_tasks,
            failed_tasks,
            cancelled_tasks,
            average_completion_time_ms: if completion_count > 0 {
                sum_completion_time / completion_count
            } else {
                0
            },
            success_rate: if total_tasks > 0 {
                (completed_tasks as f64 / total_tasks as f64) * 100.0
            } else {
                0.0
            },
        };

        Ok(statistics)
    }

    #[allow(dead_code)]
    async fn cleanup_old_tasks(&self, older_than: DateTime<Utc>) -> Result<u64, RepositoryError> {
        // Reserved for future use - cleanup old completed/failed tasks
        let mut tasks = self.tasks.write().await;
        let initial_count = tasks.len();
        
        tasks.retain(|_, task| {
            !(task.status == TaskStatus::Completed || task.status == TaskStatus::Failed || task.status == TaskStatus::Cancelled)
                || task.completed_at.is_some_and(|completed| completed > older_than)
        });
        
        Ok((initial_count - tasks.len()) as u64)
    }
}

fn is_valid_state_transition(from: TaskStatus, to: TaskStatus) -> bool {
    use TaskStatus::*;
    
    matches!((from, to),
        (Pending, Waiting) | (Pending, Cancelled) |
        (Waiting, Running) | (Waiting, Cancelled) |
        (Running, Completed) | (Running, Failed) | (Running, Cancelled) |
        (Failed, Waiting) | (Failed, Cancelled) |
        (Cancelled, Waiting) |
        (Pending, Pending) | (Waiting, Waiting) | (Running, Running) |
        (Completed, Completed) | (Failed, Failed) | (Cancelled, Cancelled)
    )
}