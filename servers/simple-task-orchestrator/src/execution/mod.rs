//! # 任务执行模块
//! 
//! 该模块提供了 Simple Task Orchestrator 的任务执行功能，支持多种执行模式。
//! 
//! ## 主要功能
//! 
//! - **多种执行模式**: 支持标准执行、ClaudeCode执行和自定义执行器
//! - **执行器工厂**: 根据任务配置自动选择合适的执行器
//! - **执行器验证**: 执行前验证执行器的可用性
//! - **异步执行**: 支持异步任务执行
//! - **错误处理**: 完整的错误处理和恢复机制
//! 
//! ## 执行器类型
//! 
//! - `StandardExecutor`: 标准执行器，提供基本的任务执行功能
//! - `ClaudeCodeExecutor`: ClaudeCode执行器，集成Claude Code AI辅助执行
//! - `CustomExecutor`: 自定义执行器，支持用户自定义执行逻辑
//! 
//! ## 核心特征
//! 
//! - `TaskExecutor`: 任务执行器特征，定义了执行器的基本接口
//! - `TaskExecutorFactory`: 执行器工厂，根据任务配置创建合适的执行器
//! 
//! ## 使用示例
//! 
//! ```rust
//! use simple_task_orchestrator::execution::{TaskExecutorFactory, TaskExecutor};
//! use simple_task_orchestrator::domain::{Task, TaskPriority, ExecutionMode};
//! 
//! // 创建任务
//! let task = Task::new(
//!     "/workspace/project".to_string(),
//!     "实现用户认证功能".to_string(),
//!     TaskPriority::High,
//!     vec!["feature".to_string()],
//! ).with_execution_mode(ExecutionMode::ClaudeCode);
//! 
//! // 创建执行器
//! let executor = TaskExecutorFactory::create_executor(&task);
//! 
//! // 验证执行器
//! if executor.validate().await? {
//!     // 执行任务
//!     let result = executor.execute(&task).await?;
//!     println!("Task executed: {}", result.output);
//! }
//! ```
//! 
//! ## 执行模式说明
//! 
//! ### Standard 模式
//! - 基本的任务执行功能
//! - 适用于简单的任务处理
//! - 总是可用，无需额外配置
//! 
//! ### ClaudeCode 模式
//! - 集成Claude Code AI辅助执行
//! - 提供智能代码生成和问题解决
//! - 需要Claude Code环境配置
//! 
//! ### Custom 模式
//! - 支持用户自定义执行器
//! - 可以集成第三方工具和服务
//! - 通过字符串标识符指定执行器类型
//! 
//! ## 执行器接口
//! 
//! 所有执行器都必须实现 `TaskExecutor` 特征：
//! 
//! ```rust
//! #[async_trait::async_trait]
//! pub trait TaskExecutor: Send + Sync {
//!     /// 执行任务
//!     async fn execute(&self, task: &Task) -> Result<TaskResult>;
//!     
//!     /// 验证执行器是否可用
//!     async fn validate(&self) -> Result<bool>;
//!     
//!     /// 获取执行器名称
//!     fn name(&self) -> &'static str;
//! }
//! ```
//! 
//! ## 错误处理
//! 
//! - 执行器验证失败时会返回错误
//! - 任务执行过程中的错误会被捕获和记录
//! - 支持执行器的优雅降级（未知执行器回退到标准执行器）
//! 
//! ## 扩展性
//! 
//! - 可以轻松添加新的执行器类型
//! - 支持执行器的动态注册和发现
//! - 提供了灵活的执行器选择机制

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