use std::sync::Arc;
use chrono::Utc;
use crate::domain::{Task, TaskId, TaskResult, TaskStatus};
use crate::infrastructure::TaskRepository;
use crate::execution::{TaskExecutor, TaskExecutorFactory};
use anyhow::{Result, Context};

/// 任务执行服务
pub struct TaskExecutionService {
    task_repository: Arc<dyn TaskRepository>,
}

impl TaskExecutionService {
    pub fn new(task_repository: Arc<dyn TaskRepository>) -> Self {
        Self {
            task_repository,
        }
    }

    /// 执行单个任务
    pub async fn execute_task(&self, task_id: &TaskId) -> Result<TaskResult> {
        // 获取任务
        let task_option = self.task_repository.get_task(task_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get task for execution: {}", e))?;
        
        let mut task = task_option.ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        
        // 检查任务状态
        if task.status != TaskStatus::Working {
            return Err(anyhow::anyhow!("Task is not in working state: {:?}", task.status));
        }

        // 创建执行器
        let executor = TaskExecutorFactory::create_executor(&task);
        
        // 验证执行器
        if !executor.validate().await.unwrap_or(false) {
            return Err(anyhow::anyhow!("Executor {} is not available", executor.name()));
        }

        // 执行任务
        let result = executor.execute(&task).await
            .context("Failed to execute task")?;

        // 更新任务状态
        task.status = TaskStatus::Completed;
        task.completed_at = Some(Utc::now());
        task.result = Some(result.clone());
        
        // 保存更新后的任务
        self.task_repository.update_task(&task).await
            .map_err(|e| anyhow::anyhow!("Failed to update task after execution: {}", e))?;

        Ok(result)
    }

    /// 批量执行任务
    pub async fn execute_tasks(&self, task_ids: &[TaskId]) -> Vec<(TaskId, Result<TaskResult>)> {
        let mut results = Vec::new();
        
        for task_id in task_ids {
            let result = self.execute_task(task_id).await;
            results.push((task_id.clone(), result));
        }
        
        results
    }

    /// 执行指定工作目录的所有待处理任务
    pub async fn execute_tasks_in_directory(&self, work_directory: &str) -> Result<Vec<(TaskId, Result<TaskResult>)>> {
        // 获取指定目录的工作状态任务
        let tasks = self.task_repository.get_tasks_by_work_directory(work_directory).await
            .map_err(|e| anyhow::anyhow!("Failed to get tasks in directory: {}", e))?;
        
        let working_tasks: Vec<TaskId> = tasks.into_iter()
            .filter(|task| task.status == TaskStatus::Working)
            .map(|task| task.id)
            .collect();
        
        let results = self.execute_tasks(&working_tasks).await;
        Ok(results)
    }

    /// 检查任务是否准备好执行
    pub async fn is_task_ready_for_execution(&self, task_id: &TaskId) -> Result<bool> {
        let task_option = self.task_repository.get_task(task_id).await
            .map_err(|e| anyhow::anyhow!("Failed to get task: {}", e))?;
        
        let task = task_option.ok_or_else(|| anyhow::anyhow!("Task not found"))?;
        Ok(task.status == TaskStatus::Working)
    }

    /// 获取执行统计
    pub async fn get_execution_stats(&self) -> Result<ExecutionStats> {
        let all_tasks = self.task_repository.get_all_tasks().await
            .map_err(|e| anyhow::anyhow!("Failed to get all tasks: {}", e))?;
        
        let mut stats = ExecutionStats::default();
        
        for task in all_tasks {
            stats.total_tasks += 1;
            
            match task.status {
                TaskStatus::Completed => {
                    stats.completed_tasks += 1;
                    if let Some(ref result) = task.result {
                        if result.status == "success" {
                            stats.successful_tasks += 1;
                        } else {
                            stats.failed_tasks += 1;
                        }
                    }
                }
                TaskStatus::Failed => {
                    stats.failed_tasks += 1;
                }
                TaskStatus::Working => {
                    stats.working_tasks += 1;
                }
                TaskStatus::Waiting => {
                    stats.waiting_tasks += 1;
                }
                TaskStatus::Cancelled => {
                    stats.cancelled_tasks += 1;
                }
            }
            
            // 计算平均执行时间
            if let Some(ref result) = task.result {
                stats.total_execution_time += result.duration_ms;
                if stats.completed_tasks > 0 {
                    stats.average_execution_time = stats.total_execution_time / stats.completed_tasks;
                }
            }
        }
        
        Ok(stats)
    }
}

/// 执行统计
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub successful_tasks: u64,
    pub failed_tasks: u64,
    pub working_tasks: u64,
    pub waiting_tasks: u64,
    pub cancelled_tasks: u64,
    pub total_execution_time: u64,
    pub average_execution_time: u64,
}

impl ExecutionStats {
    pub fn success_rate(&self) -> f64 {
        if self.completed_tasks == 0 {
            0.0
        } else {
            (self.successful_tasks as f64 / self.completed_tasks as f64) * 100.0
        }
    }

    pub fn completion_rate(&self) -> f64 {
        if self.total_tasks == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / self.total_tasks as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Task, TaskPriority, ExecutionMode};
    use crate::infrastructure::InMemoryTaskRepository;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_task_execution_service() {
        let task_repository = Arc::new(InMemoryTaskRepository::new());
        let execution_service = TaskExecutionService::new(task_repository.clone());
        
        // 创建测试任务
        let task = Task::new(
            "/test".to_string(),
            "Test task".to_string(),
            TaskPriority::Medium,
            vec!["test".to_string()],
        );
        
        let task_id = task.id;
        task_repository.create_task(&task).await.unwrap();
        
        // 测试执行准备检查
        let is_ready = execution_service.is_task_ready_for_execution(&task_id).await.unwrap();
        assert!(!is_ready); // 任务不是Working状态
        
        // 测试获取统计
        let stats = execution_service.get_execution_stats().await.unwrap();
        assert_eq!(stats.total_tasks, 1);
        assert_eq!(stats.waiting_tasks, 1);
    }

    #[tokio::test]
    async fn test_execution_stats() {
        let mut stats = ExecutionStats::default();
        
        stats.total_tasks = 10;
        stats.completed_tasks = 8;
        stats.successful_tasks = 6;
        stats.failed_tasks = 2;
        stats.total_execution_time = 1000;
        stats.average_execution_time = 125; // 直接设置平均值
        
        assert_eq!(stats.success_rate(), 75.0);
        assert_eq!(stats.completion_rate(), 80.0);
        assert_eq!(stats.average_execution_time, 125);
    }
}