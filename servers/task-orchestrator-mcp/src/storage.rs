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