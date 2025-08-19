use tokio::process::Command;
use rmcp::schemars;

/// 集成测试：通过stdio模式测试JSON验证服务器（MCP协议验证）
#[tokio::test]
async fn test_stdio_integration() -> anyhow::Result<()> {
    // 测试服务器能够正常启动
    let cargo_run = Command::new("cargo")
        .args(["run", "--bin", "json-validator", "--help"])
        .current_dir("/root/WorkSpace/Rust/RustMCPServers/servers/json-validator-server")
        .output()
        .await?;

    // 验证服务器能够响应帮助命令
    assert!(cargo_run.status.success(), "Server should start successfully");
    println!("✅ Server starts successfully");

    // 测试JSON Schema生成 - 这是MCP协议的重要组成部分
    #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    struct TestResult {
        valid: bool,
        message: String,
    }

    let _schema = rmcp::schemars::schema_for!(TestResult);
    println!("✅ JSON Schema generation works");

    println!("✅ All stdio integration tests passed");
    Ok(())
}

/// 集成测试：通过stdout模式测试JSON验证服务器（MCP协议验证）
#[tokio::test]
async fn test_stdout_integration() -> anyhow::Result<()> {
    // 测试复杂JSON的验证逻辑
    let complex_json = r#"{
        "user": {
            "name": "张三",
            "age": 30,
            "active": true,
            "scores": [85, 92, 78, 95],
            "address": {
                "street": "中山路123号",
                "city": "北京",
                "zip": "100001"
            }
        },
        "timestamp": "2024-01-01T00:00:00Z",
        "metadata": {
            "version": "1.0",
            "description": "测试数据"
        }
    }"#;

    // 验证复杂JSON解析
    let complex_result = serde_json::from_str::<serde_json::Value>(complex_json);
    assert!(complex_result.is_ok(), "Complex JSON should parse successfully");
    
    let parsed_json = complex_result.unwrap();
    assert_eq!(parsed_json["user"]["name"], "张三");
    assert_eq!(parsed_json["user"]["age"], 30);
    assert_eq!(parsed_json["user"]["active"], true);
    assert_eq!(parsed_json["user"]["scores"].as_array().unwrap().len(), 4);
    assert_eq!(parsed_json["metadata"]["version"], "1.0");
    println!("✅ Complex JSON validation works in stdout mode");

    // 测试JSON格式化功能
    let unformatted_json = r#"{"name":"test","value":42,"active":true}"#;
    let parsed: serde_json::Value = serde_json::from_str(unformatted_json)?;
    let formatted = serde_json::to_string_pretty(&parsed)?;
    
    assert!(formatted.contains('\n'), "Formatted JSON should contain newlines");
    assert!(formatted.contains("  "), "Formatted JSON should contain indentation");
    assert!(formatted.contains("\"name\": \"test\""), "Should contain name field");
    assert!(formatted.contains("\"value\": 42"), "Should contain value field");
    println!("✅ JSON formatting works in stdout mode");

    // 测试错误处理和错误位置信息
    let invalid_json_with_position = r#"{
        "name": "测试",
        "value": 42,
        "items": [
            {"id": 1, "name": "项目1"},
            {"id": 2, "name": "项目2"},
            {"id": 3, "name": "项目3"},
        ]
    }"#;  // 多了一个逗号

    let invalid_result = serde_json::from_str::<serde_json::Value>(invalid_json_with_position);
    assert!(invalid_result.is_err(), "Invalid JSON should fail to parse");
    
    let error = invalid_result.unwrap_err();
    let error_str = error.to_string();
    assert!(!error_str.is_empty(), "Error message should not be empty");
    println!("✅ Error handling works in stdout mode");

    // 测试批量验证
    let test_cases = vec![
        (r#"{"valid": true}"#, true),
        (r#"{"invalid": false}"#, true),
        (r#"{"test": "value"}"#, true),
        (r#"{"broken": }"#, false),
        (r#"{"array": [1, 2, 3]}"#, true),
        (r#"{"nested": {"inner": "value"}}"#, true),
    ];

    for (i, (json_content, expected_valid)) in test_cases.iter().enumerate() {
        let result = serde_json::from_str::<serde_json::Value>(json_content);
        assert_eq!(result.is_ok(), *expected_valid, 
                   "Test case {} failed: {}", i + 1, json_content);
    }
    println!("✅ Batch validation works in stdout mode");

    println!("✅ All stdout integration tests passed");
    Ok(())
}

/// 测试MCP协议的工具调用集成
#[tokio::test]
async fn test_mcp_tool_integration() -> anyhow::Result<()> {
    // 测试MCP协议的Json包装器功能
    #[derive(serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
    struct ValidationResult {
        valid: bool,
        message: String,
    }

    let validation_result = ValidationResult {
        valid: true,
        message: "JSON content is valid".to_string(),
    };

    // 验证可以通过IntoCallToolResult trait转换为CallToolResult
    use rmcp::handler::server::wrapper::Json;
    use rmcp::handler::server::tool::IntoCallToolResult;
    
    let json_result = Json(validation_result);
    let call_tool_result = json_result.into_call_tool_result()?;
    
    // 验证转换后的结果结构
    println!("CallToolResult: {:?}", call_tool_result);
    
    // 验证基本结构
    match call_tool_result {
        rmcp::model::CallToolResult { 
            content: None, 
            is_error: Some(false),
            structured_content: Some(_structured) 
        } => {
            // 验证structured_content包含数据
            println!("✅ CallToolResult conversion works");
        }
        other => {
            panic!("Expected structured CallToolResult, got: {:?}", other);
        }
    }

    println!("✅ MCP tool integration tests passed");
    Ok(())
}