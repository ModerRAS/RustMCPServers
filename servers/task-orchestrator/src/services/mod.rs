use std::sync::Arc;
use chrono::{DateTime, Utc};
use validator::Validate;

use crate::domain::{
    Task, TaskId, TaskStatus, TaskHistory, TaskResult,
    WorkDirectory, Prompt, TaskTag, WorkerId, CreateTaskRequest, 
    CompleteTaskRequest, AcquireTaskRequest,
};
use crate::infrastructure::{TaskRepository, LockManager};
use crate::errors::{AppError, AppResult};
use crate::models::{TaskFilter, TaskStatistics};

/// 任务服务
pub struct TaskService {
    task_repository: Arc<dyn TaskRepository>,
    lock_manager: Arc<dyn LockManager>,
    max_retries: u32,
    task_timeout: u64,
    cleanup_interval: u64,
    timeout_check_interval: u64,
    metrics_interval: u64,
}

impl TaskService {
    /// 创建新的任务服务
    pub fn new(
        task_repository: Arc<dyn TaskRepository>,
        lock_manager: Arc<dyn LockManager>,
        max_retries: u32,
        task_timeout: u64,
    ) -> Self {
        Self {
            task_repository,
            lock_manager,
            max_retries,
            task_timeout,
            cleanup_interval: 300, // 5分钟
            timeout_check_interval: 60, // 1分钟
            metrics_interval: 30, // 30秒
        }
    }

    /// 创建任务
    pub async fn create_task(&self, request: CreateTaskRequest) -> AppResult<Task> {
        // 验证请求
        request.validate().map_err(|e| {
            AppError::Validation(crate::errors::ValidationError::invalid_validation(e.to_string()))
        })?;

        // 创建值对象
        let work_directory = WorkDirectory::new(request.work_directory)?;
        let prompt = Prompt::new(request.prompt)?;
        let priority = request.priority.unwrap_or_default();
        let tags = request.tags
            .unwrap_or_default()
            .into_iter()
            .map(TaskTag::new)
            .collect::<Result<Vec<_>, _>>()?;

        // 创建任务
        let mut task = Task::new(work_directory, prompt, priority, tags);
        task.max_retries = self.max_retries;

        // 保存到数据库
        let task_id = self.task_repository.create_task(&task).await?;

        // 创建任务历史记录
        let history = TaskHistory::new(task_id, task.status, None);
        self.task_repository.create_task_history(&history).await?;

        // 设置任务ID
        task.id = task_id;

        Ok(task)
    }

    /// 获取任务
    pub async fn get_task(&self, task_id: &TaskId) -> AppResult<Task> {
        self.task_repository.get_task(task_id).await?.ok_or(AppError::TaskNotFound(*task_id))
    }

    /// 获取下一个待处理任务
    pub async fn acquire_task(&self, request: AcquireTaskRequest) -> AppResult<Option<Task>> {
        request.validate().map_err(|e| {
            AppError::Validation(crate::errors::ValidationError::invalid_validation(e.to_string()))
        })?;

        // 尝试获取任务
        let task = self.task_repository
            .get_next_task(&request.work_path, &request.worker_id)
            .await?;

        if let Some(ref task) = task {
            // 创建任务历史记录
            let history = TaskHistory::new(
                task.id,
                task.status,
                Some(WorkerId::new(request.worker_id.clone())?),
            );
            self.task_repository.create_task_history(&history).await?;
        }

        Ok(task)
    }

    /// 完成任务
    pub async fn complete_task(&self, task_id: &TaskId, request: CompleteTaskRequest) -> AppResult<Task> {
        // 获取任务
        let mut task = self.get_task(task_id).await?;

        // 验证任务状态
        if task.status != TaskStatus::Working {
            return Err(AppError::Validation(
                crate::errors::ValidationError::invalid_status_transition(
                    task.status,
                    TaskStatus::Completed,
                )
            ));
        }

        // 验证原始提示（如果提供）
        if let Some(original_prompt) = &request.original_prompt {
            if !task.prompt.as_str().contains(original_prompt) {
                return Err(AppError::Validation(
                    crate::errors::ValidationError::InvalidValidation(
                        "Original prompt does not match".to_string()
                    )
                ));
            }
        }

        // 完成任务
        let result = request.result.unwrap_or_else(|| TaskResult::success("Task completed".to_string()));
        task.complete(result)?;

        // 更新任务
        self.task_repository.update_task(&task).await?;

        // 创建任务历史记录
        let history = TaskHistory::new(task.id, task.status, task.worker_id.clone());
        self.task_repository.create_task_history(&history).await?;

        Ok(task)
    }

    /// 任务失败
    pub async fn fail_task(&self, task_id: &TaskId, error: String) -> AppResult<Task> {
        let mut task = self.get_task(task_id).await?;

        // 验证任务状态
        if task.status != TaskStatus::Working {
            return Err(AppError::Validation(
                crate::errors::ValidationError::invalid_status_transition(
                    task.status,
                    TaskStatus::Failed,
                )
            ));
        }

        // 处理失败
        task.fail(error)?;

        // 更新任务
        self.task_repository.update_task(&task).await?;

        // 创建任务历史记录
        let history = TaskHistory::new(task.id, task.status, task.worker_id.clone());
        self.task_repository.create_task_history(&history).await?;

        Ok(task)
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &TaskId, reason: Option<String>) -> AppResult<Task> {
        let mut task = self.get_task(task_id).await?;

        // 验证任务状态
        if task.status.is_terminal() {
            return Err(AppError::Validation(
                crate::errors::ValidationError::invalid_status_transition(
                    task.status,
                    TaskStatus::Cancelled,
                )
            ));
        }

        // 取消任务
        task.cancel(reason)?;

        // 更新任务
        self.task_repository.update_task(&task).await?;

        // 创建任务历史记录
        let history = TaskHistory::new(task.id, task.status, task.worker_id.clone());
        self.task_repository.create_task_history(&history).await?;

        Ok(task)
    }

    /// 重试任务
    pub async fn retry_task(&self, task_id: &TaskId) -> AppResult<Task> {
        let mut task = self.get_task(task_id).await?;

        // 验证任务状态
        if task.status != TaskStatus::Failed {
            return Err(AppError::Validation(
                crate::errors::ValidationError::invalid_status_transition(
                    task.status,
                    TaskStatus::Waiting,
                )
            ));
        }

        // 重试任务
        task.retry()?;

        // 更新任务
        self.task_repository.update_task(&task).await?;

        // 创建任务历史记录
        let history = TaskHistory::new(task.id, task.status, None);
        self.task_repository.create_task_history(&history).await?;

        Ok(task)
    }

    /// 列出任务
    pub async fn list_tasks(&self, filter: TaskFilter) -> AppResult<(Vec<Task>, u64)> {
        self.task_repository.list_tasks(&filter).await
    }

    /// 获取任务统计
    pub async fn get_statistics(&self) -> AppResult<TaskStatistics> {
        self.task_repository.get_statistics().await
    }

    /// 获取任务历史
    pub async fn get_task_history(&self, task_id: &TaskId) -> AppResult<Vec<TaskHistory>> {
        self.task_repository.get_task_history(task_id).await
    }

    /// 清理过期任务
    pub async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> AppResult<u64> {
        self.task_repository.cleanup_expired_tasks(older_than).await
    }

    /// 重试失败任务
    pub async fn retry_failed_tasks(&self) -> AppResult<u64> {
        self.task_repository.retry_failed_tasks(self.max_retries).await
    }

    /// 检查任务是否过期
    pub async fn check_task_timeout(&self, task_id: &TaskId) -> AppResult<bool> {
        let task = self.get_task(task_id).await?;
        Ok(task.is_expired(self.task_timeout))
    }

    /// 处理超时任务
    pub async fn handle_timeout_tasks(&self) -> AppResult<u64> {
        // 查找所有超时的任务
        let filter = TaskFilter::new()
            .with_status(TaskStatus::Working)
            .with_created_before(Utc::now() - chrono::Duration::seconds(self.task_timeout as i64));

        let (tasks, _) = self.list_tasks(filter).await?;
        let mut handled = 0;

        for task in tasks {
            // 标记任务为失败
            if let Err(_) = self.fail_task(&task.id, "Task timeout".to_string()).await {
                continue;
            }
            handled += 1;
        }

        Ok(handled)
    }
}

/// 任务调度器
pub struct TaskScheduler {
    task_service: Arc<TaskService>,
    cleanup_interval: u64,
    timeout_check_interval: u64,
}

impl TaskScheduler {
    pub fn new(
        task_service: Arc<TaskService>,
        cleanup_interval: u64,
        timeout_check_interval: u64,
    ) -> Self {
        Self {
            task_service,
            cleanup_interval,
            timeout_check_interval,
        }
    }

    /// 启动调度器
    pub async fn start(&self) -> AppResult<()> {
        let task_service = self.task_service.clone();
        
        // 启动清理任务
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(task_service.cleanup_interval));
            loop {
                interval.tick().await;
                if let Err(e) = task_service.cleanup_expired_tasks(Utc::now() - chrono::Duration::days(30)).await {
                    tracing::error!("Failed to cleanup expired tasks: {}", e);
                }
            }
        });

        // 启动超时检查任务
        let task_service = self.task_service.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(task_service.timeout_check_interval));
            loop {
                interval.tick().await;
                if let Err(e) = task_service.handle_timeout_tasks().await {
                    tracing::error!("Failed to handle timeout tasks: {}", e);
                }
            }
        });

        Ok(())
    }
}

/// 任务验证器
pub struct TaskValidator {
    max_prompt_length: usize,
    max_work_directory_length: usize,
    max_tag_length: usize,
    max_tags_count: usize,
}

impl TaskValidator {
    pub fn new(
        max_prompt_length: usize,
        max_work_directory_length: usize,
        max_tag_length: usize,
        max_tags_count: usize,
    ) -> Self {
        Self {
            max_prompt_length,
            max_work_directory_length,
            max_tag_length,
            max_tags_count,
        }
    }

    /// 验证任务创建请求
    pub fn validate_create_request(&self, request: &CreateTaskRequest) -> Result<(), AppError> {
        // 验证工作目录
        if request.work_directory.len() > self.max_work_directory_length {
            return Err(AppError::Validation(
                crate::errors::ValidationError::field_too_long("work_directory".to_string())
            ));
        }

        if !request.work_directory.starts_with('/') {
            return Err(AppError::Validation(
                crate::errors::ValidationError::InvalidWorkDirectory(
                    "Work directory must be an absolute path".to_string()
                )
            ));
        }

        // 验证提示
        if request.prompt.len() > self.max_prompt_length {
            return Err(AppError::Validation(
                crate::errors::ValidationError::field_too_long("prompt".to_string())
            ));
        }

        // 验证标签
        if let Some(tags) = &request.tags {
            if tags.len() > self.max_tags_count {
                return Err(AppError::Validation(
                    crate::errors::ValidationError::InvalidValidation(
                        format!("Too many tags (max {})", self.max_tags_count)
                    )
                ));
            }

            for tag in tags {
                if tag.len() > self.max_tag_length {
                    return Err(AppError::Validation(
                        crate::errors::ValidationError::field_too_long("tag".to_string())
                    ));
                }

                if !tag.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
                    return Err(AppError::Validation(
                        crate::errors::ValidationError::InvalidValidation(
                            "Invalid tag format".to_string()
                        )
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证任务获取请求
    pub fn validate_acquire_request(&self, request: &AcquireTaskRequest) -> Result<(), AppError> {
        if request.work_path.len() > self.max_work_directory_length {
            return Err(AppError::Validation(
                crate::errors::ValidationError::field_too_long("work_path".to_string())
            ));
        }

        if request.worker_id.is_empty() || request.worker_id.len() > 100 {
            return Err(AppError::Validation(
                crate::errors::ValidationError::InvalidValidation(
                    "Worker ID must be between 1 and 100 characters".to_string()
                )
            ));
        }

        Ok(())
    }

    /// 验证任务完成请求
    pub fn validate_complete_request(&self, request: &CompleteTaskRequest) -> Result<(), AppError> {
        if let Some(original_prompt) = &request.original_prompt {
            if original_prompt.len() > self.max_prompt_length {
                return Err(AppError::Validation(
                    crate::errors::ValidationError::field_too_long("original_prompt".to_string())
                ));
            }
        }

        Ok(())
    }
}

/// 任务监控器
pub struct TaskMonitor {
    task_service: Arc<TaskService>,
    metrics_interval: u64,
}

impl TaskMonitor {
    pub fn new(task_service: Arc<TaskService>, metrics_interval: u64) -> Self {
        Self {
            task_service,
            metrics_interval,
        }
    }

    /// 启动监控
    pub async fn start(&self) -> AppResult<()> {
        let task_service = self.task_service.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(task_service.metrics_interval));
            loop {
                interval.tick().await;
                
                match task_service.get_statistics().await {
                    Ok(stats) => {
                        tracing::info!(
                            total_tasks = stats.total_tasks,
                            completed_tasks = stats.completed_tasks,
                            failed_tasks = stats.failed_tasks,
                            success_rate = stats.success_rate,
                            active_tasks = stats.active_tasks,
                            "Task statistics"
                        );
                    }
                    Err(e) => {
                        tracing::error!("Failed to get task statistics: {}", e);
                    }
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::{TaskRepository, SqliteLockManager};
    use crate::domain::TaskPriority;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock repository for testing
    #[derive(Clone)]
    struct MockTaskRepository {
        tasks: Arc<Mutex<HashMap<TaskId, Task>>>,
    }

    impl MockTaskRepository {
        fn new() -> Self {
            Self {
                tasks: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait::async_trait]
    impl TaskRepository for MockTaskRepository {
        async fn create_task(&self, task: &Task) -> AppResult<TaskId> {
            Ok(task.id)
        }

        async fn get_task(&self, task_id: &TaskId) -> AppResult<Option<Task>> {
            let tasks = self.tasks.lock().unwrap();
            Ok(tasks.get(task_id).cloned())
        }

        async fn update_task(&self, task: &Task) -> AppResult<()> {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task.id, task.clone());
            Ok(())
        }

        async fn delete_task(&self, _task_id: &TaskId) -> AppResult<()> {
            Ok(())
        }

        async fn get_next_task(&self, _work_directory: &str, _worker_id: &str) -> AppResult<Option<Task>> {
            Ok(None)
        }

        async fn list_tasks(&self, _filter: &TaskFilter) -> AppResult<(Vec<Task>, u64)> {
            Ok((vec![], 0))
        }

        async fn get_statistics(&self) -> AppResult<TaskStatistics> {
            Ok(TaskStatistics::new())
        }

        async fn create_task_history(&self, _history: &TaskHistory) -> AppResult<u64> {
            Ok(1)
        }

        async fn get_task_history(&self, _task_id: &TaskId) -> AppResult<Vec<TaskHistory>> {
            Ok(vec![])
        }

        async fn cleanup_expired_tasks(&self, _older_than: DateTime<Utc>) -> AppResult<u64> {
            Ok(0)
        }

        async fn retry_failed_tasks(&self, _max_retries: u32) -> AppResult<u64> {
            Ok(0)
        }
    }

    // Mock lock manager for testing
    struct MockLockManager;

    #[async_trait::async_trait]
    impl crate::infrastructure::LockManager for MockLockManager {
        async fn try_acquire(&self, _resource_id: &str, _owner_id: &str, _ttl_seconds: u64) -> AppResult<bool> {
            Ok(true)
        }

        async fn release(&self, _resource_id: &str, _owner_id: &str) -> AppResult<bool> {
            Ok(true)
        }

        async fn check_lock(&self, _resource_id: &str) -> AppResult<Option<String>> {
            Ok(None)
        }

        async fn cleanup_expired_locks(&self) -> AppResult<u64> {
            Ok(0)
        }
    }

    #[tokio::test]
    async fn test_create_task() {
        let task_repo = Arc::new(MockTaskRepository::new());
        let lock_manager = Arc::new(MockLockManager);
        let task_service = TaskService::new(task_repo, lock_manager, 3, 3600);

        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: "Test task".to_string(),
            priority: Some(TaskPriority::Medium),
            tags: Some(vec!["test".to_string()]),
        };

        let task = task_service.create_task(request).await.unwrap();
        assert_eq!(task.prompt.as_str(), "Test task");
        assert_eq!(task.priority, TaskPriority::Medium);
        assert_eq!(task.tags.len(), 1);
    }

    #[tokio::test]
    async fn test_complete_task() {
        let task_repo = Arc::new(MockTaskRepository::new());
        let lock_manager = Arc::new(MockLockManager);
        let task_service = TaskService::new(task_repo, lock_manager, 3, 3600);

        // Create a task
        let mut task = Task::new(
            WorkDirectory::new("/test".to_string()).unwrap(),
            Prompt::new("Test task".to_string()).unwrap(),
            TaskPriority::Medium,
            vec![],
        );
        
        let task_id = task.id;
        task.start(WorkerId::new("worker-1".to_string()).unwrap()).unwrap();
        
        // Mock the repository to return the task
        let repo = task_service.task_repository.clone();
        repo.update_task(&task).await.unwrap();

        let request = CompleteTaskRequest {
            original_prompt: Some("Test task".to_string()),
            result: Some(TaskResult::success("Completed".to_string())),
        };

        let completed = task_service.complete_task(&task_id, request).await.unwrap();
        assert_eq!(completed.status, TaskStatus::Completed);
    }
}