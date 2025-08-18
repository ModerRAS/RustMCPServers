use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

use crate::domain::{TaskId, WorkerId};
use crate::infrastructure::LockManager;
use crate::errors::{AppError, AppResult};

/// 并发控制器
pub struct ConcurrencyController {
    task_locks: Arc<Mutex<HashMap<String, TaskLock>>>,
    semaphore: Arc<Semaphore>,
    lock_manager: Arc<dyn LockManager>,
    max_concurrent_tasks: usize,
    lock_timeout: Duration,
    cleanup_interval: Duration,
}

impl ConcurrencyController {
    /// 创建新的并发控制器
    pub fn new(
        lock_manager: Arc<dyn LockManager>,
        max_concurrent_tasks: usize,
        lock_timeout: Duration,
        cleanup_interval: Duration,
    ) -> Self {
        Self {
            task_locks: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            lock_manager,
            max_concurrent_tasks,
            lock_timeout,
            cleanup_interval,
        }
    }

    /// 尝试获取任务锁
    pub async fn acquire_task_lock(&self, task_id: &TaskId, worker_id: &WorkerId) -> AppResult<TaskLockHandle> {
        let task_key = task_id.to_string();
        let worker_key = worker_id.to_string();

        // 检查任务是否已被锁定
        {
            let locks = self.task_locks.lock().await;
            if let Some(existing_lock) = locks.get(&task_key) {
                if existing_lock.worker_id == worker_key {
                    return Ok(TaskLockHandle::new(task_id.clone(), worker_id.clone(), self.task_locks.clone()));
                } else {
                    return Err(AppError::TaskAlreadyAcquired);
                }
            }
        }

        // 尝试获取分布式锁
        let distributed_lock_acquired = self.lock_manager
            .try_acquire(&task_key, &worker_key, self.lock_timeout.as_secs() as u64)
            .await?;

        if !distributed_lock_acquired {
            return Err(AppError::TaskAlreadyAcquired);
        }

        // 获取信号量许可
        let permit = timeout(Duration::from_secs(5), self.semaphore.acquire())
            .await
            .map_err(|_| AppError::ServiceUnavailable("Semaphore acquisition timeout".to_string()))?
            .map_err(|_| AppError::ServiceUnavailable("Semaphore closed".to_string()))?;

        // 创建本地锁
        let task_lock = TaskLock {
            task_id: task_id.clone(),
            worker_id: worker_id.clone(),
            acquired_at: Instant::now(),
            expires_at: Instant::now() + self.lock_timeout,
            permit: Some(permit),
        };

        {
            let mut locks = self.task_locks.lock().await;
            locks.insert(task_key.clone(), task_lock);
        }

        Ok(TaskLockHandle::new(task_key, worker_key, self.task_locks.clone()))
    }

    /// 释放任务锁
    pub async fn release_task_lock(&self, task_id: &TaskId, worker_id: &WorkerId) -> AppResult<()> {
        let task_key = task_id.to_string();
        let worker_key = worker_id.to_string();

        // 检查锁是否存在并验证所有者
        let task_lock = {
            let mut locks = self.task_locks.lock().await;
            locks.remove(&task_key)
        };

        if let Some(lock) = task_lock {
            if lock.worker_id != worker_key {
                return Err(AppError::Authorization("Invalid lock owner".to_string()));
            }

            // 释放分布式锁
            let _ = self.lock_manager.release(&task_key, &worker_key).await;
        }

        Ok(())
    }

    /// 检查任务锁状态
    pub async fn check_task_lock(&self, task_id: &TaskId) -> AppResult<Option<WorkerId>> {
        let task_key = task_id.to_string();

        // 检查本地锁
        {
            let locks = self.task_locks.lock().await;
            if let Some(lock) = locks.get(&task_key) {
                if lock.expires_at > Instant::now() {
                    return Ok(Some(lock.worker_id.clone()));
                }
            }
        }

        // 检查分布式锁
        self.lock_manager.check_lock(&task_key).await
    }

    /// 清理过期锁
    pub async fn cleanup_expired_locks(&self) -> AppResult<u64> {
        let mut cleaned = 0;
        let now = Instant::now();

        {
            let mut locks = self.task_locks.lock().await;
            let expired_keys: Vec<String> = locks
                .iter()
                .filter(|(_, lock)| lock.expires_at <= now)
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                if let Some(lock) = locks.remove(&key) {
                    // 释放分布式锁
                    let _ = self.lock_manager.release(&key, &lock.worker_id).await;
                    cleaned += 1;
                }
            }
        }

        // 清理分布式过期锁
        let distributed_cleaned = self.lock_manager.cleanup_expired_locks().await?;
        cleaned += distributed_cleaned;

        Ok(cleaned)
    }

    /// 获取当前锁统计信息
    pub async fn get_lock_statistics(&self) -> LockStatistics {
        let locks = self.task_locks.lock().await;
        let now = Instant::now();

        let active_locks = locks.len();
        let expired_locks = locks.values().filter(|lock| lock.expires_at <= now).count();

        LockStatistics {
            total_locks: active_locks,
            expired_locks,
            max_concurrent_tasks: self.max_concurrent_tasks,
            available_permits: self.semaphore.available_permits(),
        }
    }

    /// 启动锁清理任务
    pub async fn start_cleanup_task(&self) -> AppResult<()> {
        let controller = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(controller.cleanup_interval);
            loop {
                interval.tick().await;
                if let Err(e) = controller.cleanup_expired_locks().await {
                    tracing::error!("Failed to cleanup expired locks: {}", e);
                }
            }
        });

        Ok(())
    }
}

impl Clone for ConcurrencyController {
    fn clone(&self) -> Self {
        Self {
            task_locks: self.task_locks.clone(),
            semaphore: self.semaphore.clone(),
            lock_manager: self.lock_manager.clone(),
            max_concurrent_tasks: self.max_concurrent_tasks,
            lock_timeout: self.lock_timeout,
            cleanup_interval: self.cleanup_interval,
        }
    }
}

/// 任务锁
#[derive(Debug, Clone)]
pub struct TaskLock {
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub acquired_at: Instant,
    pub expires_at: Instant,
    permit: Option<tokio::sync::SemaphorePermit<'static>>,
}

/// 任务锁句柄
#[derive(Debug)]
pub struct TaskLockHandle {
    task_id: String,
    worker_id: String,
    locks: Arc<Mutex<HashMap<String, TaskLock>>>,
}

impl TaskLockHandle {
    fn new(task_id: String, worker_id: String, locks: Arc<Mutex<HashMap<String, TaskLock>>>) -> Self {
        Self {
            task_id,
            worker_id,
            locks,
        }
    }

    /// 延长锁的有效期
    pub async fn renew(&self, additional_duration: Duration) -> AppResult<()> {
        let mut locks = self.locks.lock().await;
        if let Some(lock) = locks.get_mut(&self.task_id) {
            if lock.worker_id.to_string() == self.worker_id {
                lock.expires_at = lock.expires_at + additional_duration;
                return Ok(());
            }
        }
        Err(AppError::Authorization("Invalid lock owner".to_string()))
    }

    /// 检查锁是否仍然有效
    pub async fn is_valid(&self) -> bool {
        let locks = self.locks.lock().await;
        if let Some(lock) = locks.get(&self.task_id) {
            lock.worker_id.to_string() == self.worker_id && lock.expires_at > Instant::now()
        } else {
            false
        }
    }
}

impl Drop for TaskLockHandle {
    fn drop(&mut self) {
        let task_id = self.task_id.clone();
        let worker_id = self.worker_id.clone();
        let locks = self.locks.clone();

        // 在drop时异步释放锁
        tokio::spawn(async move {
            let mut locks = locks.lock().await;
            if let Some(lock) = locks.remove(&task_id) {
                if lock.worker_id.to_string() == worker_id {
                    tracing::debug!("Released task lock for task: {}", task_id);
                }
            }
        });
    }
}

/// 锁统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct LockStatistics {
    pub total_locks: usize,
    pub expired_locks: usize,
    pub max_concurrent_tasks: usize,
    pub available_permits: usize,
}

/// 速率限制器
pub struct RateLimiter {
    requests_per_minute: u32,
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    cleanup_interval: Duration,
}

impl RateLimiter {
    /// 创建新的速率限制器
    pub fn new(requests_per_minute: u32, cleanup_interval: Duration) -> Self {
        Self {
            requests_per_minute,
            requests: Arc::new(Mutex::new(HashMap::new())),
            cleanup_interval,
        }
    }

    /// 检查是否允许请求
    pub async fn check_rate_limit(&self, key: &str) -> AppResult<()> {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(60);

        let mut requests = self.requests.lock().await;
        let user_requests = requests.entry(key.to_string()).or_insert_with(Vec::new);

        // 清理过期的请求记录
        user_requests.retain(|&timestamp| timestamp > window_start);

        // 检查是否超过限制
        if user_requests.len() >= self.requests_per_minute as usize {
            return Err(AppError::RateLimitExceeded);
        }

        // 记录新请求
        user_requests.push(now);

        Ok(())
    }

    /// 获取当前速率限制状态
    pub async fn get_rate_limit_status(&self, key: &str) -> RateLimitStatus {
        let now = Instant::now();
        let window_start = now - Duration::from_secs(60);

        let requests = self.requests.lock().await;
        let user_requests = requests.get(key).map(|v| v.as_slice()).unwrap_or(&[]);

        let current_requests = user_requests.iter()
            .filter(|&timestamp| *timestamp > window_start)
            .count();

        let remaining_requests = self.requests_per_minute.saturating_sub(current_requests as u32);
        let reset_time = user_requests.first()
            .map(|&first| first + Duration::from_secs(60))
            .unwrap_or(now + Duration::from_secs(60));

        RateLimitStatus {
            limit: self.requests_per_minute,
            remaining: remaining_requests,
            reset: reset_time,
        }
    }

    /// 启动清理任务
    pub async fn start_cleanup_task(&self) -> AppResult<()> {
        let limiter = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(limiter.cleanup_interval);
            loop {
                interval.tick().await;
                
                let now = Instant::now();
                let window_start = now - Duration::from_secs(60);
                
                let mut requests = limiter.requests.lock().await;
                requests.retain(|_, user_requests| {
                    user_requests.retain(|&timestamp| timestamp > window_start);
                    !user_requests.is_empty()
                });
            }
        });

        Ok(())
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            requests_per_minute: self.requests_per_minute,
            requests: self.requests.clone(),
            cleanup_interval: self.cleanup_interval,
        }
    }
}

/// 速率限制状态
#[derive(Debug, Clone, serde::Serialize)]
pub struct RateLimitStatus {
    pub limit: u32,
    pub remaining: u32,
    pub reset: Instant,
}

/// 熔断器
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: Arc<Mutex<CircuitBreakerState>>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            state: Arc::new(Mutex::new(CircuitBreakerState::Closed {
                failures: 0,
                last_failure_time: None,
            })),
        }
    }

    /// 执行受保护的操作
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        let state = self.state.lock().await;
        
        match *state {
            CircuitBreakerState::Open { opened_at } => {
                if opened_at.elapsed() > self.recovery_timeout {
                    drop(state);
                    self.try_half_open(operation).await
                } else {
                    drop(state);
                    Err(self.create_circuit_breaker_error())
                }
            }
            CircuitBreakerState::HalfOpen => {
                drop(state);
                self.try_half_open(operation).await
            }
            CircuitBreakerState::Closed { .. } => {
                drop(state);
                self.try_closed(operation).await
            }
        }
    }

    async fn try_closed<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        match operation.await {
            Ok(result) => {
                let mut state = self.state.lock().await;
                if let CircuitBreakerState::Closed { failures, .. } = *state {
                    *state = CircuitBreakerState::Closed {
                        failures: 0,
                        last_failure_time: None,
                    };
                }
                Ok(result)
            }
            Err(error) => {
                let mut state = self.state.lock().await;
                match *state {
                    CircuitBreakerState::Closed { failures, last_failure_time } => {
                        let new_failures = failures + 1;
                        if new_failures >= self.failure_threshold {
                            *state = CircuitBreakerState::Open {
                                opened_at: Instant::now(),
                            };
                            tracing::warn!("Circuit breaker opened after {} failures", new_failures);
                        } else {
                            *state = CircuitBreakerState::Closed {
                                failures: new_failures,
                                last_failure_time: Some(Instant::now()),
                            };
                        }
                    }
                    _ => {}
                }
                Err(error)
            }
        }
    }

    async fn try_half_open<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Debug,
    {
        match operation.await {
            Ok(result) => {
                let mut state = self.state.lock().await;
                *state = CircuitBreakerState::Closed {
                    failures: 0,
                    last_failure_time: None,
                };
                tracing::info!("Circuit breaker closed after successful request");
                Ok(result)
            }
            Err(error) => {
                let mut state = self.state.lock().await;
                *state = CircuitBreakerState::Open {
                    opened_at: Instant::now(),
                };
                tracing::warn!("Circuit breaker reopened after failed request in half-open state");
                Err(error)
            }
        }
    }

    fn create_circuit_breaker_error<T>(&self) -> T {
        // 这里需要根据实际需求创建适当的错误类型
        // 由于是泛型，这里只是一个占位符
        panic!("Circuit breaker is open");
    }

    /// 获取熔断器状态
    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.lock().await.clone()
    }

    /// 手动重置熔断器
    pub async fn reset(&self) {
        let mut state = self.state.lock().await;
        *state = CircuitBreakerState::Closed {
            failures: 0,
            last_failure_time: None,
        };
    }
}

/// 熔断器状态
#[derive(Debug, Clone, serde::Serialize)]
pub enum CircuitBreakerState {
    Closed {
        failures: u32,
        last_failure_time: Option<Instant>,
    },
    Open {
        opened_at: Instant,
    },
    HalfOpen,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::LockManager;

    // Mock lock manager for testing
    struct MockLockManager;

    #[async_trait::async_trait]
    impl LockManager for MockLockManager {
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
    async fn test_concurrency_controller() {
        let lock_manager = Arc::new(MockLockManager);
        let controller = ConcurrencyController::new(
            lock_manager,
            10,
            Duration::from_secs(30),
            Duration::from_secs(60),
        );

        let task_id = TaskId::new();
        let worker_id = WorkerId::new("worker-1".to_string()).unwrap();

        // 获取锁
        let lock_handle = controller.acquire_task_lock(&task_id, &worker_id).await.unwrap();
        assert!(lock_handle.is_valid().await);

        // 检查锁状态
        let lock_status = controller.check_task_lock(&task_id).await.unwrap();
        assert_eq!(lock_status, Some(worker_id.clone()));

        // 释放锁
        controller.release_task_lock(&task_id, &worker_id).await.unwrap();

        // 检查锁是否已释放
        let lock_status = controller.check_task_lock(&task_id).await.unwrap();
        assert!(lock_status.is_none());
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        let key = "test-user";

        // 前5个请求应该成功
        for i in 0..5 {
            let result = limiter.check_rate_limit(key).await;
            assert!(result.is_ok(), "Request {} should succeed", i + 1);
        }

        // 第6个请求应该被限制
        let result = limiter.check_rate_limit(key).await;
        assert!(result.is_err(), "6th request should be rate limited");

        // 检查速率限制状态
        let status = limiter.get_rate_limit_status(key).await;
        assert_eq!(status.limit, 5);
        assert_eq!(status.remaining, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, Duration::from_secs(5));

        // 初始状态应该是关闭的
        let state = breaker.get_state().await;
        assert!(matches!(state, CircuitBreakerState::Closed { failures: 0, .. }));

        // 模拟失败
        for i in 0..3 {
            let result = breaker.call(async { Err::<(), &str>("error") }).await;
            assert!(result.is_err());
        }

        // 应该打开熔断器
        let state = breaker.get_state().await;
        assert!(matches!(state, CircuitBreakerState::Open { .. }));

        // 等待恢复超时
        tokio::time::sleep(Duration::from_secs(6)).await;

        // 尝试成功请求
        let result = breaker.call(async { Ok::<(), &str>(()) }).await;
        assert!(result.is_ok());

        // 应该关闭熔断器
        let state = breaker.get_state().await;
        assert!(matches!(state, CircuitBreakerState::Closed { failures: 0, .. }));
    }
}