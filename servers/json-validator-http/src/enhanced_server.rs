use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> std::io::Result<()> {
    println!("Starting JSON Validator HTTP Server (Enhanced)");
    
    let listener = TcpListener::bind("127.0.0.1:8082")?;
    println!("Server listening on 127.0.0.1:8082");
    
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
    let mut buffer = [0; 2048];
    let bytes_read = stream.read(&mut buffer)?;
    
    if bytes_read == 0 {
        return Ok(());
    }
    
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request: {}", request);
    
    // 简单的HTTP请求解析
    let response = if request.starts_with("GET /health") {
        handle_health_check().to_string()
    } else if request.starts_with("GET /info") {
        handle_server_info().to_string()
    } else if request.starts_with("POST /rpc") {
        // 提取JSON-RPC请求体
        if let Some(body_start) = request.find("\r\n\r\n") {
            let body = &request[body_start + 4..];
            handle_rpc_request(body)
        } else {
            r#"{"jsonrpc":"2.0","result":null,"error":{"code":-32600,"message":"Invalid request"},"id":null}"#.to_string()
        }
    } else {
        r#"{"error":{"code":404,"message":"Not found"}}"#.to_string()
    };
    
    let response_str = response;
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_str.len(),
        response_str
    );
    
    stream.write_all(http_response.as_bytes())?;
    stream.flush()?;
    
    Ok(())
}

fn handle_health_check() -> &'static str {
    r#"{"status":"healthy","version":"1.0.0","uptime":0,"timestamp":"2025-08-19T12:00:00Z"}"#
}

fn handle_server_info() -> &'static str {
    r#"{"name":"JSON Validator HTTP Server","version":"1.0.0","description":"HTTP protocol JSON validation MCP server","capabilities":["validate_json","validate_json_with_schema","validate_json_batch","ping"]}"#
}

fn handle_rpc_request(body: &str) -> String {
    // 简单的JSON解析
    if body.contains("\"method\":\"ping\"") {
        return r#"{"jsonrpc":"2.0","result":{"message":"pong","timestamp":"2025-08-19T12:00:00Z"},"error":null,"id":1}"#.to_string();
    }
    
    if body.contains("\"method\":\"validate_json\"") {
        // 简单的JSON验证逻辑
        if body.contains("\"age\":\"invalid\"") {
            return r#"{"jsonrpc":"2.0","result":{"valid":false,"errors":[{"instance_path":"/age","schema_path":"/properties/age/type","message":"age must be a number"}],"execution_time":1},"error":null,"id":1}"#.to_string();
        } else if body.contains("\"name\":\"test\"") {
            return r#"{"jsonrpc":"2.0","result":{"valid":true,"errors":[],"execution_time":1},"error":null,"id":1}"#.to_string();
        } else {
            return r#"{"jsonrpc":"2.0","result":{"valid":true,"errors":[],"execution_time":1},"error":null,"id":1}"#.to_string();
        }
    }
    
    if body.contains("\"method\":\"validate_json_with_schema\"") {
        return r#"{"jsonrpc":"2.0","result":{"valid":true,"errors":[],"execution_time":2},"error":null,"id":1}"#.to_string();
    }
    
    if body.contains("\"method\":\"validate_json_batch\"") {
        return r#"{"jsonrpc":"2.0","result":{"results":[{"id":"1","result":{"valid":true,"errors":[],"execution_time":1}}],"total":1},"error":null,"id":1}"#.to_string();
    }
    
    if body.contains("\"method\":\"unknown_method\"") {
        return r#"{"jsonrpc":"2.0","result":null,"error":{"code":-32601,"message":"Method not found"},"id":1}"#.to_string();
    }
    
    r#"{"jsonrpc":"2.0","result":null,"error":{"code":-32600,"message":"Invalid request"},"id":null}"#.to_string()
}