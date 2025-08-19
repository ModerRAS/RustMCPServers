use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use async_trait::async_trait;

use crate::domain::{
    Task, TaskId, TaskStatus, 
    TaskFilter, TaskStatistics,
};

/// 任务仓库特征
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create_task(&self, task: &Task) -> Result<TaskId, String>;
    async fn get_task(&self, task_id: &TaskId) -> Result<Option<Task>, String>;
    async fn update_task(&self, task: &Task) -> Result<(), String>;
    async fn delete_task(&self, task_id: &TaskId) -> Result<(), String>;
    async fn get_next_task(&self, work_directory: &str, worker_id: &str) -> Result<Option<Task>, String>;
    async fn list_tasks(&self, filter: &TaskFilter) -> Result<(Vec<Task>, u64), String>;
    async fn get_tasks_by_work_directory(&self, work_directory: &str) -> Result<Vec<Task>, String>;
    async fn get_all_tasks(&self) -> Result<Vec<Task>, String>;
    async fn get_statistics(&self) -> Result<TaskStatistics, String>;
    async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> Result<u64, String>;
    async fn retry_failed_tasks(&self, max_retries: u32) -> Result<u64, String>;
}

/// 内存任务仓库实现
pub struct InMemoryTaskRepository {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    worker_tasks: Arc<RwLock<HashMap<String, Vec<TaskId>>>>,
}

impl InMemoryTaskRepository {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            worker_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn find_next_task(&self, work_directory: &str, worker_id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        let worker_tasks = self.worker_tasks.read().await;
        
        // 找出指定工作目录的等待任务
        let mut candidates: Vec<_> = tasks.iter()
            .filter(|(_, task)| {
                task.work_directory == work_directory 
                && task.status == TaskStatus::Waiting
                && !task.is_expired(3600) // 1小时超时
            })
            .collect();
        
        // 按优先级排序
        candidates.sort_by(|a, b| {
            b.1.priority.cmp(&a.1.priority) // 高优先级在前
        });
        
        // 检查是否有任务已经被分配给当前worker
        if let Some(worker_task_ids) = worker_tasks.get(worker_id) {
            for task_id in worker_task_ids {
                if let Some(task) = tasks.get(task_id) {
                    if task.status == TaskStatus::Waiting {
                        return Some(task.clone());
                    }
                }
            }
        }
        
        // 返回最高优先级的任务
        candidates.first().map(|(_, task)| task.clone()).cloned()
    }
}

#[async_trait]
impl TaskRepository for InMemoryTaskRepository {
    async fn create_task(&self, task: &Task) -> Result<TaskId, String> {
        let mut tasks = self.tasks.write().await;
        let task_id = task.id.clone();
        tasks.insert(task_id.clone(), task.clone());
        Ok(task_id)
    }
    
    async fn get_task(&self, task_id: &TaskId) -> Result<Option<Task>, String> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(task_id).cloned())
    }
    
    async fn update_task(&self, task: &Task) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        Ok(())
    }
    
    async fn delete_task(&self, task_id: &TaskId) -> Result<(), String> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(task_id);
        Ok(())
    }
    
    async fn get_next_task(&self, work_directory: &str, worker_id: &str) -> Result<Option<Task>, String> {
        Ok(self.find_next_task(work_directory, worker_id).await)
    }
    
    async fn list_tasks(&self, filter: &TaskFilter) -> Result<(Vec<Task>, u64), String> {
        let tasks = self.tasks.read().await;
        let mut filtered: Vec<Task> = tasks.values().cloned().collect();
        
        // 应用过滤器
        if let Some(status) = &filter.status {
            filtered.retain(|task| task.status == *status);
        }
        
        if let Some(priority) = &filter.priority {
            filtered.retain(|task| task.priority == *priority);
        }
        
        if let Some(worker_id) = &filter.worker_id {
            filtered.retain(|task| {
                task.worker_id.as_ref().map(|w| w.as_str() == worker_id.as_str()).unwrap_or(false)
            });
        }
        
        // 按创建时间排序
        filtered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        let total = filtered.len() as u64;
        
        // 分页
        if let Some(limit) = filter.limit {
            let offset = filter.offset.unwrap_or(0) as usize;
            let end = std::cmp::min(offset + limit as usize, filtered.len());
            filtered = filtered[offset..end].to_vec();
        }
        
        Ok((filtered, total))
    }
    
    async fn get_tasks_by_work_directory(&self, work_directory: &str) -> Result<Vec<Task>, String> {
        let tasks = self.tasks.read().await;
        let filtered: Vec<Task> = tasks.values()
            .filter(|task| task.work_directory == work_directory)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
    
    async fn get_all_tasks(&self) -> Result<Vec<Task>, String> {
        let tasks = self.tasks.read().await;
        let all_tasks: Vec<Task> = tasks.values().cloned().collect();
        Ok(all_tasks)
    }
    
    async fn get_statistics(&self) -> Result<TaskStatistics, String> {
        let tasks = self.tasks.read().await;
        let mut stats = TaskStatistics::new();
        
        for task in tasks.values() {
            stats.total_tasks += 1;
            
            match task.status {
                TaskStatus::Completed => stats.completed_tasks += 1,
                TaskStatus::Failed => stats.failed_tasks += 1,
                TaskStatus::Cancelled => stats.cancelled_tasks += 1,
                TaskStatus::Waiting | TaskStatus::Working => stats.active_tasks += 1,
            }
        }
        
        if stats.total_tasks > 0 {
            stats.success_rate = stats.completed_tasks as f64 / stats.total_tasks as f64;
        }
        
        Ok(stats)
    }
    
    async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> Result<u64, String> {
        let mut tasks = self.tasks.write().await;
        let initial_count = tasks.len();
        
        tasks.retain(|_, task| {
            task.created_at > older_than && !task.status.is_terminal()
        });
        
        Ok((initial_count - tasks.len()) as u64)
    }
    
    async fn retry_failed_tasks(&self, max_retries: u32) -> Result<u64, String> {
        let mut tasks = self.tasks.write().await;
        let mut retried = 0;
        
        for task in tasks.values_mut() {
            if task.status == TaskStatus::Failed && task.retry_count < max_retries {
                if let Err(_) = task.retry() {
                    continue;
                }
                retried += 1;
            }
        }
        
        Ok(retried)
    }
}

/// 简单的锁管理器
pub struct SimpleLockManager {
    locks: Arc<RwLock<HashMap<String, (String, DateTime<Utc>)>>>,
}

impl SimpleLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn cleanup_expired_locks(&self) {
        let mut locks = self.locks.write().await;
        let now = Utc::now();
        locks.retain(|_, (_, expires_at)| *expires_at > now);
    }
}

#[async_trait]
pub trait LockManager: Send + Sync {
    async fn try_acquire(&self, resource_id: &str, owner_id: &str, ttl_seconds: u64) -> Result<bool, String>;
    async fn release(&self, resource_id: &str, owner_id: &str) -> Result<bool, String>;
    async fn check_lock(&self, resource_id: &str) -> Result<Option<String>, String>;
    async fn cleanup_expired_locks(&self) -> Result<u64, String>;
}

#[async_trait]
impl LockManager for SimpleLockManager {
    async fn try_acquire(&self, resource_id: &str, owner_id: &str, ttl_seconds: u64) -> Result<bool, String> {
        self.cleanup_expired_locks().await;
        
        let mut locks = self.locks.write().await;
        let expires_at = Utc::now() + chrono::Duration::seconds(ttl_seconds as i64);
        
        if let Some((current_owner, _)) = locks.get(resource_id) {
            if current_owner == owner_id {
                // 已经是锁的所有者，更新过期时间
                locks.insert(resource_id.to_string(), (owner_id.to_string(), expires_at));
                return Ok(true);
            }
            return Ok(false);
        }
        
        locks.insert(resource_id.to_string(), (owner_id.to_string(), expires_at));
        Ok(true)
    }
    
    async fn release(&self, resource_id: &str, owner_id: &str) -> Result<bool, String> {
        let mut locks = self.locks.write().await;
        
        if let Some((current_owner, _)) = locks.get(resource_id) {
            if current_owner == owner_id {
                locks.remove(resource_id);
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    async fn check_lock(&self, resource_id: &str) -> Result<Option<String>, String> {
        self.cleanup_expired_locks().await;
        
        let locks = self.locks.read().await;
        Ok(locks.get(resource_id).map(|(owner, _)| owner.clone()))
    }
    
    async fn cleanup_expired_locks(&self) -> Result<u64, String> {
        let mut locks = self.locks.write().await;
        let now = Utc::now();
        let initial_count = locks.len();
        
        locks.retain(|_, (_, expires_at)| *expires_at > now);
        
        Ok((initial_count - locks.len()) as u64)
    }
}