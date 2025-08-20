//! 验证工具模块

use chrono::Utc;

/// 创建验证上下文
pub fn create_validation_context(
    json_size: usize,
    schema_size: Option<usize>,
    options: &crate::models::ValidationOptions,
) -> serde_json::Value {
    serde_json::json!({
        "json_size": json_size,
        "schema_size": schema_size.unwrap_or(0),
        "strict_mode": options.strict_mode,
        "allow_additional_properties": options.allow_additional_properties,
        "detailed_errors": options.detailed_errors,
        "timestamp": Utc::now().to_rfc3339()
    })
}

/// 创建错误上下文
pub fn create_error_context(
    error_code: &str,
    error_message: &str,
    additional_info: Option<serde_json::Value>,
) -> serde_json::Value {
    let mut context = serde_json::json!({
        "error_code": error_code,
        "error_message": error_message,
        "timestamp": Utc::now().to_rfc3339()
    });
    
    if let Some(info) = additional_info {
        context["additional_info"] = info;
    }
    
    context
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ValidationOptions;

    #[test]
    fn test_validation_context_creation() {
        let options = ValidationOptions::default();
        let context = create_validation_context(1024, Some(512), &options);
        
        assert_eq!(context["json_size"], 1024);
        assert_eq!(context["schema_size"], 512);
        assert_eq!(context["strict_mode"], false);
    }

    #[test]
    fn test_error_context_creation() {
        let context = create_error_context(
            "TEST_ERROR",
            "Test error message",
            Some(serde_json::json!({"key": "value"})),
        );
        
        assert_eq!(context["error_code"], "TEST_ERROR");
        assert_eq!(context["error_message"], "Test error message");
        assert_eq!(context["additional_info"]["key"], "value");
    }
}