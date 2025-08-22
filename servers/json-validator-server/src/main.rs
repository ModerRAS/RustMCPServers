//! # JSON Validator MCP Server
//! 
//! 这是一个基于Rust的MCP (Model Context Protocol) 服务器，提供JSON数据验证功能。
//! 
//! ## 主要功能
//! 
//! - **JSON格式验证**: 验证JSON数据的基本格式和结构
//! - **JSON Schema验证**: 根据JSON Schema验证数据的完整性和正确性
//! - **批量验证**: 支持同时验证多个JSON文档
//! - **详细错误报告**: 提供详细的验证错误信息和位置
//! - **高性能**: 优化的验证算法和内存管理
//! 
//! ## 实现的MCP工具
//! 
//! - `validate_json` - 基础JSON格式验证
//! - `validate_json_with_schema` - JSON Schema验证
//! - `validate_json_batch` - 批量JSON验证
//! 
//! ## 使用示例
//! 
//! ```bash
//! # 编译项目
//! cargo build --release
//! 
//! # 启动服务器（stdio模式）
//! cargo run
//! 
//! # 设置日志级别
//! RUST_LOG=info cargo run
//! ```
//! 
//! ## MCP协议支持
//! 
//! 服务器支持MCP协议的stdio传输模式，可以与各种MCP客户端集成。
//! 
//! ## 技术特点
//! 
//! - 使用rmcp框架实现MCP协议
//! - 基于serde_json进行JSON解析和序列化
//! - 支持异步处理和高并发
//! - 完整的错误处理和日志记录

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
mod json_validator;

/// 应用程序主入口点
/// 
/// 该函数是JSON Validator MCP服务器的主入口点，负责：
/// 
/// 1. 初始化日志系统
/// 2. 创建JSON验证服务实例
/// 3. 启动stdio传输的MCP服务器
/// 4. 等待服务完成
/// 
/// # 返回值
/// 
/// 返回 `Result<()>`：
/// - `Ok(())`: 服务器正常启动和运行
/// - `Err(error)`: 启动或运行过程中的错误
/// 
/// # 错误处理
/// 
/// 该函数会处理以下错误情况：
/// - 日志初始化失败
/// - MCP服务器启动失败
/// - stdio传输错误
/// 
/// # 日志配置
/// 
/// - 默认日志级别：DEBUG
/// - 输出到stderr
/// - 禁用ANSI颜色（适合stdio传输）
/// 
/// # 使用示例
/// 
/// ```bash
//! cargo run
//! ```
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting JSON Validator MCP server");

    // 创建服务实例并启动stdio服务器
    let service = json_validator::JsonValidator::new().serve(stdio()).await.inspect_err(|e| {
        tracing::error!("serving error: {:?}", e);
    })?;

    service.waiting().await?;
    Ok(())
}
