use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use simple_task_orchestrator::*;
use tokio::runtime::Runtime;

fn benchmark_task_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    let mut group = c.benchmark_group("task_creation");

    for size in [1, 10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::new("create_tasks", size), size, |b, &size| {
            b.to_async(&rt).iter(|| async {
                let mut tasks = Vec::new();
                for i in 0..size {
                    let service = task_service.clone();
                    let task = async move {
                        let request = CreateTaskRequest {
                            work_directory: "/test".to_string(),
                            prompt: format!("Benchmark task {}", i),
                            priority: Some(TaskPriority::Medium),
                            tags: None,
                        };
                        service.create_task(request).await
                    };
                    tasks.push(task);
                }
                
                futures::future::join_all(tasks).await
            });
        });
    }
    
    group.finish();
}

fn benchmark_task_acquisition(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    // 预创建任务
    rt.block_on(async {
        for i in 0..1000 {
            let request = CreateTaskRequest {
                work_directory: "/test".to_string(),
                prompt: format!("Pre-created task {}", i),
                priority: Some(TaskPriority::High),
                tags: None,
            };
            task_service.create_task(request).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("task_acquisition");

    for workers in [1, 5, 10, 20].iter() {
        group.bench_with_input(BenchmarkId::new("acquire_tasks", workers), workers, |b, &workers| {
            b.to_async(&rt).iter(|| async {
                let mut tasks = Vec::new();
                for i in 0..workers {
                    let service = task_service.clone();
                    let task = async move {
                        let request = AcquireTaskRequest {
                            work_path: "/test".to_string(),
                            worker_id: format!("worker_{}", i),
                        };
                        service.acquire_task(request).await
                    };
                    tasks.push(task);
                }
                
                futures::future::join_all(tasks).await
            });
        });
    }
    
    group.finish();
}

fn benchmark_task_listing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let task_repository = Arc::new(InMemoryTaskRepository::new());
    let lock_manager = Arc::new(SimpleLockManager::new());
    let task_service = Arc::new(TaskService::new(
        task_repository,
        lock_manager,
        3,
        3600,
    ));

    // 预创建不同数量的任务
    let task_counts = [100, 1000, 5000];
    
    for &count in &task_counts {
        rt.block_on(async {
            for i in 0..count {
                let request = CreateTaskRequest {
                    work_directory: "/test".to_string(),
                    prompt: format!("Listable task {}", i),
                    priority: Some(if i % 3 == 0 { TaskPriority::High } else { TaskPriority::Medium }),
                    tags: Some(vec!["benchmark".to_string()]),
                };
                task_service.create_task(request).await.unwrap();
            }
        });

        let mut group = c.benchmark_group("task_listing");
        
        group.bench_with_input(BenchmarkId::new("list_all_tasks", count), &count, |b, _| {
            b.to_async(&rt).iter(|| async {
                let filter = TaskFilter::new();
                task_service.list_tasks(filter).await
            });
        });

        group.bench_with_input(BenchmarkId::new("list_with_limit", count), &count, |b, _| {
            b.to_async(&rt).iter(|| async {
                let filter = TaskFilter::new().with_limit(100);
                task_service.list_tasks(filter).await
            });
        });

        group.finish();
    }
}

fn benchmark_lock_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let lock_manager = Arc::new(SimpleLockManager::new());

    let mut group = c.benchmark_group("lock_operations");

    group.bench_function("acquire_lock", |b| {
        b.to_async(&rt).iter(|| async {
            let resource_id = uuid::Uuid::new_v4().to_string();
            let owner_id = uuid::Uuid::new_v4().to_string();
            lock_manager.try_acquire(&resource_id, &owner_id, 60).await
        });
    });

    group.bench_function("release_lock", |b| {
        b.to_async(&rt).iter(|| async {
            let resource_id = uuid::Uuid::new_v4().to_string();
            let owner_id = uuid::Uuid::new_v4().to_string();
            
            // 先获取锁
            lock_manager.try_acquire(&resource_id, &owner_id, 60).await.unwrap();
            
            // 再释放锁
            lock_manager.release(&resource_id, &owner_id).await
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_task_creation,
    benchmark_task_acquisition,
    benchmark_task_listing,
    benchmark_lock_operations
);
criterion_main!(benches);