#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use crate::infrastructure::*;
    use chrono::Utc;
    use mockall::mock;
    use std::sync::Arc;

    mock! {
        pub TaskRepository {}
        #[async_trait]
        impl TaskRepository for TaskRepository {
            async fn create_task(&self, task: &Task) -> Result<TaskId, String>;
            async fn get_task(&self, task_id: &TaskId) -> Result<Option<Task>, String>;
            async fn update_task(&self, task: &Task) -> Result<(), String>;
            async fn delete_task(&self, task_id: &TaskId) -> Result<(), String>;
            async fn get_next_task(&self, work_directory: &str, worker_id: &str) -> Result<Option<Task>, String>;
            async fn list_tasks(&self, filter: &TaskFilter) -> Result<(Vec<Task>, u64), String>;
            async fn get_statistics(&self) -> Result<TaskStatistics, String>;
            async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> Result<u64, String>;
            async fn retry_failed_tasks(&self, max_retries: u32) -> Result<u64, String>;
        }
    }

    mock! {
        pub LockManager {}
        #[async_trait]
        impl LockManager for LockManager {
            async fn try_acquire(&self, resource_id: &str, owner_id: &str, ttl_seconds: u64) -> Result<bool, String>;
            async fn release(&self, resource_id: &str, owner_id: &str) -> Result<bool, String>;
            async fn check_lock(&self, resource_id: &str) -> Result<Option<String>, String>;
            async fn cleanup_expired_locks(&self) -> Result<u64, String>;
        }
    }

    fn create_task_service() -> TaskService {
        let mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        )
    }

    #[tokio::test]
    async fn test_create_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: "test prompt".to_string(),
            priority: Some(TaskPriority::High),
            tags: Some(vec!["tag1".to_string()]),
        };

        let expected_task_id = TaskId::new();
        mock_repo.expect_create_task()
            .returning(move |_| Ok(expected_task_id.clone()));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.create_task(request).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.work_directory, "/test");
        assert_eq!(task.prompt, "test prompt");
        assert_eq!(task.priority, TaskPriority::High);
        assert_eq!(task.tags, vec!["tag1".to_string()]);
        assert_eq!(task.max_retries, 3);
    }

    #[tokio::test]
    async fn test_create_task_invalid_directory() {
        let service = create_task_service();

        let request = CreateTaskRequest {
            work_directory: "invalid".to_string(), // 不是绝对路径
            prompt: "test prompt".to_string(),
            priority: None,
            tags: None,
        };

        let result = service.create_task(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("absolute path"));
    }

    #[tokio::test]
    async fn test_create_task_directory_too_long() {
        let service = create_task_service();

        let request = CreateTaskRequest {
            work_directory: "/".repeat(1001),
            prompt: "test prompt".to_string(),
            priority: None,
            tags: None,
        };

        let result = service.create_task(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("1000 characters"));
    }

    #[tokio::test]
    async fn test_create_task_invalid_prompt() {
        let service = create_task_service();

        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: "".to_string(), // 空提示
            priority: None,
            tags: None,
        };

        let result = service.create_task(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 10000"));
    }

    #[tokio::test]
    async fn test_create_task_invalid_tags() {
        let service = create_task_service();

        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: "test prompt".to_string(),
            priority: None,
            tags: Some(vec!["".to_string()]), // 空标签
        };

        let result = service.create_task(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("between 1 and 50"));
    }

    #[tokio::test]
    async fn test_get_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let expected_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(expected_task.clone())));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.get_task(&task_id).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.prompt, "test prompt");
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        mock_repo.expect_get_task()
            .returning(move |_| Ok(None));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.get_task(&task_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_acquire_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mut mock_lock = MockLockManager::new();
        
        let request = AcquireTaskRequest {
            work_path: "/test".to_string(),
            worker_id: "worker1".to_string(),
        };

        let expected_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        mock_repo.expect_get_next_task()
            .returning(move |_, _| Ok(Some(expected_task.clone())));
        mock_lock.expect_try_acquire()
            .returning(|_, _, _| Ok(true));
        mock_repo.expect_update_task()
            .returning(|_| Ok(()));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.acquire_task(request).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert!(task.is_some());
        assert_eq!(task.unwrap().prompt, "test prompt");
    }

    #[tokio::test]
    async fn test_acquire_task_no_task_available() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let request = AcquireTaskRequest {
            work_path: "/test".to_string(),
            worker_id: "worker1".to_string(),
        };

        mock_repo.expect_get_next_task()
            .returning(|_, _| Ok(None));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.acquire_task(request).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_acquire_task_lock_failed() {
        let mut mock_repo = MockTaskRepository::new();
        let mut mock_lock = MockLockManager::new();
        
        let request = AcquireTaskRequest {
            work_path: "/test".to_string(),
            worker_id: "worker1".to_string(),
        };

        let expected_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        mock_repo.expect_get_next_task()
            .returning(move |_, _| Ok(Some(expected_task.clone())));
        mock_lock.expect_try_acquire()
            .returning(|_, _, _| Ok(false));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.acquire_task(request).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn test_acquire_task_invalid_worker_id() {
        let service = create_task_service();

        let request = AcquireTaskRequest {
            work_path: "/test".to_string(),
            worker_id: "".to_string(), // 空worker ID
        };

        let result = service.acquire_task(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Worker ID"));
    }

    #[tokio::test]
    async fn test_complete_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mut mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Working;
        existing_task.worker_id = Some(WorkerId::new("worker1".to_string()).unwrap());

        let request = CompleteTaskRequest {
            original_prompt: None,
            result: Some(TaskResult::success("done".to_string())),
        };

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));
        mock_repo.expect_update_task()
            .returning(|_| Ok(()));
        mock_lock.expect_release()
            .returning(|_, _| Ok(true));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.complete_task(&task_id, request).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_complete_task_wrong_status() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Waiting; // 错误的状态

        let request = CompleteTaskRequest {
            original_prompt: None,
            result: Some(TaskResult::success("done".to_string())),
        };

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.complete_task(&task_id, request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot complete task"));
    }

    #[tokio::test]
    async fn test_complete_task_original_prompt_mismatch() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Working;

        let request = CompleteTaskRequest {
            original_prompt: Some("different prompt".to_string()), // 不匹配的提示
            result: Some(TaskResult::success("done".to_string())),
        };

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.complete_task(&task_id, request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not match"));
    }

    #[tokio::test]
    async fn test_fail_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mut mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Working;
        existing_task.worker_id = Some(WorkerId::new("worker1".to_string()).unwrap());

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));
        mock_repo.expect_update_task()
            .returning(|_| Ok(()));
        mock_lock.expect_release()
            .returning(|_, _| Ok(true));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.fail_task(&task_id, "error occurred".to_string()).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.error_message, Some("error occurred".to_string()));
    }

    #[tokio::test]
    async fn test_cancel_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mut mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Working;
        existing_task.worker_id = Some(WorkerId::new("worker1".to_string()).unwrap());

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));
        mock_repo.expect_update_task()
            .returning(|_| Ok(()));
        mock_lock.expect_release()
            .returning(|_, _| Ok(true));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.cancel_task(&task_id, Some("cancelled".to_string())).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_cancel_task_terminal_status() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Completed; // 终端状态

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.cancel_task(&task_id, None).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cannot cancel task"));
    }

    #[tokio::test]
    async fn test_retry_task_success() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Failed;
        existing_task.retry_count = 1;

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));
        mock_repo.expect_update_task()
            .returning(|_| Ok(()));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.retry_task(&task_id).await;
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.status, TaskStatus::Waiting);
        assert_eq!(task.retry_count, 2);
    }

    #[tokio::test]
    async fn test_retry_task_not_failed() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Working; // 不是失败状态

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.retry_task(&task_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Only failed tasks"));
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let filter = TaskFilter::new();
        let expected_tasks = vec![
            Task::new("/test".to_string(), "task1".to_string(), TaskPriority::Medium, vec![]),
            Task::new("/test".to_string(), "task2".to_string(), TaskPriority::High, vec![]),
        ];

        mock_repo.expect_list_tasks()
            .returning(move |_| Ok((expected_tasks.clone(), 2)));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.list_tasks(filter).await;
        assert!(result.is_ok());
        let (tasks, total) = result.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let expected_stats = TaskStatistics {
            total_tasks: 10,
            completed_tasks: 7,
            failed_tasks: 2,
            cancelled_tasks: 1,
            active_tasks: 0,
            success_rate: 0.7,
        };

        mock_repo.expect_get_statistics()
            .returning(move |_| Ok(expected_stats.clone()));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600,
        );

        let result = service.get_statistics().await;
        assert!(result.is_ok());
        let stats = result.unwrap();
        assert_eq!(stats.total_tasks, 10);
        assert_eq!(stats.success_rate, 0.7);
    }

    #[tokio::test]
    async fn test_check_task_timeout() {
        let mut mock_repo = MockTaskRepository::new();
        let mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut existing_task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        existing_task.status = TaskStatus::Working;
        existing_task.started_at = Some(Utc::now() - chrono::Duration::hours(2));

        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(existing_task.clone())));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600, // 1小时超时
        );

        let result = service.check_task_timeout(&task_id).await;
        assert!(result.is_ok());
        assert!(result.unwrap()); // 应该超时
    }

    #[tokio::test]
    async fn test_handle_timeout_tasks() {
        let mut mock_repo = MockTaskRepository::new();
        let mut mock_lock = MockLockManager::new();
        
        let task_id = TaskId::new();
        let mut timeout_task = Task::new(
            "/test".to_string(),
            "timeout task".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        timeout_task.status = TaskStatus::Working;
        timeout_task.started_at = Some(Utc::now() - chrono::Duration::hours(2));
        timeout_task.worker_id = Some(WorkerId::new("worker1".to_string()).unwrap());

        let working_tasks = vec![timeout_task.clone()];

        mock_repo.expect_list_tasks()
            .returning(move |_| Ok((working_tasks.clone(), 1)));
        mock_repo.expect_get_task()
            .returning(move |_| Ok(Some(timeout_task.clone())));
        mock_repo.expect_update_task()
            .returning(|_| Ok(()));
        mock_lock.expect_release()
            .returning(|_, _| Ok(true));

        let service = TaskService::new(
            Arc::new(mock_repo),
            Arc::new(mock_lock),
            3,
            3600, // 1小时超时
        );

        let result = service.handle_timeout_tasks().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }
}