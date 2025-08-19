use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

fn main() -> std::io::Result<()> {
    println!("Starting JSON Validator HTTP Server (Minimal)");
    
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
    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer)?;
    
    if bytes_read == 0 {
        return Ok(());
    }
    
    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("Received request: {}", request);
    
    // 简单的HTTP请求解析
    let response = if request.starts_with("GET /health") {
        r#"{"status":"healthy","version":"1.0.0","uptime":0,"timestamp":"2025-08-19T12:00:00Z"}"#
    } else if request.starts_with("GET /info") {
        r#"{"name":"JSON Validator HTTP Server","version":"1.0.0","description":"HTTP protocol JSON validation MCP server","capabilities":["validate_json","validate_json_with_schema","validate_json_batch","ping"]}"#
    } else if request.starts_with("POST /rpc") {
        // 简单的JSON-RPC响应
        r#"{"jsonrpc":"2.0","result":{"message":"pong","timestamp":"2025-08-19T12:00:00Z"},"error":null,"id":1}"#
    } else {
        r#"{"error":{"code":404,"message":"Not found"}}"#
    };
    
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response.len(),
        response
    );
    
    stream.write_all(http_response.as_bytes())?;
    stream.flush()?;
    
    Ok(())
}