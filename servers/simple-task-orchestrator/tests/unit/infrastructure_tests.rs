#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_in_memory_task_repository_new() {
        let repo = InMemoryTaskRepository::new();
        assert_eq!(repo.tasks.read().await.len(), 0);
        assert_eq!(repo.worker_tasks.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_create_task() {
        let repo = InMemoryTaskRepository::new();
        let task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        let task_id = repo.create_task(&task).await.unwrap();
        assert_eq!(repo.tasks.read().await.len(), 1);
        assert!(repo.tasks.read().await.contains_key(&task_id));
    }

    #[tokio::test]
    async fn test_get_task() {
        let repo = InMemoryTaskRepository::new();
        let task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        let task_id = repo.create_task(&task).await.unwrap();
        let retrieved_task = repo.get_task(&task_id).await.unwrap();

        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().id, task_id);
    }

    #[tokio::test]
    async fn test_get_nonexistent_task() {
        let repo = InMemoryTaskRepository::new();
        let fake_id = TaskId::new();

        let result = repo.get_task(&fake_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_update_task() {
        let repo = InMemoryTaskRepository::new();
        let mut task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        let task_id = repo.create_task(&task).await.unwrap();
        
        // 修改任务
        task.prompt = "updated prompt".to_string();
        repo.update_task(&task).await.unwrap();

        let retrieved_task = repo.get_task(&task_id).await.unwrap().unwrap();
        assert_eq!(retrieved_task.prompt, "updated prompt");
    }

    #[tokio::test]
    async fn test_delete_task() {
        let repo = InMemoryTaskRepository::new();
        let task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        let task_id = repo.create_task(&task).await.unwrap();
        assert_eq!(repo.tasks.read().await.len(), 1);

        repo.delete_task(&task_id).await.unwrap();
        assert_eq!(repo.tasks.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_get_next_task_no_tasks() {
        let repo = InMemoryTaskRepository::new();
        let result = repo.get_next_task("/test", "worker1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_next_task_with_tasks() {
        let repo = InMemoryTaskRepository::new();
        
        // 创建任务
        let task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        let task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );
        let task3 = Task::new(
            "/other".to_string(),
            "test prompt 3".to_string(),
            TaskPriority::High,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();
        repo.create_task(&task3).await.unwrap();

        // 获取任务，应该返回高优先级的task2
        let result = repo.get_next_task("/test", "worker1").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().prompt, "test prompt 2");
    }

    #[tokio::test]
    async fn test_get_next_task_wrong_directory() {
        let repo = InMemoryTaskRepository::new();
        
        let task = Task::new(
            "/test".to_string(),
            "test prompt".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        repo.create_task(&task).await.unwrap();

        // 请求不同目录的任务
        let result = repo.get_next_task("/other", "worker1").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_list_tasks_no_filter() {
        let repo = InMemoryTaskRepository::new();
        
        let task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        let task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();

        let filter = TaskFilter::new();
        let (tasks, total) = repo.list_tasks(&filter).await.unwrap();

        assert_eq!(total, 2);
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_list_tasks_with_status_filter() {
        let repo = InMemoryTaskRepository::new();
        
        let mut task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        task1.status = TaskStatus::Completed;

        let task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();

        let filter = TaskFilter::new().with_status(TaskStatus::Completed);
        let (tasks, total) = repo.list_tasks(&filter).await.unwrap();

        assert_eq!(total, 1);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].status, TaskStatus::Completed);
    }

    #[tokio::test]
    async fn test_list_tasks_with_priority_filter() {
        let repo = InMemoryTaskRepository::new();
        
        let task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        let task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();

        let filter = TaskFilter::new().with_priority(TaskPriority::High);
        let (tasks, total) = repo.list_tasks(&filter).await.unwrap();

        assert_eq!(total, 1);
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].priority, TaskPriority::High);
    }

    #[tokio::test]
    async fn test_list_tasks_with_limit() {
        let repo = InMemoryTaskRepository::new();
        
        for i in 0..5 {
            let task = Task::new(
                "/test".to_string(),
                format!("test prompt {}", i),
                TaskPriority::Medium,
                vec![],
            );
            repo.create_task(&task).await.unwrap();
        }

        let filter = TaskFilter::new().with_limit(2);
        let (tasks, total) = repo.list_tasks(&filter).await.unwrap();

        assert_eq!(total, 5);
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_get_statistics() {
        let repo = InMemoryTaskRepository::new();
        
        let mut task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        task1.status = TaskStatus::Completed;

        let mut task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );
        task2.status = TaskStatus::Failed;

        let task3 = Task::new(
            "/test".to_string(),
            "test prompt 3".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();
        repo.create_task(&task3).await.unwrap();

        let stats = repo.get_statistics().await.unwrap();

        assert_eq!(stats.total_tasks, 3);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
        assert_eq!(stats.cancelled_tasks, 0);
        assert_eq!(stats.active_tasks, 1);
        assert_eq!(stats.success_rate, 1.0 / 3.0);
    }

    #[tokio::test]
    async fn test_cleanup_expired_tasks() {
        let repo = InMemoryTaskRepository::new();
        
        let mut task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        task1.created_at = Utc::now() - chrono::Duration::days(31);
        task1.status = TaskStatus::Completed;

        let task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );
        task2.created_at = Utc::now() - chrono::Duration::days(31);
        task2.status = TaskStatus::Waiting; // 非终端状态

        let task3 = Task::new(
            "/test".to_string(),
            "test prompt 3".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();
        repo.create_task(&task3).await.unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(30);
        let cleaned = repo.cleanup_expired_tasks(cutoff).await.unwrap();

        assert_eq!(cleaned, 1); // 只有task2应该被清理
        assert_eq!(repo.tasks.read().await.len(), 2);
    }

    #[tokio::test]
    async fn test_retry_failed_tasks() {
        let repo = InMemoryTaskRepository::new();
        
        let mut task1 = Task::new(
            "/test".to_string(),
            "test prompt 1".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        task1.status = TaskStatus::Failed;
        task1.retry_count = 1;

        let mut task2 = Task::new(
            "/test".to_string(),
            "test prompt 2".to_string(),
            TaskPriority::High,
            vec![],
        );
        task2.status = TaskStatus::Failed;
        task2.retry_count = 3; // 达到最大重试次数

        let task3 = Task::new(
            "/test".to_string(),
            "test prompt 3".to_string(),
            TaskPriority::Medium,
            vec![],
        );

        repo.create_task(&task1).await.unwrap();
        repo.create_task(&task2).await.unwrap();
        repo.create_task(&task3).await.unwrap();

        let retried = repo.retry_failed_tasks(3).await.unwrap();

        assert_eq!(retried, 1); // 只有task1应该被重试

        // 验证task1被重试
        let tasks = repo.tasks.read().await;
        let task1_updated = tasks.values().find(|t| t.prompt == "test prompt 1").unwrap();
        assert_eq!(task1_updated.status, TaskStatus::Waiting);
        assert_eq!(task1_updated.retry_count, 2);
    }

    #[tokio::test]
    async fn test_simple_lock_manager_new() {
        let lock_manager = SimpleLockManager::new();
        assert_eq!(lock_manager.locks.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_try_acquire_lock() {
        let lock_manager = SimpleLockManager::new();
        
        let result = lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        assert!(result);

        let locks = lock_manager.locks.read().await;
        assert!(locks.contains_key("resource1"));
        assert_eq!(locks.get("resource1").unwrap().0, "owner1");
    }

    #[tokio::test]
    async fn test_try_acquire_already_locked() {
        let lock_manager = SimpleLockManager::new();
        
        // 第一次获取锁
        let result1 = lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        assert!(result1);

        // 第二次尝试获取锁
        let result2 = lock_manager.try_acquire("resource1", "owner2", 60).await.unwrap();
        assert!(!result2);
    }

    #[tokio::test]
    async fn test_try_acquire_same_owner() {
        let lock_manager = SimpleLockManager::new();
        
        // 第一次获取锁
        let result1 = lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        assert!(result1);

        // 同一个owner再次获取锁
        let result2 = lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        assert!(result2);
    }

    #[tokio::test]
    async fn test_release_lock() {
        let lock_manager = SimpleLockManager::new();
        
        lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        assert_eq!(lock_manager.locks.read().await.len(), 1);

        let result = lock_manager.release("resource1", "owner1").await.unwrap();
        assert!(result);
        assert_eq!(lock_manager.locks.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_release_lock_wrong_owner() {
        let lock_manager = SimpleLockManager::new();
        
        lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        
        let result = lock_manager.release("resource1", "owner2").await.unwrap();
        assert!(!result);
        assert_eq!(lock_manager.locks.read().await.len(), 1);
    }

    #[tokio::test]
    async fn test_check_lock() {
        let lock_manager = SimpleLockManager::new();
        
        lock_manager.try_acquire("resource1", "owner1", 60).await.unwrap();
        
        let owner = lock_manager.check_lock("resource1").await.unwrap();
        assert_eq!(owner, Some("owner1".to_string()));

        let owner2 = lock_manager.check_lock("resource2").await.unwrap();
        assert!(owner2.is_none());
    }

    #[tokio::test]
    async fn test_lock_expiration() {
        let lock_manager = SimpleLockManager::new();
        
        // 获取一个短期的锁
        lock_manager.try_acquire("resource1", "owner1", 1).await.unwrap();
        
        // 等待锁过期
        sleep(Duration::from_secs(2)).await;
        
        // 检查锁是否已过期
        let owner = lock_manager.check_lock("resource1").await.unwrap();
        assert!(owner.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired_locks() {
        let lock_manager = SimpleLockManager::new();
        
        lock_manager.try_acquire("resource1", "owner1", 1).await.unwrap();
        lock_manager.try_acquire("resource2", "owner2", 60).await.unwrap();
        
        // 等待第一个锁过期
        sleep(Duration::from_secs(2)).await;
        
        let cleaned = lock_manager.cleanup_expired_locks().await.unwrap();
        assert_eq!(cleaned, 1);
        
        let locks = lock_manager.locks.read().await;
        assert_eq!(locks.len(), 1);
        assert!(locks.contains_key("resource2"));
    }
}