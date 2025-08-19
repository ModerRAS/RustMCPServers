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
