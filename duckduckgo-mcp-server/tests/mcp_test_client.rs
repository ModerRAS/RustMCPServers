use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

/// Simple MCP client for testing protocol compliance
pub struct McpTestClient {
    client: Client,
    base_url: String,
}

impl McpTestClient {
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self { client, base_url }
    }

    /// Test MCP initialize endpoint
    pub async fn test_initialize(&self) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}/mcp/initialize", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "test-client",
                        "version": "1.0.0"
                    }
                }
            }))
            .send()
            .await?;

        let status = response.status();
        let body: Value = response.json().await?;

        println!("Initialize response status: {status}");
        println!(
            "Initialize response body: {}",
            serde_json::to_string_pretty(&body)?
        );

        // Validate JSON-RPC 2.0 structure
        assert_eq!(body["jsonrpc"], "2.0", "Response should have jsonrpc field");
        assert_eq!(body["id"], 1, "Response should have matching id");
        assert!(
            body["result"].is_object(),
            "Response should have result object"
        );

        Ok(body)
    }

    /// Test MCP tools/list endpoint
    pub async fn test_list_tools(&self) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}/mcp/tools/list", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list"
            }))
            .send()
            .await?;

        let status = response.status();
        let body: Value = response.json().await?;

        println!("List tools response status: {status}");
        println!(
            "List tools response body: {}",
            serde_json::to_string_pretty(&body)?
        );

        // Validate structure
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["id"], 2);
        assert!(
            body["result"]["tools"].is_array(),
            "Should return tools array"
        );

        let tools = body["result"]["tools"].as_array().unwrap();
        assert!(!tools.is_empty(), "Should have at least one tool");

        for tool in tools {
            assert!(tool["name"].is_string(), "Tool should have name");
            assert!(
                tool["description"].is_string(),
                "Tool should have description"
            );
            assert!(
                tool["inputSchema"].is_object(),
                "Tool should have inputSchema"
            );
        }

        Ok(body)
    }

    /// Test MCP tools/call endpoint with search
    pub async fn test_search_call(&self, query: &str, max_results: usize) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}/mcp/tools/call", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "search",
                    "arguments": {
                        "query": query,
                        "max_results": max_results
                    }
                }
            }))
            .send()
            .await?;

        let status = response.status();
        let body: Value = response.json().await?;

        println!("Search call response status: {status}");
        println!(
            "Search call response body: {}",
            serde_json::to_string_pretty(&body)?
        );

        // Validate structure
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["id"], 3);
        assert!(
            body["result"]["content"].is_array(),
            "Should return content array"
        );

        let content = body["result"]["content"].as_array().unwrap();
        if !content.is_empty() {
            assert!(content[0]["type"].is_string(), "Content should have type");
            assert!(content[0]["text"].is_string(), "Content should have text");
        }

        Ok(body)
    }

    /// Test MCP ping endpoint
    pub async fn test_ping(&self) -> Result<Value> {
        let response = self
            .client
            .post(format!("{}/mcp/ping", self.base_url))
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "ping"
            }))
            .send()
            .await?;

        let status = response.status();
        let body: Value = response.json().await?;

        println!("Ping response status: {status}");
        println!(
            "Ping response body: {}",
            serde_json::to_string_pretty(&body)?
        );

        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["id"], 4);
        assert_eq!(body["result"]["pong"], true);

        Ok(body)
    }

    /// Run all tests in sequence
    pub async fn run_all_tests(&self) -> Result<()> {
        println!("=== Starting MCP Protocol Tests ===");

        println!("\n1. Testing initialize...");
        self.test_initialize().await?;

        println!("\n2. Testing tools/list...");
        self.test_list_tools().await?;

        println!("\n3. Testing search call...");
        self.test_search_call("MCP protocol", 3).await?;

        println!("\n4. Testing ping...");
        self.test_ping().await?;

        println!("\n=== All MCP Protocol Tests Passed ===");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use tokio::task::JoinHandle;

    async fn spawn_test_server() -> (SocketAddr, JoinHandle<()>) {
        use duckduckgo_mcp_server::{
            config::ServerConfig,
            mcp_handler::{
                handle_call_tool, handle_initialize, handle_list_tools, handle_ping, McpState,
            },
        };
        use std::sync::Arc;

        let config = ServerConfig::default();
        let state = Arc::new(McpState::new(config).await);

        let app = axum::Router::new()
            .route("/mcp/initialize", axum::routing::post(handle_initialize))
            .route("/mcp/tools/list", axum::routing::post(handle_list_tools))
            .route("/mcp/tools/call", axum::routing::post(handle_call_tool))
            .route("/mcp/ping", axum::routing::post(handle_ping))
            .with_state(state);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        (addr, server)
    }

    #[tokio::test]
    async fn test_client_against_real_server() {
        let (addr, server) = spawn_test_server().await;

        // Give server a moment to start
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = McpTestClient::new(format!("http://{addr}"));

        let result = client.run_all_tests().await;

        // Clean up
        server.abort();

        result.expect("MCP client tests should pass");
    }
}
