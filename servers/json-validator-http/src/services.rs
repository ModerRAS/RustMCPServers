//! JSON验证服务

use crate::config::ServerConfig;
use crate::models::*;
use crate::utils::validation;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// JSON验证服务
pub struct JsonValidatorService {
    /// 服务器配置
    config: Arc<ServerConfig>,
    /// 缓存服务
    cache: Option<Arc<dyn CacheService>>,
    /// 统计计数器
    stats: Arc<RwLock<ServiceStats>>,
    /// Schema缓存
    schema_cache: Arc<RwLock<HashMap<String, Arc<jsonschema::JSONSchema>>>>,
}

/// 服务统计信息
#[derive(Debug, Default)]
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

/// 缓存服务trait
#[async_trait::async_trait]
pub trait CacheService: Send + Sync {
    /// 获取缓存值
    async fn get(&self, key: &str) -> Option<String>;
    
    /// 设置缓存值
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError>;
    
    /// 删除缓存值
    async fn delete(&self, key: &str) -> Result<(), CacheError>;
    
    /// 检查键是否存在
    async fn exists(&self, key: &str) -> bool;
    
    /// 清空缓存
    async fn clear(&self) -> Result<(), CacheError>;
}

/// 缓存错误
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Deserialization error: {0}")]
    Deserialization(String),
    
    #[error("Connection error: {0}")]
    Connection(String),
    
    #[error("Key not found")]
    NotFound,
}

impl JsonValidatorService {
    /// 创建新的JSON验证服务
    pub async fn new(config: ServerConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(config);
        let cache = if config.cache.enabled {
            Self::create_cache_service(&config).await?
        } else {
            None
        };
        
        let service = Self {
            config,
            cache,
            stats: Arc::new(RwLock::new(ServiceStats::default())),
            schema_cache: Arc::new(RwLock::new(HashMap::new())),
        };
        
        info!("JSON Validator service created successfully");
        Ok(service)
    }
    
    /// 创建缓存服务
    async fn create_cache_service(config: &ServerConfig) -> Result<Option<Arc<dyn CacheService>>, CacheError> {
        match config.cache.cache_type.as_str() {
            "redis" => {
                #[cfg(feature = "redis-cache")]
                {
                    if let Some(redis_url) = &config.cache.redis_url {
                        let redis_cache = RedisCache::new(redis_url).await?;
                        Ok(Some(Arc::new(redis_cache)))
                    } else {
                        warn!("Redis cache enabled but no URL provided");
                        Ok(None)
                    }
                }
                #[cfg(not(feature = "redis-cache"))]
                {
                    warn!("Redis cache feature not enabled");
                    Ok(None)
                }
            }
            "lru" => {
                #[cfg(feature = "lru-cache")]
                {
                    let lru_cache = LruCache::new(config.cache.max_size);
                    Ok(Some(Arc::new(lru_cache)))
                }
                #[cfg(not(feature = "lru-cache"))]
                {
                    warn!("LRU cache feature not enabled");
                    Ok(None)
                }
            }
            _ => {
                warn!("Unknown cache type: {}", config.cache.cache_type);
                Ok(None)
            }
        }
    }
    
    /// 验证JSON
    pub async fn validate_json(
        &self,
        json_data: &serde_json::Value,
        options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        let start_time = Instant::now();
        let mut stats = self.stats.write().await;
        stats.requests_total += 1;
        stats.validations_total += 1;
        drop(stats);
        
        debug!("Validating JSON with options: {:?}", options);
        
        // 检查缓存
        let cache_key = if let Some(key) = &options.cache_key {
            key.clone()
        } else {
            self.generate_cache_key("validate_json", json_data, &serde_json::Value::Null)
        };
        
        if let Some(cached_result) = self.get_cached_result(&cache_key).await {
            debug!("Cache hit for validation: {}", cache_key);
            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;
            drop(stats);
            
            let mut result = cached_result;
            result.cache_hit = true;
            result.cache_key = Some(cache_key);
            
            self.record_stats(&result, start_time.elapsed()).await;
            return Ok(result);
        }
        
        let mut stats = self.stats.write().await;
        stats.cache_misses += 1;
        drop(stats);
        
        // 执行验证
        let result = validation::validate_json(json_data, options)?;
        
        // 缓存结果
        if self.config.cache.enabled {
            if let Err(e) = self.cache_result(&cache_key, &result).await {
                warn!("Failed to cache validation result: {}", e);
            }
        }
        
        let mut final_result = result;
        final_result.cache_key = Some(cache_key);
        
        self.record_stats(&final_result, start_time.elapsed()).await;
        Ok(final_result)
    }
    
    /// 验证JSON与Schema
    pub async fn validate_json_with_schema(
        &self,
        json_data: &serde_json::Value,
        schema: &serde_json::Value,
        options: &ValidationOptions,
    ) -> Result<ValidationResult, String> {
        let start_time = Instant::now();
        let mut stats = self.stats.write().await;
        stats.requests_total += 1;
        stats.validations_total += 1;
        drop(stats);
        
        debug!("Validating JSON with schema");
        
        // 检查缓存
        let cache_key = if let Some(key) = &options.cache_key {
            key.clone()
        } else {
            self.generate_cache_key("validate_json_with_schema", json_data, schema)
        };
        
        if let Some(cached_result) = self.get_cached_result(&cache_key).await {
            debug!("Cache hit for schema validation: {}", cache_key);
            let mut stats = self.stats.write().await;
            stats.cache_hits += 1;
            drop(stats);
            
            let mut result = cached_result;
            result.cache_hit = true;
            result.cache_key = Some(cache_key);
            
            self.record_stats(&result, start_time.elapsed()).await;
            return Ok(result);
        }
        
        let mut stats = self.stats.write().await;
        stats.cache_misses += 1;
        drop(stats);
        
        // 编译Schema
        let compiled_schema = self.get_or_compile_schema(schema).await?;
        
        // 执行验证
        let result = validation::validate_json_with_schema(json_data, schema, options)?;
        
        // 缓存结果
        if self.config.cache.enabled {
            if let Err(e) = self.cache_result(&cache_key, &result).await {
                warn!("Failed to cache schema validation result: {}", e);
            }
        }
        
        let mut final_result = result;
        final_result.cache_key = Some(cache_key);
        
        self.record_stats(&final_result, start_time.elapsed()).await;
        Ok(final_result)
    }
    
    /// 批量验证JSON
    pub async fn validate_json_batch(
        &self,
        items: &[BatchValidationItem],
        options: &ValidationOptions,
    ) -> Result<Vec<BatchValidationResult>, String> {
        let start_time = Instant::now();
        let mut stats = self.stats.write().await;
        stats.requests_total += 1;
        stats.validations_total += items.len() as u64;
        drop(stats);
        
        debug!("Validating JSON batch with {} items", items.len());
        
        let mut results = Vec::with_capacity(items.len());
        
        for item in items {
            let item_start = Instant::now();
            
            let result = if let Some(schema) = &item.schema {
                self.validate_json_with_schema(&item.json_data, schema, options).await
            } else {
                self.validate_json(&item.json_data, options).await
            };
            
            match result {
                Ok(validation_result) => {
                    results.push(BatchValidationResult {
                        id: item.id.clone(),
                        result: validation_result,
                    });
                }
                Err(e) => {
                    error!("Failed to validate batch item {}: {}", item.id, e);
                    results.push(BatchValidationResult {
                        id: item.id.clone(),
                        result: ValidationResult::failure(
                            vec![ValidationError {
                                instance_path: "".to_string(),
                                schema_path: "".to_string(),
                                message: e.clone(),
                                error_code: "BATCH_VALIDATION_ERROR".to_string(),
                                location: None,
                            }],
                            item_start.elapsed().as_millis() as u64,
                            false,
                        ),
                    });
                }
            }
        }
        
        // 记录批量验证统计
        let duration = start_time.elapsed();
        let mut stats = self.stats.write().await;
        stats.total_response_time += duration;
        
        let success_count = results.iter().filter(|r| r.result.valid).count();
        if success_count == results.len() {
            stats.requests_success += 1;
            stats.validations_success += results.len() as u64;
        } else {
            stats.requests_failed += 1;
            stats.validations_failed += (results.len() - success_count) as u64;
        }
        
        drop(stats);
        
        info!("Batch validation completed: {}/{} successful", success_count, results.len());
        Ok(results)
    }
    
    /// 获取或编译Schema
    async fn get_or_compile_schema(
        &self,
        schema: &serde_json::Value,
    ) -> Result<Arc<jsonschema::JSONSchema>, String> {
        let schema_hash = self.hash_value(schema);
        let mut cache = self.schema_cache.write().await;
        
        if let Some(compiled_schema) = cache.get(&schema_hash) {
            debug!("Schema cache hit: {}", schema_hash);
            return Ok(compiled_schema.clone());
        }
        
        debug!("Compiling new schema: {}", schema_hash);
        
        let compiled_schema = jsonschema::JSONSchema::compile(schema)
            .map_err(|e| format!("Failed to compile schema: {}", e))?;
        
        let compiled_schema = Arc::new(compiled_schema);
        cache.insert(schema_hash, compiled_schema.clone());
        
        // 如果缓存太大，清理一些旧的条目
        if cache.len() > self.config.cache.max_size {
            self.cleanup_schema_cache(&mut cache).await;
        }
        
        Ok(compiled_schema)
    }
    
    /// 清理Schema缓存
    async fn cleanup_schema_cache(&self, cache: &mut HashMap<String, Arc<jsonschema::JSONSchema>>) {
        let target_size = self.config.cache.max_size / 2;
        let current_size = cache.len();
        
        if current_size > target_size {
            // 简单的LRU策略：删除前一半的条目
            let keys_to_remove: Vec<String> = cache.keys().take(current_size - target_size).cloned().collect();
            
            for key in keys_to_remove {
                cache.remove(&key);
            }
            
            debug!("Cleaned up schema cache: {} -> {}", current_size, cache.len());
        }
    }
    
    /// 生成缓存键
    fn generate_cache_key(
        &self,
        operation: &str,
        json_data: &serde_json::Value,
        schema: &serde_json::Value,
    ) -> String {
        let json_hash = self.hash_value(json_data);
        let schema_hash = self.hash_value(schema);
        
        format!("{}:{}:{}", operation, json_hash, schema_hash)
    }
    
    /// 计算值的哈希
    fn hash_value(&self, value: &serde_json::Value) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        value.to_string().hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
    
    /// 获取缓存结果
    async fn get_cached_result(&self, key: &str) -> Option<ValidationResult> {
        if let Some(cache) = &self.cache {
            match cache.get(key).await {
                Ok(Some(value)) => {
                    match serde_json::from_str(&value) {
                        Ok(result) => Some(result),
                        Err(e) => {
                            warn!("Failed to deserialize cached result: {}", e);
                            None
                        }
                    }
                }
                Ok(None) => None,
                Err(e) => {
                    warn!("Failed to get cached result: {}", e);
                    None
                }
            }
        } else {
            None
        }
    }
    
    /// 缓存结果
    async fn cache_result(&self, key: &str, result: &ValidationResult) -> Result<(), CacheError> {
        if let Some(cache) = &self.cache {
            let value = serde_json::to_string(result)
                .map_err(|e| CacheError::Serialization(e.to_string()))?;
            
            cache.set(key, &value, Duration::from_secs(self.config.cache.ttl)).await?;
        }
        Ok(())
    }
    
    /// 记录统计信息
    async fn record_stats(&self, result: &ValidationResult, duration: Duration) {
        let mut stats = self.stats.write().await;
        stats.total_response_time += duration;
        
        if result.valid {
            stats.requests_success += 1;
            stats.validations_success += 1;
        } else {
            stats.requests_failed += 1;
            stats.validations_failed += 1;
        }
        
        drop(stats);
        
        // 记录指标
        if cfg!(feature = "metrics") {
            crate::middleware::metrics::record_validation_metrics(
                result.valid,
                duration,
                result.cache_hit,
            ).await;
        }
    }
    
    /// 获取服务统计信息
    pub async fn get_stats(&self) -> ServiceStats {
        self.stats.read().await.clone()
    }
    
    /// 获取请求计数
    pub async fn get_request_count(&self) -> u64 {
        self.stats.read().await.requests_total
    }
    
    /// 获取成功计数
    pub async fn get_success_count(&self) -> u64 {
        self.stats.read().await.requests_success
    }
    
    /// 获取错误计数
    pub async fn get_error_count(&self) -> u64 {
        self.stats.read().await.requests_failed
    }
    
    /// 获取验证计数
    pub async fn get_validation_count(&self) -> u64 {
        self.stats.read().await.validations_total
    }
    
    /// 获取平均响应时间
    pub async fn get_avg_response_time(&self) -> f64 {
        let stats = self.stats.read().await;
        if stats.requests_total == 0 {
            0.0
        } else {
            stats.total_response_time.as_secs_f64() / stats.requests_total as f64
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
    
    /// 获取验证成功率
    pub async fn get_validation_success_rate(&self) -> f64 {
        let stats = self.stats.read().await;
        let total = stats.validations_success + stats.validations_failed;
        if total == 0 {
            0.0
        } else {
            stats.validations_success as f64 / total as f64
        }
    }
    
    /// 清理缓存
    pub async fn clear_cache(&self) -> Result<(), CacheError> {
        if let Some(cache) = &self.cache {
            cache.clear().await?;
        }
        Ok(())
    }
}

/// Redis缓存实现
#[cfg(feature = "redis-cache")]
pub struct RedisCache {
    client: redis::Client,
}

#[cfg(feature = "redis-cache")]
impl RedisCache {
    /// 创建新的Redis缓存
    pub async fn new(redis_url: &str) -> Result<Self, CacheError> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        // 测试连接
        let mut conn = client.get_async_connection().await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        redis::cmd("PING").query_async::<_, String>(&mut conn).await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        info!("Redis cache connected successfully");
        Ok(Self { client })
    }
}

#[cfg(feature = "redis-cache")]
#[async_trait::async_trait]
impl CacheService for RedisCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut conn = self.client.get_async_connection().await.ok()?;
        redis::cmd("GET").arg(key).query_async(&mut conn).await.ok()
    }
    
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        redis::cmd("SETEX")
            .arg(key)
            .arg(ttl.as_secs())
            .arg(value)
            .query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;
        
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        redis::cmd("DEL").arg(key).query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;
        
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> bool {
        let mut conn = self.client.get_async_connection().await.ok()?;
        redis::cmd("EXISTS").arg(key).query_async(&mut conn).await.unwrap_or(0) > 0
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| CacheError::Connection(e.to_string()))?;
        
        redis::cmd("FLUSHDB").query_async(&mut conn)
            .await
            .map_err(|e| CacheError::Redis(e.to_string()))?;
        
        Ok(())
    }
}

/// LRU缓存实现
#[cfg(feature = "lru-cache")]
pub struct LruCache {
    cache: Arc<tokio::sync::RwLock<lru::LruCache<String, CacheEntry>>>,
}

#[cfg(feature = "lru-cache")]
struct CacheEntry {
    value: String,
    expires_at: Instant,
}

#[cfg(feature = "lru-cache")]
impl LruCache {
    /// 创建新的LRU缓存
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(tokio::sync::RwLock::new(lru::LruCache::new(capacity))),
        }
    }
    
    /// 清理过期条目
    async fn cleanup_expired(&self) {
        let mut cache = self.cache.write().await;
        let now = Instant::now();
        
        cache.retain(|_, entry| entry.expires_at > now);
    }
}

#[cfg(feature = "lru-cache")]
#[async_trait::async_trait]
impl CacheService for LruCache {
    async fn get(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write().await;
        
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            } else {
                cache.pop(key);
            }
        }
        
        None
    }
    
    async fn set(&self, key: &str, value: &str, ttl: Duration) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        
        let entry = CacheEntry {
            value: value.to_string(),
            expires_at: Instant::now() + ttl,
        };
        
        cache.put(key.to_string(), entry);
        Ok(())
    }
    
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        cache.pop(key);
        Ok(())
    }
    
    async fn exists(&self, key: &str) -> bool {
        let cache = self.cache.read().await;
        cache.contains(key)
    }
    
    async fn clear(&self) -> Result<(), CacheError> {
        let mut cache = self.cache.write().await;
        cache.clear();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_creation() {
        let config = ServerConfig::default();
        let service = JsonValidatorService::new(config).await;
        
        assert!(service.is_ok());
    }

    #[tokio::test]
    async fn test_basic_json_validation() {
        let config = ServerConfig::default();
        let service = JsonValidatorService::new(config).await.unwrap();
        
        let json_data = serde_json::json!({"name": "test", "age": 25});
        let options = ValidationOptions::default();
        
        let result = service.validate_json(&json_data, &options).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().valid);
    }

    #[tokio::test]
    async fn test_schema_validation() {
        let config = ServerConfig::default();
        let service = JsonValidatorService::new(config).await.unwrap();
        
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
        
        let result = service.validate_json_with_schema(&json_data, &schema, &options).await;
        
        assert!(result.is_ok());
        assert!(result.unwrap().valid);
    }

    #[tokio::test]
    async fn test_batch_validation() {
        let config = ServerConfig::default();
        let service = JsonValidatorService::new(config).await.unwrap();
        
        let items = vec![
            BatchValidationItem {
                id: "1".to_string(),
                json_data: serde_json::json!({"name": "test1"}),
                schema: None,
            },
            BatchValidationItem {
                id: "2".to_string(),
                json_data: serde_json::json!({"name": "test2"}),
                schema: None,
            },
        ];
        let options = ValidationOptions::default();
        
        let result = service.validate_json_batch(&items, &options).await;
        
        assert!(result.is_ok());
        let results = result.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].result.valid);
        assert!(results[1].result.valid);
    }
}