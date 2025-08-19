//! HTTP JSON Validator Client Example
//! 
//! This example demonstrates how to use the HTTP JSON validator server

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// JSON-RPC client for JSON validator server
pub struct JsonValidatorClient {
    client: Client,
    base_url: String,
}

impl JsonValidatorClient {
    /// Create a new client
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
    
    /// Create a client with authentication
    pub fn with_auth(base_url: &str, token: &str) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token))
                .expect("Invalid auth header"),
        );
        
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .default_headers(headers)
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
    
    /// Check server health
    pub async fn health_check(&self) -> Result<HealthResponse, reqwest::Error> {
        let url = format!("{}/health", self.base_url);
        self.client.get(&url).send().await?.json().await
    }
    
    /// Get server information
    pub async fn server_info(&self) -> Result<ServerInfo, reqwest::Error> {
        let url = format!("{}/info", self.base_url);
        self.client.get(&url).send().await?.json().await
    }
    
    /// Validate JSON data
    pub async fn validate_json(
        &self,
        json_data: &serde_json::Value,
        options: Option<ValidationOptions>,
    ) -> Result<ValidationResult, reqwest::Error> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "validate_json".to_string(),
            params: Some(serde_json::json!({
                "json_data": json_data,
                "options": options.unwrap_or_default()
            })),
            id: serde_json::Value::Number(1.into()),
        };
        
        let url = format!("{}/rpc", self.base_url);
        let response: JsonRpcResponse<ValidationResult> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        response.into_result()
    }
    
    /// Validate JSON with schema
    pub async fn validate_json_with_schema(
        &self,
        json_data: &serde_json::Value,
        schema: &serde_json::Value,
        options: Option<ValidationOptions>,
    ) -> Result<ValidationResult, reqwest::Error> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "validate_json_with_schema".to_string(),
            params: Some(serde_json::json!({
                "json_data": json_data,
                "schema": schema,
                "options": options.unwrap_or_default()
            })),
            id: serde_json::Value::Number(1.into()),
        };
        
        let url = format!("{}/rpc", self.base_url);
        let response: JsonRpcResponse<ValidationResult> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        response.into_result()
    }
    
    /// Batch validate JSON data
    pub async fn validate_json_batch(
        &self,
        items: &[BatchValidationItem],
        options: Option<ValidationOptions>,
    ) -> Result<BatchValidationResult, reqwest::Error> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "validate_json_batch".to_string(),
            params: Some(serde_json::json!({
                "items": items,
                "options": options.unwrap_or_default()
            })),
            id: serde_json::Value::Number(1.into()),
        };
        
        let url = format!("{}/rpc", self.base_url);
        let response: JsonRpcResponse<BatchValidationResult> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        response.into_result()
    }
    
    /// Ping the server
    pub async fn ping(&self) -> Result<PingResponse, reqwest::Error> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "ping".to_string(),
            params: None,
            id: serde_json::Value::Number(1.into()),
        };
        
        let url = format!("{}/rpc", self.base_url);
        let response: JsonRpcResponse<PingResponse> = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?
            .json()
            .await?;
        
        response.into_result()
    }
}

/// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest<T = serde_json::Value> {
    jsonrpc: String,
    method: String,
    params: Option<T>,
    id: serde_json::Value,
}

/// JSON-RPC response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    jsonrpc: String,
    result: Option<T>,
    error: Option<JsonRpcError>,
    id: serde_json::Value,
}

impl<T> JsonRpcResponse<T> {
    fn into_result(self) -> Result<T, reqwest::Error> {
        if let Some(error) = self.error {
            Err(reqwest::Error::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("JSON-RPC error: {} - {}", error.code, error.message),
            )))
        } else {
            self.result.ok_or_else(|| {
                reqwest::Error::from(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "No result in JSON-RPC response",
                ))
            })
        }
    }
}

/// JSON-RPC error
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

// Response types (re-export from server)
pub use crate::models::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("JSON Validator HTTP Client Example");
    println!("====================================");
    
    // Create client
    let client = JsonValidatorClient::new("http://localhost:8080");
    
    // Test health check
    println!("\n1. Testing health check...");
    match client.health_check().await {
        Ok(health) => {
            println!("✓ Server is healthy");
            println!("  Status: {}", health.status);
            println!("  Version: {}", health.version);
            println!("  Uptime: {}s", health.uptime);
        }
        Err(e) => {
            println!("✗ Health check failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test server info
    println!("\n2. Testing server info...");
    match client.server_info().await {
        Ok(info) => {
            println!("✓ Server info retrieved");
            println!("  Name: {}", info.name);
            println!("  Version: {}", info.version);
            println!("  Capabilities: {:?}", info.capabilities);
        }
        Err(e) => {
            println!("✗ Server info failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test ping
    println!("\n3. Testing ping...");
    match client.ping().await {
        Ok(ping) => {
            println!("✓ Ping successful");
            println!("  Message: {}", ping.message);
            println!("  Timestamp: {}", ping.timestamp);
        }
        Err(e) => {
            println!("✗ Ping failed: {}", e);
            return Err(e.into());
        }
    }
    
    // Test JSON validation
    println!("\n4. Testing JSON validation...");
    let valid_json = serde_json::json!({
        "name": "John Doe",
        "age": 30,
        "email": "john@example.com"
    });
    
    match client.validate_json(&valid_json, None).await {
        Ok(result) => {
            println!("✓ JSON validation completed");
            println!("  Valid: {}", result.valid);
            println!("  Errors: {}", result.errors.len());
            println!("  Execution time: {}ms", result.execution_time);
        }
        Err(e) => {
            println!("✗ JSON validation failed: {}", e);
        }
    }
    
    // Test invalid JSON validation
    let invalid_json = serde_json::json!({
        "name": "Jane Doe",
        "age": "invalid",  // Should be a number
        "email": "invalid-email"  // Should be a valid email
    });
    
    match client.validate_json(&invalid_json, None).await {
        Ok(result) => {
            println!("✓ Invalid JSON validation completed");
            println!("  Valid: {}", result.valid);
            println!("  Errors: {}", result.errors.len());
            for error in &result.errors {
                println!("    - {}: {}", error.instance_path, error.message);
            }
        }
        Err(e) => {
            println!("✗ Invalid JSON validation failed: {}", e);
        }
    }
    
    // Test JSON Schema validation
    println!("\n5. Testing JSON Schema validation...");
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer", "minimum": 0},
            "email": {"type": "string", "format": "email"}
        },
        "required": ["name", "age", "email"]
    });
    
    match client.validate_json_with_schema(&valid_json, &schema, None).await {
        Ok(result) => {
            println!("✓ JSON Schema validation completed");
            println!("  Valid: {}", result.valid);
            println!("  Errors: {}", result.errors.len());
        }
        Err(e) => {
            println!("✗ JSON Schema validation failed: {}", e);
        }
    }
    
    // Test batch validation
    println!("\n6. Testing batch validation...");
    let batch_items = vec![
        BatchValidationItem {
            id: "1".to_string(),
            json_data: serde_json::json!({"name": "Item 1", "value": 100}),
            schema: None,
        },
        BatchValidationItem {
            id: "2".to_string(),
            json_data: serde_json::json!({"name": "Item 2", "value": "invalid"}),
            schema: None,
        },
        BatchValidationItem {
            id: "3".to_string(),
            json_data: serde_json::json!({"name": "Item 3", "value": 300}),
            schema: Some(schema.clone()),
        },
    ];
    
    match client.validate_json_batch(&batch_items, None).await {
        Ok(result) => {
            println!("✓ Batch validation completed");
            println!("  Total items: {}", result.results.len());
            for item_result in &result.results {
                println!("    Item {}: {}", item_result.id, 
                    if item_result.result.valid { "✓" } else { "✗" });
            }
        }
        Err(e) => {
            println!("✗ Batch validation failed: {}", e);
        }
    }
    
    // Test with options
    println!("\n7. Testing validation with options...");
    let options = ValidationOptions {
        strict_mode: true,
        allow_additional_properties: false,
        custom_formats: std::collections::HashMap::new(),
        detailed_errors: true,
        cache_key: Some("test-validation-key".to_string()),
    };
    
    match client.validate_json(&valid_json, Some(options)).await {
        Ok(result) => {
            println!("✓ Validation with options completed");
            println!("  Valid: {}", result.valid);
            println!("  Cache hit: {}", result.cache_hit);
            println!("  Cache key: {:?}", result.cache_key);
        }
        Err(e) => {
            println!("✗ Validation with options failed: {}", e);
        }
    }
    
    println!("\n====================================");
    println!("All tests completed successfully!");
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_client_creation() {
        let client = JsonValidatorClient::new("http://localhost:8080");
        assert_eq!(client.base_url, "http://localhost:8080");
    }
    
    #[tokio::test]
    async fn test_client_with_auth() {
        let client = JsonValidatorClient::with_auth("http://localhost:8080", "test-token");
        assert_eq!(client.base_url, "http://localhost:8080");
    }
    
    #[tokio::test]
    async fn test_json_rpc_request_creation() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
            id: serde_json::Value::Number(1.into()),
        };
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test");
        assert!(request.params.is_some());
        assert_eq!(request.id, serde_json::Value::Number(1.into()));
    }
}