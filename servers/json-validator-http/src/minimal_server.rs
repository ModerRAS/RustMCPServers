use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    result: Option<T>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct ValidationResult {
    valid: bool,
    errors: Vec<ValidationError>,
    execution_time: u64,
}

#[derive(Debug, Serialize)]
struct ValidationError {
    instance_path: String,
    schema_path: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
    description: String,
    capabilities: Vec<String>,
}

#[derive(Debug, Serialize)]
struct PingResponse {
    message: String,
    timestamp: String,
}

fn main() -> std::io::Result<()> {
    println!("Starting JSON Validator HTTP Server (Simplified)");
    
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on 127.0.0.1:8080");
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    if let Err(e) = handle_client(stream) {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
    
    Ok(())
}

fn handle_client(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    
    if bytes_read == 0 {
        return Ok(());
    }
    
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request: {}", request);
    
    // 简单的HTTP请求解析
    let response = if request.starts_with("GET /health") {
        handle_health_check()
    } else if request.starts_with("GET /info") {
        handle_server_info()
    } else if request.starts_with("POST /rpc") {
        // 提取JSON-RPC请求体
        if let Some(body_start) = request.find("\r\n\r\n") {
            let body = &request[body_start + 4..];
            if let Ok(rpc_request) = serde_json::from_str::<JsonRpcRequest>(body) {
                handle_rpc_request(rpc_request)
            } else {
                create_error_response(400, "Invalid JSON-RPC request")
            }
        } else {
            create_error_response(400, "Missing request body")
        }
    } else {
        create_error_response(404, "Not found")
    };
    
    let response_str = serde_json::to_string_pretty(&response).unwrap();
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_str.len(),
        response_str
    );
    
    stream.write_all(http_response.as_bytes())?;
    stream.flush()?;
    
    Ok(())
}

fn handle_health_check() -> Value {
    json!({
        "status": "healthy",
        "version": "1.0.0",
        "uptime": 0,
        "timestamp": "2025-08-19T12:00:00Z"
    })
}

fn handle_server_info() -> Value {
    json!({
        "name": "JSON Validator HTTP Server",
        "version": "1.0.0",
        "description": "HTTP protocol JSON validation MCP server",
        "capabilities": [
            "validate_json",
            "validate_json_with_schema",
            "validate_json_batch",
            "ping"
        ]
    })
}

fn handle_rpc_request(request: JsonRpcRequest) -> Value {
    if request.jsonrpc != "2.0" {
        return json!({
            "jsonrpc": "2.0",
            "result": null,
            "error": {
                "code": -32600,
                "message": "Invalid JSON-RPC version"
            },
            "id": request.id
        });
    }
    
    let result = match request.method.as_str() {
        "ping" => handle_ping(),
        "validate_json" => handle_validate_json(request.params),
        "validate_json_with_schema" => handle_validate_json_with_schema(request.params),
        "validate_json_batch" => handle_validate_json_batch(request.params),
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };
    
    match result {
        Ok(result_value) => json!({
            "jsonrpc": "2.0",
            "result": result_value,
            "error": null,
            "id": request.id
        }),
        Err(error) => json!({
            "jsonrpc": "2.0",
            "result": null,
            "error": error,
            "id": request.id
        }),
    }
}

fn handle_ping() -> Result<Value, JsonRpcError> {
    Ok(json!({
        "message": "pong",
        "timestamp": "2025-08-19T12:00:00Z"
    }))
}

fn handle_validate_json(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;
    
    let json_data = params.get("json_data").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing json_data parameter".to_string(),
        data: None,
    })?;
    
    let mut errors = Vec::new();
    
    // 基本JSON格式检查
    if let Some(obj) = json_data.as_object() {
        if obj.contains_key("name") && !obj["name"].is_string() {
            errors.push(ValidationError {
                instance_path: "/name".to_string(),
                schema_path: "/properties/name/type".to_string(),
                message: "name must be a string".to_string(),
            });
        }
        
        if obj.contains_key("age") && !obj["age"].is_number() {
            errors.push(ValidationError {
                instance_path: "/age".to_string(),
                schema_path: "/properties/age/type".to_string(),
                message: "age must be a number".to_string(),
            });
        }
    }
    
    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "execution_time": 1
    }))
}

fn handle_validate_json_with_schema(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;
    
    let json_data = params.get("json_data").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing json_data parameter".to_string(),
        data: None,
    })?;
    
    let schema = params.get("schema").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing schema parameter".to_string(),
        data: None,
    })?;
    
    let mut errors = Vec::new();
    
    // 简化的schema验证
    if let Some(schema_obj) = schema.as_object() {
        if let Some(expected_type) = schema_obj.get("type") {
            if let Some(data_obj) = json_data.as_object() {
                for (key, value) in data_obj {
                    if let Some(properties) = schema_obj.get("properties") {
                        if let Some(props) = properties.as_object() {
                            if let Some(prop_schema) = props.get(key) {
                                if let Some(type_constraint) = prop_schema.get("type") {
                                    if type_constraint == "string" && !value.is_string() {
                                        errors.push(ValidationError {
                                            instance_path: format!("/{}", key),
                                            schema_path: format!("/properties/{}/type", key),
                                            message: format!("{} must be a string", key),
                                        });
                                    } else if type_constraint == "number" && !value.is_number() {
                                        errors.push(ValidationError {
                                            instance_path: format!("/{}", key),
                                            schema_path: format!("/properties/{}/type", key),
                                            message: format!("{} must be a number", key),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(json!({
        "valid": errors.is_empty(),
        "errors": errors,
        "execution_time": 2
    }))
}

fn handle_validate_json_batch(params: Option<Value>) -> Result<Value, JsonRpcError> {
    let params = params.ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Invalid params".to_string(),
        data: None,
    })?;
    
    let items = params.get("items").ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "Missing items parameter".to_string(),
        data: None,
    })?;
    
    let items_array = items.as_array().ok_or_else(|| JsonRpcError {
        code: -32602,
        message: "items must be an array".to_string(),
        data: None,
    })?;
    
    let mut results = Vec::new();
    
    for (index, item) in items_array.iter().enumerate() {
        let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or(&index.to_string());
        let json_data = item.get("json_data");
        
        if let Some(data) = json_data {
            let result = handle_validate_json(Some(json!({"json_data": data})));
            match result {
                Ok(validation_result) => {
                    results.push(json!({
                        "id": item_id,
                        "result": validation_result
                    }));
                }
                Err(_) => {
                    results.push(json!({
                        "id": item_id,
                        "error": {
                            "code": -32603,
                            "message": "Internal error"
                        }
                    }));
                }
            }
        } else {
            results.push(json!({
                "id": item_id,
                "error": {
                    "code": -32602,
                    "message": "Missing json_data"
                }
            }));
        }
    }
    
    Ok(json!({
        "results": results,
        "total": results.len()
    }))
}

fn create_error_response(status_code: u32, message: &str) -> Value {
    json!({
        "error": {
            "code": status_code,
            "message": message
        }
    })
}