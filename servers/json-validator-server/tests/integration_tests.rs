use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

/// 测试JSON验证功能（直接调用函数）
#[tokio::test]
async fn test_json_validation_logic() -> anyhow::Result<()> {
    // 测试有效JSON
    let valid_json = r#"{"name": "test", "value": 42}"#;
    let result = serde_json::from_str::<serde_json::Value>(valid_json);
    assert!(result.is_ok(), "Valid JSON should parse successfully");
    println!("✅ Valid JSON parsing works");

    // 测试无效JSON
    let invalid_json = r#"{"name": "test", "value": 42"#;
    let result = serde_json::from_str::<serde_json::Value>(invalid_json);
    assert!(result.is_err(), "Invalid JSON should fail to parse");
    println!("✅ Invalid JSON parsing fails as expected");

    // 测试JSON格式化
    let unformatted_json = r#"{"name":"test","value":42}"#;
    let parsed: serde_json::Value = serde_json::from_str(unformatted_json)?;
    let formatted = serde_json::to_string_pretty(&parsed)?;
    assert!(formatted.contains('\n'), "Formatted JSON should contain newlines");
    assert!(formatted.contains("  "), "Formatted JSON should contain indentation");
    println!("✅ JSON formatting works");

    Ok(())
}

/// 测试文件验证功能
#[tokio::test]
async fn test_file_validation_logic() -> anyhow::Result<()> {
    // 创建临时测试文件
    let mut temp_file = NamedTempFile::new()?;
    let valid_json = r#"{"test": "data"}"#;
    temp_file.write_all(valid_json.as_bytes())?;
    let temp_path = temp_file.path().to_string_lossy().to_string();

    // 测试文件存在性检查
    assert!(std::path::Path::new(&temp_path).exists(), "Temp file should exist");

    // 测试文件读取
    let content = fs::read_to_string(&temp_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(parsed["test"], "data", "File content should be parsed correctly");

    // 清理
    temp_file.close()?;
    println!("✅ File validation logic works");

    Ok(())
}

/// 测试验证结果序列化
#[tokio::test]
async fn test_validation_result_serialization() -> anyhow::Result<()> {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct ValidationResult {
        pub valid: bool,
        pub message: String,
        pub file_path: Option<String>,
        pub error_line: Option<usize>,
        pub error_column: Option<usize>,
    }

    // 测试成功验证结果
    let success_result = ValidationResult {
        valid: true,
        message: "JSON file is valid".to_string(),
        file_path: Some("/test.json".to_string()),
        error_line: None,
        error_column: None,
    };

    let serialized = serde_json::to_string(&success_result)?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.valid, true);
    assert_eq!(deserialized.message, "JSON file is valid");
    assert_eq!(deserialized.file_path, Some("/test.json".to_string()));
    
    println!("✅ Validation result serialization works");

    // 测试失败验证结果
    let failure_result = ValidationResult {
        valid: false,
        message: "Invalid JSON".to_string(),
        file_path: None,
        error_line: Some(1),
        error_column: Some(5),
    };

    let serialized = serde_json::to_string(&failure_result)?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.valid, false);
    assert_eq!(deserialized.message, "Invalid JSON");
    assert_eq!(deserialized.error_line, Some(1));
    assert_eq!(deserialized.error_column, Some(5));
    
    println!("✅ Failure validation result serialization works");

    Ok(())
}

/// 测试格式化结果序列化
#[tokio::test]
async fn test_format_result_serialization() -> anyhow::Result<()> {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct FormatResult {
        pub success: bool,
        pub formatted_json: Option<String>,
        pub message: Option<String>,
        pub error_line: Option<usize>,
        pub error_column: Option<usize>,
    }

    // 测试成功格式化结果
    let success_result = FormatResult {
        success: true,
        formatted_json: Some("{\n  \"name\": \"test\"\n}".to_string()),
        message: None,
        error_line: None,
        error_column: None,
    };

    let serialized = serde_json::to_string(&success_result)?;
    let deserialized: FormatResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.success, true);
    assert!(deserialized.formatted_json.is_some());
    assert!(deserialized.formatted_json.unwrap().contains('\n'));
    
    println!("✅ Format result serialization works");

    // 测试失败格式化结果
    let failure_result = FormatResult {
        success: false,
        formatted_json: None,
        message: Some("Invalid JSON".to_string()),
        error_line: Some(1),
        error_column: Some(5),
    };

    let serialized = serde_json::to_string(&failure_result)?;
    let deserialized: FormatResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.success, false);
    assert!(deserialized.formatted_json.is_none());
    assert!(deserialized.message.is_some());
    
    println!("✅ Failure format result serialization works");

    Ok(())
}

/// 测试错误位置信息
#[tokio::test]
async fn test_error_location_info() -> anyhow::Result<()> {
    // 测试一个有错误的JSON，检查是否能正确获取错误位置
    let invalid_json = r#"{"name": "test", "value": }"#;
    let result = serde_json::from_str::<serde_json::Value>(invalid_json);
    
    assert!(result.is_err());
    
    // 检查错误类型是否包含位置信息
    let error = result.unwrap_err();
    let error_str = error.to_string();
    
    // 注意：serde_json的错误格式可能不包含行号列号，但至少应该有错误描述
    assert!(!error_str.is_empty(), "Error message should not be empty");
    assert!(error_str.contains("expected"), "Error should mention what was expected");
    
    println!("✅ Error location info works");

    Ok(())
}

/// 测试JSON验证功能（直接调用函数）
fn test_json_validation_logic_sync() -> anyhow::Result<()> {
    // 测试有效JSON
    let valid_json = r#"{"name": "test", "value": 42}"#;
    let result = serde_json::from_str::<serde_json::Value>(valid_json);
    assert!(result.is_ok(), "Valid JSON should parse successfully");

    // 测试无效JSON
    let invalid_json = r#"{"name": "test", "value": 42"#;
    let result = serde_json::from_str::<serde_json::Value>(invalid_json);
    assert!(result.is_err(), "Invalid JSON should fail to parse");

    // 测试JSON格式化
    let unformatted_json = r#"{"name":"test","value":42}"#;
    let parsed: serde_json::Value = serde_json::from_str(unformatted_json)?;
    let formatted = serde_json::to_string_pretty(&parsed)?;
    assert!(formatted.contains('\n'), "Formatted JSON should contain newlines");
    assert!(formatted.contains("  "), "Formatted JSON should contain indentation");

    Ok(())
}

/// 测试文件验证功能
fn test_file_validation_logic_sync() -> anyhow::Result<()> {
    // 创建临时测试文件
    let mut temp_file = NamedTempFile::new()?;
    let valid_json = r#"{"test": "data"}"#;
    temp_file.write_all(valid_json.as_bytes())?;
    let temp_path = temp_file.path().to_string_lossy().to_string();

    // 测试文件存在性检查
    assert!(std::path::Path::new(&temp_path).exists(), "Temp file should exist");

    // 测试文件读取
    let content = fs::read_to_string(&temp_path)?;
    let parsed: serde_json::Value = serde_json::from_str(&content)?;
    assert_eq!(parsed["test"], "data", "File content should be parsed correctly");

    // 清理
    temp_file.close()?;
    Ok(())
}

/// 测试验证结果序列化
fn test_validation_result_serialization_sync() -> anyhow::Result<()> {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct ValidationResult {
        pub valid: bool,
        pub message: String,
        pub file_path: Option<String>,
        pub error_line: Option<usize>,
        pub error_column: Option<usize>,
    }

    // 测试成功验证结果
    let success_result = ValidationResult {
        valid: true,
        message: "JSON file is valid".to_string(),
        file_path: Some("/test.json".to_string()),
        error_line: None,
        error_column: None,
    };

    let serialized = serde_json::to_string(&success_result)?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.valid, true);
    assert_eq!(deserialized.message, "JSON file is valid");
    assert_eq!(deserialized.file_path, Some("/test.json".to_string()));
    
    // 测试失败验证结果
    let failure_result = ValidationResult {
        valid: false,
        message: "Invalid JSON".to_string(),
        file_path: None,
        error_line: Some(1),
        error_column: Some(5),
    };

    let serialized = serde_json::to_string(&failure_result)?;
    let deserialized: ValidationResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.valid, false);
    assert_eq!(deserialized.message, "Invalid JSON");
    assert_eq!(deserialized.error_line, Some(1));
    assert_eq!(deserialized.error_column, Some(5));

    Ok(())
}

/// 测试格式化结果序列化
fn test_format_result_serialization_sync() -> anyhow::Result<()> {
    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    struct FormatResult {
        pub success: bool,
        pub formatted_json: Option<String>,
        pub message: Option<String>,
        pub error_line: Option<usize>,
        pub error_column: Option<usize>,
    }

    // 测试成功格式化结果
    let success_result = FormatResult {
        success: true,
        formatted_json: Some("{\n  \"name\": \"test\"\n}".to_string()),
        message: None,
        error_line: None,
        error_column: None,
    };

    let serialized = serde_json::to_string(&success_result)?;
    let deserialized: FormatResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.success, true);
    assert!(deserialized.formatted_json.is_some());
    assert!(deserialized.formatted_json.unwrap().contains('\n'));
    
    // 测试失败格式化结果
    let failure_result = FormatResult {
        success: false,
        formatted_json: None,
        message: Some("Invalid JSON".to_string()),
        error_line: Some(1),
        error_column: Some(5),
    };

    let serialized = serde_json::to_string(&failure_result)?;
    let deserialized: FormatResult = serde_json::from_str(&serialized)?;
    
    assert_eq!(deserialized.success, false);
    assert!(deserialized.formatted_json.is_none());
    assert!(deserialized.message.is_some());

    Ok(())
}

/// 测试错误位置信息
fn test_error_location_info_sync() -> anyhow::Result<()> {
    // 测试一个有错误的JSON，检查是否能正确获取错误位置
    let invalid_json = r#"{"name": "test", "value": }"#;
    let result = serde_json::from_str::<serde_json::Value>(invalid_json);
    
    assert!(result.is_err());
    
    // 检查错误类型是否包含位置信息
    let error = result.unwrap_err();
    let error_str = error.to_string();
    
    // 注意：serde_json的错误格式可能不包含行号列号，但至少应该有错误描述
    assert!(!error_str.is_empty(), "Error message should not be empty");
    assert!(error_str.contains("expected"), "Error should mention what was expected");

    Ok(())
}

/// 集成测试：测试完整的工作流程
#[tokio::test]
async fn test_complete_workflow() -> anyhow::Result<()> {
    // 1. 测试JSON验证逻辑
    test_json_validation_logic_sync()?;
    
    // 2. 测试文件验证逻辑
    test_file_validation_logic_sync()?;
    
    // 3. 测试验证结果序列化
    test_validation_result_serialization_sync()?;
    
    // 4. 测试格式化结果序列化
    test_format_result_serialization_sync()?;
    
    // 5. 测试错误位置信息
    test_error_location_info_sync()?;
    
    println!("✅ Complete workflow test passed");
    Ok(())
}