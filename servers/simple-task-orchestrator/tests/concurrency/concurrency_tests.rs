use simple_task_orchestrator::*;
use std::sync::Arc;
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_concurrent_task_creation() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    let mut tasks = Vec::new();
    let num_tasks = 10;

    // 并发创建任务
    for i in 0..num_tasks {
        let service = task_service.clone();
        let task = tokio::spawn(async move {
            let request = CreateTaskRequest {
                work_directory: format!("/test{}", i % 3), // 分散到不同目录
                prompt: format!("Concurrent task {}", i),
                priority: Some(if i % 3 == 0 { TaskPriority::High } else { TaskPriority::Medium }),
                tags: Some(vec!["concurrent".to_string()]),
            };
            
            service.create_task(request).await
        });
        tasks.push(task);
    }

    // 等待所有任务完成
    let mut results = Vec::new();
    for task in tasks {
        let result = task.await.unwrap();
        results.push(result);
    }

    // 验证所有任务都创建成功
    assert_eq!(results.len(), num_tasks);
    for result in results {
        assert!(result.is_ok());
    }

    // 验证任务确实被创建
    let filter = TaskFilter::new();
    let (created_tasks, total) = task_service.list_tasks(filter).await.unwrap();
    assert_eq!(total, num_tasks as u64);
    assert_eq!(created_tasks.len(), num_tasks);
}

#[tokio::test]
async fn test_concurrent_task_acquisition() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    // 创建一些任务
    let num_tasks = 5;
    for i in 0..num_tasks {
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: format!("Task for acquisition {}", i),
            priority: Some(TaskPriority::High),
            tags: None,
        };
        task_service.create_task(request).await.unwrap();
    }

    let mut acquire_tasks = Vec::new();
    let num_workers = 10;

    // 并发获取任务
    for i in 0..num_workers {
        let service = task_service.clone();
        let task = tokio::spawn(async move {
            let request = AcquireTaskRequest {
                work_path: "/test".to_string(),
                worker_id: format!("worker_{}", i),
            };
            
            service.acquire_task(request).await
        });
        acquire_tasks.push(task);
    }

    // 等待所有获取操作完成
    let mut results = Vec::new();
    for task in acquire_tasks {
        let result = task.await.unwrap();
        results.push(result);
    }

    // 统计成功获取的任务数量
    let successful_acquisitions = results.iter().filter(|r| r.as_ref().map(|t| t.is_some()).unwrap_or(false)).count();
    let failed_acquisitions = results.iter().filter(|r| r.as_ref().map(|t| t.is_none()).unwrap_or(false)).count();
    let error_acquisitions = results.iter().filter(|r| r.is_err()).count();

    // 由于并发竞争，应该有一些成功和一些失败
    assert!(successful_acquisitions > 0);
    assert!(successful_acquisitions <= num_tasks);
    assert_eq!(error_acquisitions, 0); // 不应该有错误，只有None结果
    assert_eq!(successful_acquisitions + failed_acquisitions, num_workers);
}

#[tokio::test]
async fn test_lock_manager_concurrent_access() {
    let lock_manager = Arc::new(SimpleLockManager::new());
    let resource_id = "test_resource";
    let num_threads = 10;

    let mut tasks = Vec::new();

    // 并发尝试获取同一个锁
    for i in 0..num_threads {
        let lock_manager = lock_manager.clone();
        let resource_id = resource_id.to_string();
        let owner_id = format!("owner_{}", i);
        
        let task = tokio::spawn(async move {
            lock_manager.try_acquire(&resource_id, &owner_id, 10).await
        });
        tasks.push(task);
    }

    // 等待所有任务完成
    let results = futures::future::join_all(tasks).await;
    let lock_results: Vec<bool> = results.into_iter().map(|r| r.unwrap()).collect();

    // 只有一个应该成功获取锁
    let successful_locks = lock_results.iter().filter(|&&r| r).count();
    assert_eq!(successful_locks, 1);

    // 获取锁的owner
    let lock_owner = lock_manager.check_lock(resource_id).await.unwrap();
    assert!(lock_owner.is_some());
}

#[tokio::test]
async fn test_lock_manager_concurrent_different_resources() {
    let lock_manager = Arc::new(SimpleLockManager::new());
    let num_resources = 5;
    let num_threads_per_resource = 3;

    let mut tasks = Vec::new();

    // 并发获取不同资源的锁
    for resource_i in 0..num_resources {
        for thread_j in 0..num_threads_per_resource {
            let lock_manager = lock_manager.clone();
            let resource_id = format!("resource_{}", resource_i);
            let owner_id = format!("owner_{}_{}", resource_i, thread_j);
            
            let task = tokio::spawn(async move {
                lock_manager.try_acquire(&resource_id, &owner_id, 10).await
            });
            tasks.push(task);
        }
    }

    // 等待所有任务完成
    let results = futures::future::join_all(tasks).await;
    let lock_results: Vec<bool> = results.into_iter().map(|r| r.unwrap()).collect();

    // 每个资源应该有一个锁获取成功
    let successful_locks = lock_results.iter().filter(|&&r| r).count();
    assert_eq!(successful_locks, num_resources);
}

#[tokio::test]
async fn test_concurrent_task_completion() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    // 创建并获取一些任务
    let mut task_ids = Vec::new();
    for i in 0..5 {
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: format!("Task for completion {}", i),
            priority: Some(TaskPriority::Medium),
            tags: None,
        };
        
        let task = task_service.create_task(request).await.unwrap();
        let acquire_request = AcquireTaskRequest {
            work_path: "/test".to_string(),
            worker_id: format!("worker_{}", i),
        };
        
        let acquired_task = task_service.acquire_task(acquire_request).await.unwrap().unwrap();
        task_ids.push(acquired_task.id);
    }

    let mut complete_tasks = Vec::new();

    // 并发完成任务
    for task_id in task_ids {
        let service = task_service.clone();
        let task = tokio::spawn(async move {
            let request = CompleteTaskRequest {
                original_prompt: None,
                result: Some(TaskResult::success("Task completed".to_string())),
            };
            
            service.complete_task(&task_id, request).await
        });
        complete_tasks.push(task);
    }

    // 等待所有完成操作完成
    let results = futures::future::join_all(complete_tasks).await;
    let completion_results: Vec<Result<Task, String>> = results.into_iter().map(|r| r.unwrap()).collect();

    // 验证所有任务都成功完成
    assert_eq!(completion_results.len(), 5);
    for result in completion_results {
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.status, TaskStatus::Completed);
    }
}

#[tokio::test]
async fn test_concurrent_task_listing() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
    3600,
    ));

    // 创建一些任务
    let num_tasks = 20;
    for i in 0..num_tasks {
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: format!("Task for listing {}", i),
            priority: Some(if i % 4 == 0 { TaskPriority::High } else { TaskPriority::Medium }),
            tags: Some(vec!["listing".to_string()]),
        };
        task_service.create_task(request).await.unwrap();
    }

    let mut list_tasks = Vec::new();
    let num_concurrent_lists = 10;

    // 并发列出任务
    for i in 0..num_concurrent_lists {
        let service = task_service.clone();
        let task = tokio::spawn(async move {
            let filter = TaskFilter::new()
                .with_limit(10)
                .with_offset(i * 2);
            
            service.list_tasks(filter).await
        });
        list_tasks.push(task);
    }

    // 等待所有列出操作完成
    let results = futures::future::join_all(list_tasks).await;
    let list_results: Vec<Result<(Vec<Task>, u64), String>> = results.into_iter().map(|r| r.unwrap()).collect();

    // 验证所有列出操作都成功
    assert_eq!(list_results.len(), num_concurrent_lists);
    for result in list_results {
        assert!(result.is_ok());
        let (tasks, total) = result.unwrap();
        assert!(tasks.len() <= 10);
        assert_eq!(total, num_tasks as u64);
    }
}

#[tokio::test]
async fn test_concurrent_statistics() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    let mut stats_tasks = Vec::new();
    let num_concurrent_stats = 20;

    // 并发获取统计信息
    for _ in 0..num_concurrent_stats {
        let service = task_service.clone();
        let task = tokio::spawn(async move {
            service.get_statistics().await
        });
        stats_tasks.push(task);
    }

    // 等待所有统计操作完成
    let results = futures::future::join_all(stats_tasks).await;
    let stats_results: Vec<Result<TaskStatistics, String>> = results.into_iter().map(|r| r.unwrap()).collect();

    // 验证所有统计操作都成功
    assert_eq!(stats_results.len(), num_concurrent_stats);
    for result in stats_results {
        assert!(result.is_ok());
        let stats = result.unwrap();
        // 初始状态下应该没有任务
        assert_eq!(stats.total_tasks, 0);
    }
}

#[tokio::test]
async fn test_concurrent_task_creation_and_listing() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    let mut tasks = Vec::new();
    let num_operations = 15;

    // 混合并发操作：创建任务和列出任务
    for i in 0..num_operations {
        let service = task_service.clone();
        
        if i % 3 == 0 {
            // 创建任务
            let task = tokio::spawn(async move {
                let request = CreateTaskRequest {
                    work_directory: "/test".to_string(),
                    prompt: format!("Mixed operation task {}", i),
                    priority: Some(TaskPriority::Medium),
                    tags: None,
                };
                
                service.create_task(request).await
            });
            tasks.push(task);
        } else {
            // 列出任务
            let task = tokio::spawn(async move {
                let filter = TaskFilter::new();
                service.list_tasks(filter).await
            });
            tasks.push(task);
        }
    }

    // 等待所有操作完成
    let results = futures::future::join_all(tasks).await;

    // 验证所有操作都成功
    let mut creation_count = 0;
    let mut listing_count = 0;

    for result in results {
        assert!(result.is_ok());
        let operation_result = result.unwrap();
        assert!(operation_result.is_ok());
        
        // 根据操作类型计数
        if operation_result.as_ref().unwrap().0.len() > 0 || operation_result.as_ref().unwrap().1 > 0 {
            listing_count += 1;
        } else {
            creation_count += 1;
        }
    }

    assert_eq!(creation_count + listing_count, num_operations);
    assert!(creation_count > 0);
    assert!(listing_count > 0);
}

#[tokio::test]
async fn test_lock_contention_resolution() {
    let lock_manager = Arc::new(SimpleLockManager::new());
    let resource_id = "contended_resource";

    // 第一个线程获取锁
    let lock_manager1 = lock_manager.clone();
    let resource_id1 = resource_id.to_string();
    let first_lock = tokio::spawn(async move {
        lock_manager1.try_acquire(&resource_id1, "owner1", 2).await
    });

    // 等待第一个锁获取成功
    assert!(first_lock.await.unwrap());

    // 其他线程尝试获取锁（应该失败）
    let mut contention_tasks = Vec::new();
    for i in 2..6 {
        let lock_manager = lock_manager.clone();
        let resource_id = resource_id.to_string();
        let owner_id = format!("owner{}", i);
        
        let task = tokio::spawn(async move {
            lock_manager.try_acquire(&resource_id, &owner_id, 1).await
        });
        contention_tasks.push(task);
    }

    // 等待所有竞争尝试完成
    let results = futures::future::join_all(contention_tasks).await;
    let contention_results: Vec<bool> = results.into_iter().map(|r| r.unwrap()).collect();

    // 所有竞争尝试都应该失败
    assert!(contention_results.iter().all(|&r| !r));

    // 等待锁过期
    sleep(Duration::from_secs(3)).await;

    // 现在应该可以获取锁
    let new_lock_result = lock_manager.try_acquire(resource_id, "new_owner", 5).await;
    assert!(new_lock_result);
}

#[tokio::test]
async fn test_concurrent_task_with_same_worker() {
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    // 创建多个任务
    let num_tasks = 8;
    for i in 0..num_tasks {
        let request = CreateTaskRequest {
            work_directory: "/test".to_string(),
            prompt: format!("Task for worker {}", i),
            priority: Some(TaskPriority::High),
            tags: None,
        };
        task_service.create_task(request).await.unwrap();
    }

    let worker_id = "same_worker".to_string();
    let mut acquire_tasks = Vec::new();

    // 同一个worker并发获取任务
    for _ in 0..num_tasks {
        let service = task_service.clone();
        let worker_id = worker_id.clone();
        let task = tokio::spawn(async move {
            let request = AcquireTaskRequest {
                work_path: "/test".to_string(),
                worker_id,
            };
            
            service.acquire_task(request).await
        });
        acquire_tasks.push(task);
    }

    // 等待所有获取操作完成
    let results = futures::future::join_all(acquire_tasks).await;
    let acquire_results: Vec<Result<Option<Task>, String>> = results.into_iter().map(|r| r.unwrap()).collect();

    // 统计结果
    let successful_acquisitions = acquire_results.iter().filter(|r| r.as_ref().map(|t| t.is_some()).unwrap_or(false)).count();
    let none_results = acquire_results.iter().filter(|r| r.as_ref().map(|t| t.is_none()).unwrap_or(false)).count();

    // 由于是同一个worker，应该只有一个任务被获取（worker_tasks机制）
    assert_eq!(successful_acquisitions, 1);
    assert_eq!(none_results, num_tasks - 1);
}