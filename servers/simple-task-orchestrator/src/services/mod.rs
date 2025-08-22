//! # 服务层模块
//! 
//! 该模块提供了 Simple Task Orchestrator 的核心业务逻辑和服务实现。
//! 
//! ## 主要功能
//! 
//! - **任务管理**: 完整的任务生命周期管理服务
//! - **任务调度**: 自动化的任务调度和监控
//! - **并发控制**: 基于锁的任务并发执行控制
//! - **错误处理**: 健壮的错误处理和重试机制
//! - **后台任务**: 自动清理和超时处理
//! 
//! ## 核心服务
//! 
//! - `TaskService`: 核心任务服务，提供任务CRUD操作
//! - `TaskScheduler`: 任务调度器，处理定时任务和清理
//! - `TaskMonitor`: 任务监控器，收集和显示统计信息
//! - `TaskExecutionService`: 任务执行服务，处理不同执行模式
//! 
//! ## 使用示例
//! 
//! ```rust
//! use simple_task_orchestrator::services::{TaskService, TaskScheduler, TaskMonitor};
//! use simple_task_orchestrator::infrastructure::{TaskRepository, LockManager};
//! use std::sync::Arc;
//! 
//! // 创建任务服务
//! let task_service = Arc::new(TaskService::new(
//!     task_repository,
//!     lock_manager,
//!     3,  // max_retries
//!     3600,  // task_timeout
//! ));
//! 
//! // 创建调度器和监控器
//! let scheduler = TaskScheduler::new(task_service.clone(), 3600, 60);
//! let monitor = TaskMonitor::new(task_service.clone(), 60);
//! 
//! // 启动后台任务
//! scheduler.start().await?;
//! monitor.start().await?;
//! 
//! // 创建任务
//! let task = task_service.create_task(create_request).await?;
//! 
//! // 获取任务
//! let task = task_service.acquire_task(acquire_request).await?;
//! ```
//! 
//! ## 任务生命周期
//! 
//! 1. **创建**: 通过 `create_task` 创建新任务
//! 2. **获取**: 通过 `acquire_task` 获取待处理任务
//! 3. **执行**: 工作节点执行任务
//! 4. **完成**: 通过 `complete_task` 标记任务完成
//! 5. **清理**: 后台自动清理过期任务
//! 
//! ## 并发控制
//! 
//! - 使用分布式锁确保任务不会被多个工作节点同时执行
//! - 支持锁的超时和自动释放
//! - 提供锁的获取和释放管理
//! 
//! ## 错误处理
//! 
//! - 任务失败时自动重试（最多3次）
//! - 超时任务自动标记为失败
//! - 详细的错误信息和上下文
//! 
//! ## 监控和统计
//! 
//! - 实时任务统计信息
//! - 成功率计算
//! - 任务执行时间监控
//! - 后台定时统计输出

use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::domain::{
    Task, TaskId, TaskStatus, WorkerId, 
    CreateTaskRequest, CompleteTaskRequest, AcquireTaskRequest,
    TaskFilter, TaskStatistics, TaskResult,
};
use crate::infrastructure::{TaskRepository, LockManager};

pub mod execution_service;
pub use execution_service::{TaskExecutionService, ExecutionStats};

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
            cleanup_interval: 3600,
            timeout_check_interval: 60,
            metrics_interval: 60,
        }
    }
    
    /// 创建任务
    pub async fn create_task(&self, request: CreateTaskRequest) -> Result<Task, String> {
        // 验证工作目录
        if !request.work_directory.starts_with('/') {
            return Err("Work directory must be an absolute path".to_string());
        }
        
        if request.work_directory.is_empty() || request.work_directory.len() > 1000 {
            return Err("Work directory must be between 1 and 1000 characters".to_string());
        }
        
        // 验证提示
        if request.prompt.is_empty() || request.prompt.len() > 10000 {
            return Err("Prompt must be between 1 and 10000 characters".to_string());
        }
        
        // 验证标签
        let tags = request.tags.unwrap_or_default();
        for tag in &tags {
            if tag.is_empty() || tag.len() > 50 {
                return Err("Tags must be between 1 and 50 characters".to_string());
            }
        }
        
        // 创建任务
        let mut task = Task::new(
            request.work_directory,
            request.prompt,
            request.priority.unwrap_or_default(),
            tags,
        );
        
        // 设置执行模式
        if let Some(execution_mode) = request.execution_mode {
            task.execution_mode = execution_mode;
        }
        
        task.max_retries = self.max_retries;
        
        // 保存到仓库
        let task_id = self.task_repository.create_task(&task).await?;
        task.id = task_id;
        
        Ok(task)
    }
    
    /// 获取任务
    pub async fn get_task(&self, task_id: &TaskId) -> Result<Task, String> {
        self.task_repository.get_task(task_id).await?
            .ok_or_else(|| format!("Task not found: {}", task_id))
    }
    
    /// 获取下一个待处理任务
    pub async fn acquire_task(&self, request: AcquireTaskRequest) -> Result<Option<Task>, String> {
        // 验证worker_id
        if request.worker_id.is_empty() || request.worker_id.len() > 100 {
            return Err("Worker ID must be between 1 and 100 characters".to_string());
        }
        
        // 验证工作路径
        if !request.work_path.starts_with('/') {
            return Err("Work path must be an absolute path".to_string());
        }
        
        // 获取任务
        let task = self.task_repository
            .get_next_task(&request.work_path, &request.worker_id)
            .await?;
        
        if let Some(mut task) = task {
            // 尝试获取锁
            let lock_acquired = self.lock_manager
                .try_acquire(&task.id.to_string(), &request.worker_id, 3600)
                .await?;
            
            if !lock_acquired {
                return Ok(None);
            }
            
            // 开始任务
            let worker_id = WorkerId::new(request.worker_id.clone())?;
            task.start(worker_id)?;
            
            // 更新任务
            self.task_repository.update_task(&task).await?;
            
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
    
    /// 完成任务
    pub async fn complete_task(&self, task_id: &TaskId, request: CompleteTaskRequest) -> Result<Task, String> {
        let mut task = self.get_task(task_id).await?;
        
        // 验证任务状态
        if task.status != TaskStatus::Working {
            return Err(format!("Cannot complete task in {:?} status", task.status));
        }
        
        // 验证原始提示（如果提供）
        if let Some(original_prompt) = &request.original_prompt {
            if !task.prompt.contains(original_prompt) {
                return Err("Original prompt does not match".to_string());
            }
        }
        
        // 完成任务
        let result = request.result.unwrap_or_else(|| TaskResult::success("Task completed".to_string()));
        task.complete(result)?;
        
        // 更新任务
        self.task_repository.update_task(&task).await?;
        
        // 释放锁
        if let Some(worker_id) = &task.worker_id {
            let _ = self.lock_manager.release(&task_id.to_string(), worker_id.as_str()).await;
        }
        
        Ok(task)
    }
    
    /// 任务失败
    pub async fn fail_task(&self, task_id: &TaskId, error: String) -> Result<Task, String> {
        let mut task = self.get_task(task_id).await?;
        
        // 验证任务状态
        if task.status != TaskStatus::Working {
            return Err(format!("Cannot fail task in {:?} status", task.status));
        }
        
        // 处理失败
        task.fail(error)?;
        
        // 更新任务
        self.task_repository.update_task(&task).await?;
        
        // 释放锁
        if let Some(worker_id) = &task.worker_id {
            let _ = self.lock_manager.release(&task_id.to_string(), worker_id.as_str()).await;
        }
        
        Ok(task)
    }
    
    /// 取消任务
    pub async fn cancel_task(&self, task_id: &TaskId, reason: Option<String>) -> Result<Task, String> {
        let mut task = self.get_task(task_id).await?;
        
        // 验证任务状态
        if task.status.is_terminal() {
            return Err(format!("Cannot cancel task in {:?} status", task.status));
        }
        
        // 取消任务
        task.cancel(reason)?;
        
        // 更新任务
        self.task_repository.update_task(&task).await?;
        
        // 释放锁
        if let Some(worker_id) = &task.worker_id {
            let _ = self.lock_manager.release(&task_id.to_string(), worker_id.as_str()).await;
        }
        
        Ok(task)
    }
    
    /// 重试任务
    pub async fn retry_task(&self, task_id: &TaskId) -> Result<Task, String> {
        let mut task = self.get_task(task_id).await?;
        
        // 验证任务状态
        if task.status != TaskStatus::Failed {
            return Err("Only failed tasks can be retried".to_string());
        }
        
        // 重试任务
        task.retry()?;
        
        // 更新任务
        self.task_repository.update_task(&task).await?;
        
        Ok(task)
    }
    
    /// 列出任务
    pub async fn list_tasks(&self, filter: TaskFilter) -> Result<(Vec<Task>, u64), String> {
        self.task_repository.list_tasks(&filter).await
    }
    
    /// 获取任务统计
    pub async fn get_statistics(&self) -> Result<TaskStatistics, String> {
        self.task_repository.get_statistics().await
    }
    
    /// 清理过期任务
    pub async fn cleanup_expired_tasks(&self, older_than: DateTime<Utc>) -> Result<u64, String> {
        self.task_repository.cleanup_expired_tasks(older_than).await
    }
    
    /// 重试失败任务
    pub async fn retry_failed_tasks(&self) -> Result<u64, String> {
        self.task_repository.retry_failed_tasks(self.max_retries).await
    }
    
    /// 检查任务是否过期
    pub async fn check_task_timeout(&self, task_id: &TaskId) -> Result<bool, String> {
        let task = self.get_task(task_id).await?;
        Ok(task.is_expired(self.task_timeout))
    }
    
    /// 处理超时任务
    pub async fn handle_timeout_tasks(&self) -> Result<u64, String> {
        // 查找所有超时的任务
        let filter = TaskFilter::new()
            .with_status(TaskStatus::Working);
        
        let (tasks, _) = self.list_tasks(filter).await?;
        let mut handled = 0;
        
        for task in tasks {
            if task.is_expired(self.task_timeout) {
                if let Err(_) = self.fail_task(&task.id, "Task timeout".to_string()).await {
                    continue;
                }
                handled += 1;
            }
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
    pub async fn start(&self) -> Result<(), String> {
        let task_service = self.task_service.clone();
        
        // 启动清理任务
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(task_service.cleanup_interval));
            loop {
                interval.tick().await;
                if let Err(e) = task_service.cleanup_expired_tasks(Utc::now() - chrono::Duration::days(30)).await {
                    eprintln!("Failed to cleanup expired tasks: {}", e);
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
                    eprintln!("Failed to handle timeout tasks: {}", e);
                }
            }
        });
        
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
    pub async fn start(&self) -> Result<(), String> {
        let task_service = self.task_service.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(task_service.metrics_interval));
            loop {
                interval.tick().await;
                
                match task_service.get_statistics().await {
                    Ok(stats) => {
                        println!("Task Statistics: {} total, {} completed, {} failed, {:.2}% success rate",
                            stats.total_tasks,
                            stats.completed_tasks,
                            stats.failed_tasks,
                            stats.success_rate * 100.0
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to get task statistics: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
}