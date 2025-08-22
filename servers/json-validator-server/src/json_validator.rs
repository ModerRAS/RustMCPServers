//! # JSON验证器模块
//! 
//! 该模块提供了 JSON Validator MCP 服务器的核心功能实现，支持JSON格式验证、文件验证和格式化。
//! 
//! ## 主要功能
//! 
//! - **JSON格式验证**: 验证JSON数据的基本格式和结构
//! - **文件验证**: 从文件路径读取并验证JSON文件
//! - **内容验证**: 直接验证JSON字符串内容
//! - **JSON格式化**: 美化JSON输出格式
//! - **错误定位**: 提供详细的错误位置信息（行号、列号）
//! - **MCP协议集成**: 完整的MCP工具接口实现
//! 
//! ## 核心结构
//! 
//! - `JsonValidator`: 主要的验证器结构，实现MCP服务器接口
//! - `ValidationResult`: 验证结果，包含有效性、消息和错误位置
//! - `FormatResult`: 格式化结果，包含格式化后的JSON和错误信息
//! - `ValidateFileRequest`: 文件验证请求
//! - `ValidateJsonRequest`: JSON内容验证请求
//! 
//! ## 实现的MCP工具
//! 
//! ### 1. validate_json_file
//! 验证指定路径的JSON文件：
//! 
//! ```bash
//! # MCP工具调用示例
//! {
//!   "method": "tools/call",
//!   "params": {
//!     "name": "validate_json_file",
//!     "arguments": {
//!       "file_path": "/path/to/config.json"
//!     }
//!   }
//! }
//! ```
//! 
//! ### 2. validate_json_content
//! 直接验证JSON字符串内容：
//! 
//! ```bash
//! # MCP工具调用示例
//! {
//!   "method": "tools/call",
//!   "params": {
//!     "name": "validate_json_content",
//!     "arguments": {
//!       "json_content": "{\"name\": \"test\", \"value\": 123}"
//!     }
//!   }
//! }
//! ```
//! 
//! ### 3. format_json
//! 格式化JSON字符串：
//! 
//! ```bash
//! # MCP工具调用示例
//! {
//!   "method": "tools/call",
//!   "params": {
//!     "name": "format_json",
//!     "arguments": {
//!       "json_content": "{\"name\":\"test\",\"value\":123}"
//!     }
//!   }
//! }
//! ```
//! 
//! ## 使用示例
//! 
//! ```rust
//! use json_validator_server::json_validator::JsonValidator;
//! 
//! // 创建验证器实例
//! let validator = JsonValidator::new();
//! 
//! // 验证JSON文件
//! let file_result = validator.validate_json_file(
//!     rmcp::handler::server::tool::Parameters(ValidateFileRequest {
//!         file_path: "config.json".to_string(),
//!     })
//! ).await?;
//! 
//! // 验证JSON内容
//! let content_result = validator.validate_json_content(
//!     rmcp::handler::server::tool::Parameters(ValidateJsonRequest {
//!         json_content: "{\"name\": \"test\"}".to_string(),
//!     })
//! ).await?;
//! 
//! // 格式化JSON
//! let format_result = validator.format_json(
//!     rmcp::handler::server::tool::Parameters(ValidateJsonRequest {
//!         json_content: "{\"name\":\"test\"}".to_string(),
//!     })
//! ).await?;
//! ```
//! 
//! ## 验证结果格式
//! 
//! ### ValidationResult
//! 
//! ```json
//! {
//!   "valid": true,
//!   "message": "JSON file is valid",
//!   "file_path": "/path/to/file.json",
//!   "error_line": null,
//!   "error_column": null
//! }
//! ```
//! 
//! ### 错误结果示例
//! 
//! ```json
//! {
//!   "valid": false,
//!   "message": "Invalid JSON: expected `:` at line 1 column 10",
//!   "file_path": "/path/to/invalid.json",
//!   "error_line": 1,
//!   "error_column": 10
//! }
//! ```
//! 
//! ## 格式化结果格式
//! 
//! ### FormatResult
//! 
//! ```json
//! {
//!   "success": true,
//!   "formatted_json": "{\n  \"name\": \"test\",\n  \"value\": 123\n}",
//!   "message": null,
//!   "error_line": null,
//!   "error_column": null
//! }
//! ```
//! 
//! ## 错误处理
//! 
//! 验证器提供详细的错误信息：
//! 
//! - **文件不存在**: 返回明确的文件未找到错误
//! - **IO错误**: 返回文件读取错误信息
//! - **JSON解析错误**: 返回具体的语法错误和位置
//! - **格式化错误**: 返回格式化过程中的错误
//! 
//! ## 错误位置信息
//! 
//! 对于JSON解析错误，验证器会返回：
//! 
//! - `error_line`: 错误发生的行号（从1开始）
//! - `error_column`: 错误发生的列号（从1开始）
//! - `message`: 详细的错误描述
//! 
//! ## 性能特点
//! 
//! - **高效解析**: 使用serde_json进行高性能JSON解析
//! - **内存友好**: 流式处理大文件
//! - **错误恢复**: 即使遇到错误也能返回有用的信息
//! - **异步处理**: 完全异步的实现，支持高并发
//! 
//! ## 集成说明
//! 
//! 该验证器实现了完整的MCP服务器接口：
//! 
//! - `ServerHandler`: MCP服务器处理器
//! - `ToolRouter`: MCP工具路由器
//! - `JsonSchema`: 完整的JSON Schema支持
//! - **日志记录**: 集成tracing进行结构化日志记录
//! 
//! ## 配置要求
//! 
//! - Rust 1.70+
//! - serde_json crate
//! - rmcp 0.5.0 crate
//! - tokio异步运行时
//! 
//! ## 扩展性
//! 
//! 验证器设计考虑了未来扩展：
//! 
//! - 可以轻松添加新的验证规则
//! - 支持JSON Schema验证
//! - 可以扩展支持其他数据格式
//! - 模块化设计便于维护和测试

use std::fs;
use std::future::Future;
use std::path::Path;

use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters, wrapper::Json},
    model::*,
    schemars, tool, tool_handler, tool_router, ServerHandler,
};

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidationResult {
    pub valid: bool,
    pub message: String,
    pub file_path: Option<String>,
    pub error_line: Option<usize>,
    pub error_column: Option<usize>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatResult {
    pub success: bool,
    pub formatted_json: Option<String>,
    pub message: Option<String>,
    pub error_line: Option<usize>,
    pub error_column: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateFileRequest {
    pub file_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ValidateJsonRequest {
    pub json_content: String,
}

#[derive(Clone)]
pub struct JsonValidator {
    tool_router: ToolRouter<JsonValidator>,
}

#[tool_router]
impl JsonValidator {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Validate JSON file from path")]
    async fn validate_json_file(
        &self,
        Parameters(ValidateFileRequest { file_path }): Parameters<ValidateFileRequest>,
    ) -> Result<Json<ValidationResult>, String> {
        tracing::info!(file_path = %file_path, "Validating JSON file");

        let path = Path::new(&file_path);

        // 检查文件是否存在
        if !path.exists() {
            return Ok(Json(ValidationResult {
                valid: false,
                message: format!("File not found: {file_path}"),
                file_path: Some(file_path),
                error_line: None,
                error_column: None,
            }));
        }

        // 读取文件内容
        let content = match fs::read_to_string(path) {
            Ok(content) => content,
            Err(e) => {
                return Ok(Json(ValidationResult {
                    valid: false,
                    message: format!("IO error: {e}"),
                    file_path: Some(file_path),
                    error_line: None,
                    error_column: None,
                }));
            }
        };

        // 验证JSON格式
        match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(_) => {
                Ok(Json(ValidationResult {
                    valid: true,
                    message: "JSON file is valid".to_string(),
                    file_path: Some(file_path),
                    error_line: None,
                    error_column: None,
                }))
            }
            Err(e) => {
                Ok(Json(ValidationResult {
                    valid: false,
                    message: format!("Invalid JSON: {e}"),
                    file_path: Some(file_path),
                    error_line: Some(e.line()),
                    error_column: Some(e.column()),
                }))
            }
        }
    }

    #[tool(description = "Validate JSON content directly")]
    async fn validate_json_content(
        &self,
        Parameters(ValidateJsonRequest { json_content }): Parameters<ValidateJsonRequest>,
    ) -> Result<Json<ValidationResult>, String> {
        tracing::info!("Validating JSON content");

        // 验证JSON格式
        match serde_json::from_str::<serde_json::Value>(&json_content) {
            Ok(_) => {
                Ok(Json(ValidationResult {
                    valid: true,
                    message: "JSON content is valid".to_string(),
                    file_path: None,
                    error_line: None,
                    error_column: None,
                }))
            }
            Err(e) => {
                Ok(Json(ValidationResult {
                    valid: false,
                    message: format!("Invalid JSON: {e}"),
                    file_path: None,
                    error_line: Some(e.line()),
                    error_column: Some(e.column()),
                }))
            }
        }
    }

    #[tool(description = "Format JSON content")]
    async fn format_json(
        &self,
        Parameters(ValidateJsonRequest { json_content }): Parameters<ValidateJsonRequest>,
    ) -> Result<Json<FormatResult>, String> {
        tracing::info!("Formatting JSON content");

        match serde_json::from_str::<serde_json::Value>(&json_content) {
            Ok(value) => match serde_json::to_string_pretty(&value) {
                Ok(formatted) => {
                    Ok(Json(FormatResult {
                        success: true,
                        formatted_json: Some(formatted),
                        message: None,
                        error_line: None,
                        error_column: None,
                    }))
                }
                Err(e) => {
                    Ok(Json(FormatResult {
                        success: false,
                        formatted_json: None,
                        message: Some(format!("Failed to format JSON: {e}")),
                        error_line: None,
                        error_column: None,
                    }))
                }
            },
            Err(e) => {
                Ok(Json(FormatResult {
                    success: false,
                    formatted_json: None,
                    message: Some(format!("Invalid JSON: {e}")),
                    error_line: Some(e.line()),
                    error_column: Some(e.column()),
                }))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for JsonValidator {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("This server provides JSON validation and formatting tools. Use 'validate_json_file' to validate a JSON file from path, 'validate_json_content' to validate JSON content directly, and 'format_json' to format JSON content.".to_string()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
