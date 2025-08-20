//! HTTP协议JSON验证MCP服务器
//! 
//! 这是一个基于Axum和Tower的高性能HTTP JSON验证服务器，
//! 实现了JSON-RPC over HTTP协议，提供了企业级的性能、安全性和可扩展性。

pub mod app;
pub mod config;
pub mod handlers;
pub mod models;
pub mod services;
pub mod tls;
pub mod performance;
pub mod utils;

pub use app::create_app;
pub use config::ServerConfig;
pub use models::*;
pub use services::JsonValidatorService;

// 宏在utils/logging.rs中定义并导出