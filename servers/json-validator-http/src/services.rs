//! JSON验证服务

use crate::models::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::error;

/// JSON验证服务
#[derive(Clone)]
pub struct JsonValidatorService {
    /// 统计计数器
    stats: Arc<RwLock<ServiceStats>>,
    /// Schema缓存
    schema_cache: Arc<RwLock<HashMap<String, Arc<jsonschema::JSONSchema>>>>,
}

/// 服务统计信息
#[derive(Debug, Default, Clone)]
pub struct ServiceStats {
    /// 请求总数
    pub requests_total: u64,
    /// 成功请求数
    pub requests_success: u64,
    /// 失败请求数
    pub requests_failed: u64,
    /// 验证总数
    pub validations_total: u64,
    /// 成功验证数
    pub validations_success: u64,
    /// 失败验证数
    pub validations_failed: u64,
    /// 缓存命中数
    pub cache_hits: u64,
    /// 缓存未命中数
    pub cache_misses: u64,
    /// 总响应时间
    pub total_response_time: Duration,
}

impl JsonValidatorService {
    /// 创建新的验证服务
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(ServiceStats::default())),
            schema_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 验证JSON
    pub async fn validate_json(
        &self,
        json_data: &serde_json::Value,
        schema: Option<&serde_json::Value>,
        options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        let start_time = Instant::now();
        
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.requests_total += 1;
            stats.validations_total += 1;
        }
        
        let result = if let Some(schema) = schema {
            // 使用schema验证
            self.validate_with_schema(json_data, schema, options).await
        } else {
            // 基本JSON格式验证
            self.validate_basic(json_data, options).await
        };
        
        // 更新统计
        {
            let mut stats = self.stats.write().await;
            stats.total_response_time += start_time.elapsed();
            
            if let Ok(ref result) = result {
                if result.valid {
                    stats.requests_success += 1;
                    stats.validations_success += 1;
                } else {
                    stats.requests_failed += 1;
                    stats.validations_failed += 1;
                }
            } else {
                stats.requests_failed += 1;
                stats.validations_failed += 1;
            }
        }
        
        result
    }
    
    /// 使用schema验证JSON
    async fn validate_with_schema(
        &self,
        json_data: &serde_json::Value,
        schema: &serde_json::Value,
        options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        // 编译schema
        let compiled_schema = match self.get_or_compile_schema(schema).await {
            Ok(schema) => schema,
            Err(e) => {
                error!("Failed to compile schema: {}", e);
                return Err(format!("Invalid schema: {}", e));
            }
        };
        
        // 执行验证
        let start_time = Instant::now();
        let validation_result = compiled_schema.validate(json_data);
        let validation_time = start_time.elapsed();
        
        match validation_result {
            Ok(_) => Ok(ValidationResult {
                valid: true,
                errors: vec![],
                warnings: vec![],
                execution_time: validation_time.as_millis() as u64,
                cache_hit: false,
                cache_key: None,
            }),
            Err(errors) => {
                let error_messages: Vec<ValidationError> = errors
                    .into_iter()
                    .map(|e| ValidationError {
                        instance_path: e.instance_path.to_string(),
                        schema_path: e.schema_path.to_string(),
                        message: e.to_string(),
                        error_code: "SCHEMA_VALIDATION_ERROR".to_string(),
                        location: None,
                    })
                    .collect();
                
                Ok(ValidationResult {
                    valid: false,
                    errors: error_messages,
                    warnings: vec![],
                    execution_time: validation_time.as_millis() as u64,
                    cache_hit: false,
                    cache_key: None,
                })
            }
        }
    }
    
    /// 基本JSON格式验证
    async fn validate_basic(
        &self,
        json_data: &serde_json::Value,
        _options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        let start_time = Instant::now();
        
        // 基本验证 - 检查是否为有效的JSON
        let result = serde_json::to_string(json_data).is_ok();
        
        Ok(ValidationResult {
            valid: result,
            errors: if result { vec![] } else { 
                vec![ValidationError {
                    instance_path: "".to_string(),
                    schema_path: "".to_string(),
                    message: "Invalid JSON format".to_string(),
                    error_code: "INVALID_JSON_FORMAT".to_string(),
                    location: None,
                }]
            },
            warnings: vec![],
            execution_time: start_time.elapsed().as_millis() as u64,
            cache_hit: false,
            cache_key: None,
        })
    }
    
    /// 获取或编译schema
    async fn get_or_compile_schema(
        &self,
        schema: &serde_json::Value,
    ) -> Result<Arc<jsonschema::JSONSchema>, String> {
        let schema_key = schema.to_string();
        
        // 检查缓存
        {
            let cache = self.schema_cache.read().await;
            if let Some(compiled_schema) = cache.get(&schema_key) {
                let mut stats = self.stats.write().await;
                stats.cache_hits += 1;
                return Ok(compiled_schema.clone());
            }
        }
        
        // 编译schema
        let compiled_schema = jsonschema::JSONSchema::compile(schema)
            .map_err(|e| format!("Schema compilation failed: {}", e))?;
        
        // 缓存schema
        let arc_schema = Arc::new(compiled_schema);
        {
            let mut cache = self.schema_cache.write().await;
            cache.insert(schema_key, arc_schema.clone());
            
            let mut stats = self.stats.write().await;
            stats.cache_misses += 1;
        }
        
        Ok(arc_schema)
    }
    
    /// 获取统计信息
    pub async fn get_stats(&self) -> ServiceStats {
        self.stats.read().await.clone()
    }

    /// 验证JSON（简化版本）
    pub async fn validate_json_simple(
        &self,
        json_data: &serde_json::Value,
        options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        self.validate_json(json_data, None, options).await
    }

    /// 验证JSON与Schema（简化版本）
    pub async fn validate_json_with_schema_simple(
        &self,
        json_data: &serde_json::Value,
        schema: &serde_json::Value,
        options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        self.validate_json(json_data, Some(schema), options).await
    }

    /// 批量验证JSON
    pub async fn validate_json_batch(
        &self,
        items: &[crate::models::BatchValidationItem],
        options: &ValidationOptions,
    ) -> Result<Vec<crate::models::BatchValidationResult>, String> {
        let mut results = Vec::new();
        
        for item in items {
            let result = if let Some(ref schema) = item.schema {
                self.validate_json(&item.json_data, Some(schema), options).await
            } else {
                self.validate_json(&item.json_data, None, options).await
            };
            
            let validation_result = result.unwrap_or_else(|e| {
                ValidationResult {
                    valid: false,
                    errors: vec![ValidationError {
                        instance_path: "".to_string(),
                        schema_path: "".to_string(),
                        message: e,
                        error_code: "VALIDATION_ERROR".to_string(),
                        location: None,
                    }],
                    warnings: vec![],
                    execution_time: 0,
                    cache_hit: false,
                    cache_key: None,
                }
            });
            
            results.push(crate::models::BatchValidationResult {
                id: item.id.clone(),
                result: validation_result,
            });
        }
        
        Ok(results)
    }

    /// 获取请求总数
    pub async fn get_request_count(&self) -> u64 {
        self.stats.read().await.requests_total
    }

    /// 获取成功请求数
    pub async fn get_success_count(&self) -> u64 {
        self.stats.read().await.requests_success
    }

    /// 获取错误请求数
    pub async fn get_error_count(&self) -> u64 {
        self.stats.read().await.requests_failed
    }

    /// 获取平均响应时间
    pub async fn get_avg_response_time(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.requests_total == 0 {
            0.0
        } else {
            stats.total_response_time.as_millis() as f64 / stats.requests_total as f64
        }
    }

    /// 获取缓存命中率
    pub async fn get_cache_hit_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total = stats.cache_hits + stats.cache_misses;
        if total == 0 {
            0.0
        } else {
            stats.cache_hits as f64 / total as f64
        }
    }

    /// 获取验证总数
    pub async fn get_validation_count(&self) -> u64 {
        self.stats.read().await.validations_total
    }

    /// 获取验证成功率
    pub async fn get_validation_success_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.validations_total == 0 {
            0.0
        } else {
            stats.validations_success as f64 / stats.validations_total as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let service = JsonValidatorService::new();
        let stats = service.get_stats().await;
        
        assert_eq!(stats.requests_total, 0);
        assert_eq!(stats.validations_total, 0);
    }

    #[tokio::test]
    async fn test_basic_validation() {
        let service = JsonValidatorService::new();
        let json_data = serde_json::json!({"test": "value"});
        let options = ValidationOptions::default();
        
        let result = service.validate_json(&json_data, None, &options).await;
        
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_schema_validation() {
        let service = JsonValidatorService::new();
        let json_data = serde_json::json!({"name": "test", "age": 25});
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name", "age"]
        });
        let options = ValidationOptions::default();
        
        let result = service.validate_json(&json_data, Some(&schema), &options).await;
        
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }
}