use duckduckgo_mcp_server::{
    auth::AuthState,
    client::{EnhancedDuckDuckGoClient, SearchRequest},
    config::ServerConfig,
    mcp_types::{CallToolRequest, McpRequest},
};

#[tokio::test]
async fn test_config_from_env() {
    std::env::set_var("HOST", "0.0.0.0");
    std::env::set_var("PORT", "8080");
    std::env::set_var("LOG_LEVEL", "debug");

    let config = ServerConfig::from_env();

    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert_eq!(config.log_level, "debug");

    // Clean up
    std::env::remove_var("HOST");
    std::env::remove_var("PORT");
    std::env::remove_var("LOG_LEVEL");
}

#[tokio::test]
async fn test_search_request_validation() {
    let request = SearchRequest {
        query: "test query".to_string(),
        max_results: 5,
        region: Some("us".to_string()),
        time_filter: Some("d".to_string()),
        safe_search: Some(true),
    };

    assert_eq!(request.query, "test query");
    assert_eq!(request.max_results, 5);
    assert_eq!(request.region.unwrap(), "us");
    assert_eq!(request.time_filter.unwrap(), "d");
    assert!(request.safe_search.unwrap());
}

#[tokio::test]
async fn test_search_request_defaults() {
    let request = SearchRequest {
        query: "test".to_string(),
        max_results: 10,
        region: None,
        time_filter: None,
        safe_search: None,
    };

    assert_eq!(request.max_results, 10);
    assert!(request.region.is_none());
    assert!(request.time_filter.is_none());
    assert!(request.safe_search.is_none());
}

#[tokio::test]
async fn test_auth_service_token_generation() {
    let config = ServerConfig::default();
    let auth_service = AuthState::new(config.secret_key.clone(), config.require_auth);

    let token = auth_service.generate_token("test_user").unwrap();
    assert!(!token.is_empty());

    let validation = auth_service.validate_token(&token).await;
    assert!(validation.is_ok());
    assert!(validation.unwrap());
}

#[tokio::test]
async fn test_client_cache_functionality() {
    let config = ServerConfig::default();
    let client = EnhancedDuckDuckGoClient::new(config);

    // Test cache stats
    let (count, weight) = client.cache_stats();
    assert_eq!(count, 0);
    assert_eq!(weight, 0);

    // Test cache clear
    client.clear_cache().await;
    let (count, weight) = client.cache_stats();
    assert_eq!(count, 0);
    assert_eq!(weight, 0);
}

#[tokio::test]
async fn test_mcp_request_serialization() {
    let request = McpRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::Value::Number(1.into())),
        method: "tools/list".to_string(),
        params: Some(serde_json::Value::Null),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"jsonrpc\":\"2.0\""));
    assert!(serialized.contains("\"method\":\"tools/list\""));

    let deserialized: McpRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.jsonrpc, "2.0");
    assert_eq!(deserialized.method, "tools/list");
}

#[tokio::test]
async fn test_call_tool_request_serialization() {
    let request = CallToolRequest {
        name: "search".to_string(),
        arguments: Some(serde_json::json!({
            "query": "test query",
            "max_results": 5
        })),
    };

    let serialized = serde_json::to_string(&request).unwrap();
    assert!(serialized.contains("\"name\":\"search\""));
    assert!(serialized.contains("\"query\":\"test query\""));

    let deserialized: CallToolRequest = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.name, "search");
}

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = ServerConfig::default();

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3000);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.rate_limit_per_minute, 60);
        assert_eq!(config.cache_ttl_seconds, 300);
        assert_eq!(config.max_search_results, 20);
    }

    #[test]
    fn test_config_env_parsing() {
        std::env::set_var("STATIC_TOKENS", "token1,token2,token3");

        let config = ServerConfig::from_env();

        assert!(config.static_tokens.contains(&"token1".to_string()));
        assert!(config.static_tokens.contains(&"token2".to_string()));
        assert!(config.static_tokens.contains(&"token3".to_string()));

        std::env::remove_var("STATIC_TOKENS");
    }
}
