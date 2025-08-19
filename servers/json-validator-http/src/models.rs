//! 数据模型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JSON-RPC 2.0 请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC版本
    pub jsonrpc: String,
    /// 调用的方法
    pub method: String,
    /// 方法参数
    pub params: Option<serde_json::Value>,
    /// 请求ID
    pub id: serde_json::Value,
}

impl JsonRpcRequest {
    /// 创建新的JSON-RPC请求
    pub fn new(method: String, params: Option<serde_json::Value>, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method,
            params,
            id,
        }
    }
    
    /// 验证请求格式
    pub fn validate(&self) -> Result<(), JsonRpcError> {
        if self.jsonrpc != "2.0" {
            return Err(JsonRpcError::new(
                -32600,
                "Invalid Request".to_string(),
                Some("jsonrpc version must be 2.0".to_string()),
            ));
        }
        
        if self.method.is_empty() {
            return Err(JsonRpcError::new(
                -32600,
                "Invalid Request".to_string(),
                Some("method cannot be empty".to_string()),
            ));
        }
        
        Ok(())
    }
}

/// JSON-RPC 2.0 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC版本
    pub jsonrpc: String,
    /// 成功结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 错误结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// 请求ID
    pub id: serde_json::Value,
}

impl JsonRpcResponse {
    /// 创建成功响应
    pub fn success(result: serde_json::Value, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }
    
    /// 创建错误响应
    pub fn error(error: JsonRpcError, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// JSON-RPC错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// 错误代码
    pub code: i32,
    /// 错误消息
    pub message: String,
    /// 错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// 创建新的JSON-RPC错误
    pub fn new(code: i32, message: String, data: Option<String>) -> Self {
        Self {
            code,
            message,
            data: data.map(|d| serde_json::Value::String(d)),
        }
    }
    
    /// 解析错误
    pub fn parse_error() -> Self {
        Self::new(
            -32700,
            "Parse error".to_string(),
            Some("Invalid JSON was received by the server".to_string()),
        )
    }
    
    /// 无效请求错误
    pub fn invalid_request() -> Self {
        Self::new(
            -32600,
            "Invalid Request".to_string(),
            Some("The JSON sent is not a valid Request object".to_string()),
        )
    }
    
    /// 方法未找到错误
    pub fn method_not_found(method: String) -> Self {
        Self::new(
            -32601,
            "Method not found".to_string(),
            Some(format!("Method '{}' not found", method)),
        )
    }
    
    /// 无效参数错误
    pub fn invalid_params(message: String) -> Self {
        Self::new(
            -32602,
            "Invalid params".to_string(),
            Some(message),
        )
    }
    
    /// 内部错误
    pub fn internal_error(message: String) -> Self {
        Self::new(
            -32603,
            "Internal error".to_string(),
            Some(message),
        )
    }
}

/// JSON验证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateJsonRequest {
    /// JSON数据
    pub json_data: serde_json::Value,
    /// 验证选项
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ValidationOptions>,
}

/// JSON Schema验证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateJsonWithSchemaRequest {
    /// JSON数据
    pub json_data: serde_json::Value,
    /// JSON Schema
    pub schema: serde_json::Value,
    /// 验证选项
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ValidationOptions>,
}

/// 批量JSON验证请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateJsonBatchRequest {
    /// 验证项列表
    pub items: Vec<BatchValidationItem>,
    /// 验证选项
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<ValidationOptions>,
}

/// 批量验证项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidationItem {
    /// 项目ID
    pub id: String,
    /// JSON数据
    pub json_data: serde_json::Value,
    /// JSON Schema（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// 验证选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationOptions {
    /// 是否启用严格模式
    #[serde(default = "default_strict_mode")]
    pub strict_mode: bool,
    /// 是否允许额外属性
    #[serde(default = "default_allow_additional")]
    pub allow_additional_properties: bool,
    /// 自定义格式验证器
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_formats: HashMap<String, String>,
    /// 是否返回详细错误信息
    #[serde(default = "default_detailed_errors")]
    pub detailed_errors: bool,
    /// 缓存键（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
}

fn default_strict_mode() -> bool {
    false
}

fn default_allow_additional() -> bool {
    true
}

fn default_detailed_errors() -> bool {
    true
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            strict_mode: false,
            allow_additional_properties: true,
            custom_formats: HashMap::new(),
            detailed_errors: true,
            cache_key: None,
        }
    }
}

/// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// 验证是否成功
    pub valid: bool,
    /// 错误列表
    pub errors: Vec<ValidationError>,
    /// 警告列表
    pub warnings: Vec<ValidationWarning>,
    /// 执行时间（毫秒）
    pub execution_time: u64,
    /// 是否命中缓存
    pub cache_hit: bool,
    /// 缓存键（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
}

impl ValidationResult {
    /// 创建成功的验证结果
    pub fn success(execution_time: u64, cache_hit: bool) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            execution_time,
            cache_hit,
            cache_key: None,
        }
    }
    
    /// 创建失败的验证结果
    pub fn failure(errors: Vec<ValidationError>, execution_time: u64, cache_hit: bool) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            execution_time,
            cache_hit,
            cache_key: None,
        }
    }
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// 实例路径
    pub instance_path: String,
    /// 模式路径
    pub schema_path: String,
    /// 错误消息
    pub message: String,
    /// 错误代码
    pub error_code: String,
    /// 错误位置（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<ErrorLocation>,
}

/// 错误位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// 行号
    pub line: usize,
    /// 列号
    pub column: usize,
}

/// 验证警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// 警告消息
    pub message: String,
    /// 警告代码
    pub warning_code: String,
    /// 路径
    pub path: String,
}

/// 批量验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchValidationResult {
    /// 项目ID
    pub id: String,
    /// 验证结果
    pub result: ValidationResult,
}

/// 健康检查响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// 服务状态
    pub status: String,
    /// 版本
    pub version: String,
    /// 启动时间
    pub start_time: String,
    /// 运行时间（秒）
    pub uptime: u64,
    /// 各组件状态
    pub components: HashMap<String, ComponentHealth>,
}

/// 组件健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// 组件名称
    pub name: String,
    /// 状态
    pub status: String,
    /// 消息
    pub message: Option<String>,
    /// 检查时间
    pub checked_at: String,
}

/// 服务器信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// 服务器名称
    pub name: String,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 构建时间
    pub build_time: String,
    /// 构建哈希
    pub build_hash: String,
    /// 支持的功能
    pub capabilities: ServerCapabilities,
}

/// 服务器功能
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// 支持的工具列表
    pub tools: Vec<String>,
    /// 支持的格式
    pub formats: Vec<String>,
    /// 是否支持缓存
    pub cache: bool,
    /// 是否支持批量操作
    pub batch: bool,
    /// 是否支持自定义格式
    pub custom_formats: bool,
}

/// 指标数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    /// 请求总数
    pub requests_total: u64,
    /// 成功请求数
    pub requests_success: u64,
    /// 失败请求数
    pub requests_failed: u64,
    /// 平均响应时间（毫秒）
    pub avg_response_time: f64,
    /// 缓存命中率
    pub cache_hit_rate: f64,
    /// 活跃连接数
    pub active_connections: u64,
    /// 验证总数
    pub validations_total: u64,
    /// 验证成功率
    pub validation_success_rate: f64,
}

/// API错误响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 错误详情
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// 请求ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// 时间戳
    pub timestamp: String,
}

impl ApiErrorResponse {
    /// 创建新的API错误响应
    pub fn new(code: String, message: String) -> Self {
        Self {
            code,
            message,
            details: None,
            request_id: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// 带详情的错误响应
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
    
    /// 带请求ID的错误响应
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
}