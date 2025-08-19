pub mod claude_code_executor;

pub use claude_code_executor::{ClaudeCodeExecutor, ClaudeCodeConfig};

use crate::domain::{Task, TaskResult};
use anyhow::Result;

/// 任务执行器特征
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    /// 执行任务
    async fn execute(&self, task: &Task) -> Result<TaskResult>;
    
    /// 验证执行器是否可用
    async fn validate(&self) -> Result<bool>;
    
    /// 获取执行器名称
    fn name(&self) -> &'static str;
}

/// 标准任务执行器
pub struct StandardExecutor;

#[async_trait::async_trait]
impl TaskExecutor for StandardExecutor {
    async fn execute(&self, task: &Task) -> Result<TaskResult> {
        // 标准执行器只是返回任务提示作为结果
        // 实际使用中，这里可以集成其他执行逻辑
        Ok(TaskResult::success(format!(
            "Standard execution for task in {}: {}",
            task.work_directory, task.prompt
        )))
    }

    async fn validate(&self) -> Result<bool> {
        Ok(true) // 标准执行器总是可用
    }

    fn name(&self) -> &'static str {
        "standard"
    }
}

/// 任务执行器工厂
pub struct TaskExecutorFactory;

impl TaskExecutorFactory {
    /// 创建任务执行器
    pub fn create_executor(task: &Task) -> Box<dyn TaskExecutor> {
        match task.execution_mode {
            crate::domain::ExecutionMode::Standard => {
                Box::new(StandardExecutor)
            }
            crate::domain::ExecutionMode::ClaudeCode => {
                let config = ClaudeCodeConfig {
                    work_directory: task.work_directory.clone(),
                    ..Default::default()
                };
                Box::new(ClaudeCodeExecutor::new(config))
            }
            crate::domain::ExecutionMode::Custom(ref executor_name) => {
                match executor_name.as_str() {
                    "standard" => Box::new(StandardExecutor),
                    _ => {
                        // 未知执行器，使用标准执行器
                        eprintln!("Unknown executor: {}, falling back to standard", executor_name);
                        Box::new(StandardExecutor)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Task, TaskPriority, ExecutionMode};

    #[tokio::test]
    async fn test_standard_executor() {
        let executor = StandardExecutor;
        
        let task = Task::new(
            "/test".to_string(),
            "Test task".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        
        let result = executor.execute(&task).await.unwrap();
        assert_eq!(result.status, "success");
        assert!(result.output.contains("Standard execution"));
        
        assert!(executor.validate().await.unwrap());
        assert_eq!(executor.name(), "standard");
    }

    #[tokio::test]
    async fn test_executor_factory() {
        let mut task = Task::new(
            "/test".to_string(),
            "Test task".to_string(),
            TaskPriority::Medium,
            vec![],
        );
        
        // 测试标准执行器
        let executor = TaskExecutorFactory::create_executor(&task);
        assert_eq!(executor.name(), "standard");
        
        // 测试Claude Code执行器
        task.execution_mode = ExecutionMode::ClaudeCode;
        let executor = TaskExecutorFactory::create_executor(&task);
        assert_eq!(executor.name(), "claude_code");
        
        // 测试自定义执行器
        task.execution_mode = ExecutionMode::Custom("standard".to_string());
        let executor = TaskExecutorFactory::create_executor(&task);
        assert_eq!(executor.name(), "standard");
        
        // 测试未知执行器
        task.execution_mode = ExecutionMode::Custom("unknown".to_string());
        let executor = TaskExecutorFactory::create_executor(&task);
        assert_eq!(executor.name(), "standard"); // 回退到标准执行器
    }
}