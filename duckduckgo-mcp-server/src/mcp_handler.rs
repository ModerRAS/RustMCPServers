use crate::auth::AuthState;
use crate::client::{EnhancedDuckDuckGoClient, SearchRequest};
use crate::config::ServerConfig;
use crate::mcp_types::*;
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct McpState {
    pub duckduckgo_client: Arc<EnhancedDuckDuckGoClient>,
    pub config: ServerConfig,
    pub auth: Arc<AuthState>,
}

impl McpState {
    pub async fn new(config: ServerConfig) -> Self {
        Self {
            duckduckgo_client: Arc::new(EnhancedDuckDuckGoClient::new(config.clone())),
            config: config.clone(),
            auth: Arc::new(AuthState::new(config.auth_config().secret_key, config.auth_config().require_auth)),
        }
    }
}

pub async fn handle_initialize(
    State(_state): State<Arc<McpState>>,
    Json(request): Json<McpRequest>,
) -> Result<Json<McpResponse>, StatusCode> {
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": true
                },
                "resources": {
                    "subscribe": false,
                    "listChanged": false
                },
                "prompts": {
                    "listChanged": false
                },
                "logging": {}
            },
            "serverInfo": {
                "name": "duckduckgo-mcp-server",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        error: None,
    };

    Ok(Json(response))
}

pub async fn handle_list_tools(
    State(_state): State<Arc<McpState>>,
    Json(request): Json<McpRequest>,
) -> Result<Json<McpResponse>, StatusCode> {
    let tools = vec![
        Tool {
            name: "search".to_string(),
            description: "Search DuckDuckGo for web results".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 10, max: 20)",
                        "minimum": 1,
                        "maximum": 20
                    },
                    "region": {
                        "type": "string",
                        "description": "Region code for localized results (e.g., 'us', 'uk', 'cn')",
                        "optional": true
                    },
                    "time_filter": {
                        "type": "string",
                        "description": "Time filter for results (e.g., 'd' for day, 'w' for week, 'm' for month, 'y' for year)",
                        "optional": true
                    }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "search_news".to_string(),
            description: "Search DuckDuckGo for news results".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "Maximum number of results to return (default: 10, max: 20)",
                        "minimum": 1,
                        "maximum": 20
                    }
                },
                "required": ["query"]
            }),
        },
    ];

    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({ "tools": tools })),
        error: None,
    };

    Ok(Json(response))
}

pub async fn handle_call_tool(
    State(_state): State<Arc<McpState>>,
    Json(request): Json<McpRequest>,
) -> Result<Json<McpResponse>, StatusCode> {
    let params = request.params.clone().unwrap_or(Value::Null);
    
    let call_tool_request: CallToolRequest = serde_json::from_value(params)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let result = match call_tool_request.name.as_str() {
        "search" => handle_search(&_state, call_tool_request.arguments).await,
        "search_news" => handle_search_news(&_state, call_tool_request.arguments).await,
        _ => {
            let error_response = McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: None,
                error: Some(McpError::method_not_found(format!(
                    "Unknown tool: {}",
                    call_tool_request.name
                ))),
            };
            return Ok(Json(error_response));
        }
    };

    match result {
        Ok(content) => {
            let response = McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({ "content": content })),
                error: None,
            };
            Ok(Json(response))
        }
        Err(e) => {
            let error_response = McpResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: None,
                error: Some(McpError::internal_error(e.to_string())),
            };
            Ok(Json(error_response))
        }
    }
}

async fn handle_search(
    state: &McpState,
    arguments: Option<Value>,
) -> Result<Vec<ToolContent>> {
    let args: SearchRequest = serde_json::from_value(arguments.unwrap_or(Value::Null))
        .map_err(|e| anyhow::anyhow!("Invalid search arguments: {}", e))?;

    let results = state.duckduckgo_client.search(args).await
        .map_err(|e| anyhow::anyhow!("Search failed: {}", e))?;

    let content = ToolContent {
        content_type: "text".to_string(),
        text: serde_json::to_string_pretty(&results)?,
    };

    Ok(vec![content])
}

async fn handle_search_news(
    state: &McpState,
    arguments: Option<Value>,
) -> Result<Vec<ToolContent>> {
    let args: SearchRequest = serde_json::from_value(arguments.unwrap_or(Value::Null))
        .map_err(|e| anyhow::anyhow!("Invalid search arguments: {}", e))?;

    let results = state.duckduckgo_client.search_news(args).await
        .map_err(|e| anyhow::anyhow!("News search failed: {}", e))?;

    let content = ToolContent {
        content_type: "text".to_string(),
        text: serde_json::to_string_pretty(&results)?,
    };

    Ok(vec![content])
}

pub async fn handle_ping(
    Json(request): Json<McpRequest>,
) -> Result<Json<McpResponse>, StatusCode> {
    let response = McpResponse {
        jsonrpc: "2.0".to_string(),
        id: request.id,
        result: Some(json!({ "pong": true })),
        error: None,
    };

    Ok(Json(response))
}
