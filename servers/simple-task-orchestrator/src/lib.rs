//! # Simple Task Orchestrator MCP Server
//! 
//! 这是一个基于Rust的MCP (Model Context Protocol) 服务器，提供任务编排和执行功能。
//! 
//! ## 主要功能
//! 
//! - **任务管理**: 创建、执行、监控和完成任务
//! - **多种执行模式**: 支持标准执行和ClaudeCode AI辅助执行
//! - **完整的生命周期管理**: 任务从创建到完成的完整流程
//! - **错误处理和重试机制**: 健壮的错误处理和任务重试
//! - **统计信息**: 提供任务执行统计和监控
//! 
//! ## 架构设计
//! 
//! 服务器采用分层架构设计：
//! 
//! - **配置层** (`config`): 管理服务器配置和设置
//! - **领域层** (`domain`): 定义核心业务领域模型
//! - **基础设施层** (`infrastructure`): 提供基础设施支持
//! - **服务层** (`services`): 实现业务逻辑和核心服务
//! - **处理器层** (`handlers`): 处理MCP协议请求和响应
//! - **执行层** (`execution`): 提供任务执行功能
//! - **工具层** (`utils`): 提供通用工具和辅助函数
//! - **错误处理** (`errors`): 统一的错误处理机制
//! - **MCP服务器** (`mcp_server`): MCP协议实现
//! 
//! ## 使用示例
//! 
//! ```rust
//! use simple_task_orchestrator::mcp_server::McpServer;
//! use simple_task_orchestrator::config::AppConfig;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 加载配置
//!     let config = AppConfig::load()?;
//!     
//!     // 创建并启动MCP服务器
//!     let server = McpServer::new(config).await?;
//!     server.run().await?;
//!     
//!     Ok(())
//! }
//! ```
//! 
//! ## 可用工具
//! 
//! - `create_task` - 创建新任务
//! - `get_task` - 获取任务信息
//! - `acquire_task` - 获取待处理任务
//! - `execute_task` - 执行任务
//! - `complete_task` - 完成任务
//! - `list_tasks` - 列出任务
//! - `get_statistics` - 获取统计信息
//! 
//! ## 配置
//! 
//! 服务器支持通过配置文件进行配置，主要配置项包括：
//! 
//! - 服务器设置（端口、主机等）
//! - 任务执行配置
//! - 数据库连接配置
//! - 日志配置
//! - ClaudeCode集成配置
//! 
//! ## 错误处理
//! 
//! 所有操作都返回 `Result<T, Error>`，其中 `Error` 是统一的错误类型，
//! 包含详细的错误信息和上下文。

pub mod config;
pub mod domain;
pub mod infrastructure;
pub mod services;
pub mod handlers;
pub mod errors;
pub mod utils;
pub mod execution;
pub mod mcp_server;