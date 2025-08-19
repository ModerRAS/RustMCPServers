# JSON验证MCP服务器性能优化和安全策略

## 概述

本文档详细描述了JSON验证MCP服务器的性能优化策略和安全措施，确保系统在高负载下的稳定性和安全性。

## 性能优化策略

### 1. 架构级优化

#### 1.1 无状态设计
**目标**: 消除服务器状态，实现水平扩展

**实现策略**:
```rust
// 无状态服务设计
pub struct JsonValidatorService {
    validator: Arc<JsonValidator>,
    cache: Arc<CacheManager>,
    metrics: Arc<MetricsCollector>,
}

impl Clone for JsonValidatorService {
    fn clone(&self) -> Self {
        Self {
            validator: Arc::clone(&self.validator),
            cache: Arc::clone(&self.cache),
            metrics: Arc::clone(&self.metrics),
        }
    }
}
```

**优化效果**:
- 支持任意水平扩展
- 无会话亲和性要求
- 简化负载均衡策略

#### 1.2 连接池管理
**目标**: 减少连接建立开销，提高资源利用率

**实现策略**:
```rust
use sqlx::postgres::PgPoolOptions;
use redis::aio::ConnectionManager;

pub struct ConnectionPools {
    db_pool: sqlx::PgPool,
    redis_pool: bb8::Pool<ConnectionManager<redis::aio::Connection>>,
}

impl ConnectionPools {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // 数据库连接池
        let db_pool = PgPoolOptions::new()
            .max_connections(config.database.max_connections)
            .min_connections(config.database.min_connections)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(3600))
            .connect(&config.database.url)
            .await?;
        
        // Redis连接池
        let redis_manager = ConnectionManager::new(config.redis.url.clone());
        let redis_pool = bb8::Pool::builder()
            .max_size(config.redis.pool_size)
            .build(redis_manager)
            .await?;
        
        Ok(Self { db_pool, redis_pool })
    }
}
```

**优化效果**:
- 减少连接建立时间90%
- 提高并发处理能力
- 避免连接泄漏

#### 1.3 异步I/O优化
**目标**: 最大化I/O并发，减少阻塞

**实现策略**:
```rust
// 异步处理器
pub struct AsyncValidator {
    runtime: tokio::runtime::Runtime,
    task_queue: mpsc::UnboundedSender<ValidationTask>,
}

impl AsyncValidator {
    pub fn new(worker_count: usize) -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(worker_count)
            .thread_name("validation-worker")
            .thread_stack_size(2 * 1024 * 1024) // 2MB stack
            .max_blocking_threads(512)
            .enable_all()
            .build()
            .unwrap();
        
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        // 启动工作线程
        for i in 0..worker_count {
            let receiver = task_receiver.clone();
            runtime.spawn(async move {
                Self::worker_loop(i, receiver).await;
            });
        }
        
        Self { runtime, task_queue: task_sender }
    }
    
    async fn worker_loop(worker_id: usize, mut receiver: mpsc::UnboundedReceiver<ValidationTask>) {
        while let Some(task) = receiver.recv().await {
            let result = Self::process_task(task).await;
            // 发送结果
        }
    }
}
```

**优化效果**:
- 支持高并发I/O操作
- 减少线程上下文切换
- 提高CPU利用率

### 2. 缓存优化策略

#### 2.1 多级缓存架构
**目标**: 减少重复计算，提高响应速度

**架构设计**:
```rust
pub struct MultiLevelCache {
    l1_cache: Arc<RwLock<LruCache<String, ValidationResult>>>,
    l2_cache: Arc<RedisCache>,
    l3_cache: Arc<DistributedCache>,
}

impl MultiLevelCache {
    pub async fn get(&self, key: &str) -> Option<ValidationResult> {
        // L1: 内存缓存
        if let Some(result) = self.l1_cache.write().await.get(key) {
            return Some(result.clone());
        }
        
        // L2: Redis缓存
        if let Some(result) = self.l2_cache.get(key).await {
            // 回填L1缓存
            self.l1_cache.write().await.put(key.to_string(), result.clone());
            return Some(result);
        }
        
        // L3: 分布式缓存
        if let Some(result) = self.l3_cache.get(key).await {
            // 回填L1和L2缓存
            self.l1_cache.write().await.put(key.to_string(), result.clone());
            self.l2_cache.set(key, &result).await;
            return Some(result);
        }
        
        None
    }
    
    pub async fn set(&self, key: &str, value: ValidationResult) {
        // 设置所有级别的缓存
        self.l1_cache.write().await.put(key.to_string(), value.clone());
        self.l2_cache.set(key, &value).await;
        self.l3_cache.set(key, &value).await;
    }
}
```

**缓存策略**:
- **L1缓存**: 1000个条目，TTL 1小时
- **L2缓存**: Redis集群，TTL 30分钟
- **L3缓存**: 分布式缓存，TTL 15分钟

#### 2.2 缓存预热策略
**目标**: 启动时预加载热点数据

**实现策略**:
```rust
pub struct CacheWarmer {
    popular_schemas: Vec<Schema>,
    cache: Arc<MultiLevelCache>,
}

impl CacheWarmer {
    pub async fn warm_cache(&self) {
        for schema in &self.popular_schemas {
            // 预编译Schema
            let compiled_schema = JsonValidator::compile_schema(&schema.definition).await;
            if let Ok(schema) = compiled_schema {
                let key = self.generate_schema_key(&schema);
                self.cache.set(&key, CachedSchema::Compiled(schema)).await;
            }
        }
        
        // 预加载常见验证结果
        for validation in &self.common_validations {
            let result = self.validate_cached(validation).await;
            if let Ok(result) = result {
                let key = self.generate_validation_key(validation);
                self.cache.set(&key, result).await;
            }
        }
    }
}
```

#### 2.3 缓存失效策略
**目标**: 避免缓存雪崩，保证数据一致性

**实现策略**:
```rust
pub struct CacheInvalidator {
    cache: Arc<MultiLevelCache>,
    invalidation_queue: mpsc::UnboundedSender<InvalidationRequest>,
}

impl CacheInvalidator {
    pub async fn invalidate_schema(&self, schema_id: &str) {
        // 生成所有相关缓存键
        let pattern = format!("schema:{}:*", schema_id);
        let keys = self.cache.get_keys_by_pattern(&pattern).await;
        
        // 分批失效，避免雪崩
        for chunk in keys.chunks(100) {
            for key in chunk {
                self.cache.delete(key).await;
            }
            // 添加延迟，避免压力过大
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    
    pub async fn gradual_invalidation(&self, keys: Vec<String>) {
        // 渐进式失效
        for (i, key) in keys.iter().enumerate() {
            self.cache.delete(key).await;
            
            // 指数退避
            let delay = std::time::Duration::from_millis(50 * 2u64.pow((i / 10) as u32));
            tokio::time::sleep(delay).await;
        }
    }
}
```

### 3. 内存优化策略

#### 3.1 内存池管理
**目标**: 减少内存分配开销，提高内存利用率

**实现策略**:
```rust
use std::alloc::{GlobalAlloc, System, Layout};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MemoryPool {
    small_objects: Vec<Vec<u8>>,
    medium_objects: Vec<Vec<u8>>,
    small_objects_free: Vec<*mut u8>,
    medium_objects_free: Vec<*mut u8>,
    allocated: AtomicUsize,
}

impl MemoryPool {
    pub fn new() -> Self {
        Self {
            small_objects: Vec::new(),
            medium_objects: Vec::new(),
            small_objects_free: Vec::new(),
            medium_objects_free: Vec::new(),
            allocated: AtomicUsize::new(0),
        }
    }
    
    pub fn allocate_small(&mut self) -> *mut u8 {
        if let Some(ptr) = self.small_objects_free.pop() {
            return ptr;
        }
        
        let mut buffer = vec![0u8; 256]; // 256字节小对象池
        let ptr = buffer.as_mut_ptr();
        self.small_objects.push(buffer);
        ptr
    }
    
    pub fn deallocate_small(&mut self, ptr: *mut u8) {
        self.small_objects_free.push(ptr);
    }
}
```

#### 3.2 字符串去重
**目标**: 减少重复字符串的内存占用

**实现策略**:
```rust
use std::collections::HashSet;
use std::sync::Arc;

pub struct StringInterner {
    strings: HashSet<Arc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: HashSet::new(),
        }
    }
    
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(existing) = self.strings.get(s) {
            existing.clone()
        } else {
            let arc_str: Arc<str> = Arc::from(s);
            self.strings.insert(arc_str.clone());
            arc_str
        }
    }
    
    pub fn get_memory_usage(&self) -> usize {
        self.strings.iter().map(|s| s.len()).sum()
    }
}
```

#### 3.3 零拷贝优化
**目标**: 减少数据拷贝，提高处理效率

**实现策略**:
```rust
use serde_json::Value;
use bytes::{Bytes, BytesMut, BufMut};

pub struct ZeroCopyValidator {
    buffer_pool: Vec<BytesMut>,
}

impl ZeroCopyValidator {
    pub fn validate_json_slice(&self, data: &[u8]) -> Result<ValidationResult, ValidationError> {
        // 使用零拷贝解析
        let value: Value = serde_json::from_slice(data)?;
        
        // 避免拷贝，直接引用
        self.validate_value(&value)
    }
    
    pub fn validate_bytes(&self, bytes: &Bytes) -> Result<ValidationResult, ValidationError> {
        // 使用Bytes的零拷贝特性
        let value: Value = serde_json::from_slice(bytes.as_ref())?;
        self.validate_value(&value)
    }
}
```

### 4. 算法优化策略

#### 4.1 Schema编译优化
**目标**: 优化Schema编译性能

**实现策略**:
```rust
pub struct OptimizedSchemaCompiler {
    compilation_cache: Arc<RwLock<HashMap<String, Arc<CompiledSchema>>>>,
    compilation_stats: CompilationStats,
}

impl OptimizedSchemaCompiler {
    pub async fn compile_schema(&self, schema: &Value) -> Result<Arc<CompiledSchema>, CompilationError> {
        let schema_key = self.generate_schema_key(schema);
        
        // 检查编译缓存
        if let Some(cached) = self.compilation_cache.read().await.get(&schema_key) {
            return Ok(cached.clone());
        }
        
        // 优化编译过程
        let optimized_schema = self.optimize_schema(schema)?;
        let compiled = self.compile_optimized_schema(&optimized_schema).await?;
        
        // 缓存编译结果
        let arc_compiled = Arc::new(compiled);
        self.compilation_cache.write().await.insert(schema_key, arc_compiled.clone());
        
        Ok(arc_compiled)
    }
    
    fn optimize_schema(&self, schema: &Value) -> Result<Value, OptimizationError> {
        // 移除冗余的Schema定义
        // 预计算常量表达式
        // 优化正则表达式
        // 简化嵌套结构
        Ok(schema.clone())
    }
}
```

#### 4.2 验证算法优化
**目标**: 优化验证算法性能

**实现策略**:
```rust
pub struct OptimizedValidator {
    compiled_schemas: Arc<RwLock<HashMap<String, Arc<CompiledSchema>>>>,
    validation_stats: ValidationStats,
}

impl OptimizedValidator {
    pub async fn validate_optimized(&self, data: &Value, schema: &CompiledSchema) -> ValidationResult {
        let start_time = std::time::Instant::now();
        
        // 快速路径检查
        if let Some(fast_result) = self.fast_path_validation(data, schema) {
            return fast_result;
        }
        
        // 深度验证
        let result = self.deep_validation(data, schema).await;
        
        let duration = start_time.elapsed();
        self.validation_stats.record_validation(duration, result.is_valid());
        
        result
    }
    
    fn fast_path_validation(&self, data: &Value, schema: &CompiledSchema) -> Option<ValidationResult> {
        // 类型快速检查
        if !self.check_type_compatibility(data, schema) {
            return Some(ValidationResult::invalid("Type mismatch"));
        }
        
        // 必需字段快速检查
        if !self.check_required_fields(data, schema) {
            return Some(ValidationResult::invalid("Missing required fields"));
        }
        
        None // 需要深度验证
    }
}
```

### 5. 并发优化策略

#### 5.1 工作窃取调度
**目标**: 优化线程间负载均衡

**实现策略**:
```rust
use std::sync::mpsc;
use std::thread;
use std::collections::VecDeque;

pub struct WorkStealingScheduler {
    workers: Vec<Worker>,
    work_queue: mpsc::Sender<WorkItem>,
}

struct Worker {
    id: usize,
    local_queue: VecDeque<WorkItem>,
    steal_target: Option<usize>,
}

impl WorkStealingScheduler {
    pub fn new(num_workers: usize) -> Self {
        let (work_sender, work_receiver) = mpsc::channel();
        let mut workers = Vec::new();
        
        for i in 0..num_workers {
            let worker = Worker {
                id: i,
                local_queue: VecDeque::new(),
                steal_target: if i < num_workers - 1 { Some(i + 1) } else { None },
            };
            workers.push(worker);
        }
        
        Self { workers, work_queue: work_sender }
    }
    
    pub fn schedule_work(&self, work: WorkItem) {
        // 选择负载最低的工作线程
        let target_worker = self.select_least_loaded_worker();
        // 发送工作项
    }
    
    fn select_least_loaded_worker(&self) -> usize {
        // 基于负载选择工作线程
        0 // 简化实现
    }
}
```

#### 5.2 读写锁优化
**目标**: 减少锁竞争，提高并发性能

**实现策略**:
```rust
use std::sync::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use parking_lot::RwLock as ParkingRwLock;

pub struct OptimizedCache {
    data: ParkingRwLock<HashMap<String, CacheEntry>>,
    read_count: AtomicUsize,
    write_count: AtomicUsize,
}

impl OptimizedCache {
    pub fn get(&self, key: &str) -> Option<CacheEntry> {
        self.read_count.fetch_add(1, Ordering::Relaxed);
        
        // 使用parking_lot的RwLock，减少锁竞争
        let guard = self.data.read();
        guard.get(key).cloned()
    }
    
    pub fn set(&self, key: String, value: CacheEntry) {
        self.write_count.fetch_add(1, Ordering::Relaxed);
        
        // 批量写入优化
        let mut guard = self.data.write();
        guard.insert(key, value);
    }
    
    pub fn get_stats(&self) -> (usize, usize) {
        (
            self.read_count.load(Ordering::Relaxed),
            self.write_count.load(Ordering::Relaxed)
        )
    }
}
```

## 安全策略

### 1. 输入验证和清理

#### 1.1 严格的输入验证
**目标**: 防止恶意输入导致的安全问题

**实现策略**:
```rust
use validator::Validate;
use serde::Deserialize;

#[derive(Debug, Deserialize, Validate)]
pub struct ValidationRequest {
    #[validate(length(max = 1024 * 1024))] // 1MB limit
    pub json_data: serde_json::Value,
    
    #[validate(length(max = 100 * 1024))] // 100KB limit
    pub schema: serde_json::Value,
    
    #[validate(custom = "validate_options")]
    pub options: ValidationOptions,
}

fn validate_options(options: &ValidationOptions) -> Result<(), validator::ValidationError> {
    // 验证自定义格式
    if !options.custom_formats.is_empty() {
        for (format_name, format_def) in &options.custom_formats {
            if format_name.len() > 50 {
                return Err(validator::ValidationError::new("format_name_too_long"));
            }
            if format_def.len() > 200 {
                return Err(validator::ValidationError::new("format_def_too_long"));
            }
        }
    }
    
    Ok(())
}

pub struct InputValidator {
    max_depth: usize,
    max_keys: usize,
    max_string_length: usize,
}

impl InputValidator {
    pub fn validate_json_structure(&self, value: &Value) -> Result<(), ValidationError> {
        self.validate_value_depth(value, 0)?;
        self.validate_value_size(value)?;
        self.validate_string_content(value)?;
        Ok(())
    }
    
    fn validate_value_depth(&self, value: &Value, current_depth: usize) -> Result<(), ValidationError> {
        if current_depth > self.max_depth {
            return Err(ValidationError::DepthExceeded(self.max_depth));
        }
        
        match value {
            Value::Object(obj) => {
                for (_, v) in obj {
                    self.validate_value_depth(v, current_depth + 1)?;
                }
            }
            Value::Array(arr) => {
                for v in arr {
                    self.validate_value_depth(v, current_depth + 1)?;
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn validate_value_size(&self, value: &Value) -> Result<(), ValidationError> {
        let key_count = match value {
            Value::Object(obj) => obj.len(),
            Value::Array(arr) => arr.len(),
            _ => 0,
        };
        
        if key_count > self.max_keys {
            return Err(ValidationError::SizeExceeded(self.max_keys));
        }
        
        Ok(())
    }
    
    fn validate_string_content(&self, value: &Value) -> Result<(), ValidationError> {
        if let Value::String(s) = value {
            if s.len() > self.max_string_length {
                return Err(ValidationError::StringTooLong(self.max_string_length));
            }
            
            // 检查潜在的恶意内容
            if s.contains("<script") || s.contains("javascript:") {
                return Err(ValidationError::MaliciousContent);
            }
        }
        
        Ok(())
    }
}
```

#### 1.2 Schema安全验证
**目标**: 防止恶意Schema导致的安全问题

**实现策略**:
```rust
pub struct SchemaSecurityValidator {
    max_schema_size: usize,
    allowed_keywords: HashSet<String>,
    forbidden_patterns: Vec<regex::Regex>,
}

impl SchemaSecurityValidator {
    pub fn validate_schema_security(&self, schema: &Value) -> Result<(), SecurityError> {
        // 验证Schema大小
        self.validate_schema_size(schema)?;
        
        // 验证Schema关键字
        self.validate_schema_keywords(schema)?;
        
        // 验证模式安全性
        self.validate_schema_patterns(schema)?;
        
        // 验证递归深度
        self.validate_schema_recursion(schema)?;
        
        Ok(())
    }
    
    fn validate_schema_size(&self, schema: &Value) -> Result<(), SecurityError> {
        let schema_str = serde_json::to_string(schema)?;
        if schema_str.len() > self.max_schema_size {
            return Err(SecurityError::SchemaTooLarge(self.max_schema_size));
        }
        Ok(())
    }
    
    fn validate_schema_keywords(&self, schema: &Value) -> Result<(), SecurityError> {
        if let Value::Object(obj) = schema {
            for keyword in obj.keys() {
                if !self.allowed_keywords.contains(keyword) {
                    return Err(SecurityError::ForbiddenKeyword(keyword.clone()));
                }
            }
        }
        Ok(())
    }
    
    fn validate_schema_patterns(&self, schema: &Value) -> Result<(), SecurityError> {
        let schema_str = serde_json::to_string(schema)?;
        
        for pattern in &self.forbidden_patterns {
            if pattern.is_match(&schema_str) {
                return Err(SecurityError::MaliciousPattern);
            }
        }
        
        Ok(())
    }
    
    fn validate_schema_recursion(&self, schema: &Value) -> Result<(), SecurityError> {
        // 检查Schema中的递归引用
        self.check_recursive_references(schema, &mut HashSet::new())?;
        Ok(())
    }
    
    fn check_recursive_references(
        &self,
        schema: &Value,
        visited: &mut HashSet<String>,
    ) -> Result<(), SecurityError> {
        if let Value::Object(obj) = schema {
            if let Some(Value::String(ref_value)) = obj.get("$ref") {
                if visited.contains(ref_value) {
                    return Err(SecurityError::RecursiveReference);
                }
                visited.insert(ref_value.clone());
            }
            
            for (_, value) in obj {
                self.check_recursive_references(value, visited)?;
            }
        }
        
        Ok(())
    }
}
```

### 2. 认证和授权

#### 2.1 JWT认证
**目标**: 安全的身份验证机制

**实现策略**:
```rust
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

pub struct AuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl AuthManager {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation: Validation::new(Algorithm::HS256),
        }
    }
    
    pub fn generate_token(&self, user_id: &str, roles: Vec<String>, permissions: Vec<String>) -> Result<String, AuthError> {
        let now = chrono::Utc::now();
        let exp = now + chrono::Duration::hours(24);
        
        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
            roles,
            permissions,
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AuthError::TokenGeneration(e.to_string()))
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| AuthError::InvalidToken(e.to_string()))?;
        
        Ok(token_data.claims)
    }
    
    pub fn check_permission(&self, claims: &Claims, required_permission: &str) -> bool {
        claims.permissions.contains(&required_permission.to_string())
    }
    
    pub fn check_role(&self, claims: &Claims, required_role: &str) -> bool {
        claims.roles.contains(&required_role.to_string())
    }
}
```

#### 2.2 基于角色的访问控制
**目标**: 细粒度的权限控制

**实现策略**:
```rust
pub enum Permission {
    ValidateJson,
    ManageSchemas,
    ViewMetrics,
    SystemAdmin,
}

pub enum Role {
    User,
    SchemaManager,
    Admin,
}

pub struct AccessControl {
    role_permissions: HashMap<Role, HashSet<Permission>>,
}

impl AccessControl {
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();
        
        // 用户角色
        let mut user_permissions = HashSet::new();
        user_permissions.insert(Permission::ValidateJson);
        role_permissions.insert(Role::User, user_permissions);
        
        // Schema管理角色
        let mut schema_permissions = HashSet::new();
        schema_permissions.insert(Permission::ValidateJson);
        schema_permissions.insert(Permission::ManageSchemas);
        schema_permissions.insert(Permission::ViewMetrics);
        role_permissions.insert(Role::SchemaManager, schema_permissions);
        
        // 管理员角色
        let mut admin_permissions = HashSet::new();
        admin_permissions.insert(Permission::ValidateJson);
        admin_permissions.insert(Permission::ManageSchemas);
        admin_permissions.insert(Permission::ViewMetrics);
        admin_permissions.insert(Permission::SystemAdmin);
        role_permissions.insert(Role::Admin, admin_permissions);
        
        Self { role_permissions }
    }
    
    pub fn check_permission(&self, roles: &[Role], permission: Permission) -> bool {
        roles.iter().any(|role| {
            self.role_permissions
                .get(role)
                .map(|permissions| permissions.contains(&permission))
                .unwrap_or(false)
        })
    }
    
    pub fn check_admin_access(&self, roles: &[Role]) -> bool {
        self.check_permission(roles, Permission::SystemAdmin)
    }
}
```

### 3. 网络安全

#### 3.1 TLS配置
**目标**: 安全的传输层加密

**实现策略**:
```rust
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile;
use std::fs::File;
use std::io::BufReader;

pub struct TlsConfig {
    server_config: ServerConfig,
}

impl TlsConfig {
    pub fn new(cert_path: &str, key_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // 加载证书
        let cert_file = File::open(cert_path)?;
        let mut cert_reader = BufReader::new(cert_file);
        let certs = rustls_pemfile::certs(&mut cert_reader)?
            .into_iter()
            .map(Certificate)
            .collect::<Vec<_>>();
        
        // 加载私钥
        let key_file = File::open(key_path)?;
        let mut key_reader = BufReader::new(key_file);
        let keys = rustls_pemfile::pkcs8_private_keys(&mut key_reader)?
            .into_iter()
            .map(PrivateKey)
            .collect::<Vec<_>>();
        
        // 创建服务器配置
        let config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, keys[0].clone())?;
        
        Ok(Self { server_config: config })
    }
    
    pub fn get_server_config(&self) -> &ServerConfig {
        &self.server_config
    }
}
```

#### 3.2 安全头设置
**目标**: 防止常见的Web攻击

**实现策略**:
```rust
use axum::http::HeaderValue;
use tower_http::set_header::SetResponseHeaderLayer;

pub fn security_headers() -> Vec<SetResponseHeaderLayer> {
    vec![
        // 严格传输安全
        SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ),
        // 内容类型选项
        SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ),
        // X-Frame-Options
        SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ),
        // XSS保护
        SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("x-xss-protection"),
            HeaderValue::from_static("1; mode=block"),
        ),
        // 内容安全策略
        SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'"),
        ),
        // 引用策略
        SetResponseHeaderLayer::overriding(
            axum::http::HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ),
    ]
}
```

### 4. 限流和防护

#### 4.1 请求限流
**目标**: 防止滥用和DoS攻击

**实现策略**:
```rust
use governor::{Quota, RateLimiter};
use nonzero_ext::*;
use std::sync::Arc;
use std::time::Duration;

pub struct RateLimiterManager {
    user_limiter: Arc<RateLimiter<governor::clock::QuantaClock>>,
    ip_limiter: Arc<RateLimiter<governor::clock::QuantaClock>>,
    global_limiter: Arc<RateLimiter<governor::clock::QuantaClock>>,
}

impl RateLimiterManager {
    pub fn new() -> Self {
        // 用户级别限流：1000请求/分钟
        let user_limiter = Arc::new(RateLimiter::direct(
            Quota::per_minute(nonzero!(1000u32))
        ));
        
        // IP级别限流：100请求/分钟
        let ip_limiter = Arc::new(RateLimiter::direct(
            Quota::per_minute(nonzero!(100u32))
        ));
        
        // 全局限流：10000请求/分钟
        let global_limiter = Arc::new(RateLimiter::direct(
            Quota::per_minute(nonzero!(10000u32))
        ));
        
        Self {
            user_limiter,
            ip_limiter,
            global_limiter,
        }
    }
    
    pub async fn check_user_rate_limit(&self, user_id: &str) -> Result<(), RateLimitError> {
        match self.user_limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::UserRateLimitExceeded),
        }
    }
    
    pub async fn check_ip_rate_limit(&self, ip: &str) -> Result<(), RateLimitError> {
        match self.ip_limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::IpRateLimitExceeded),
        }
    }
    
    pub async fn check_global_rate_limit(&self) -> Result<(), RateLimitError> {
        match self.global_limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError::GlobalRateLimitExceeded),
        }
    }
}
```

#### 4.2 请求大小限制
**目标**: 防止大请求导致的资源耗尽

**实现策略**:
```rust
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

pub async fn request_size_limit(
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    const MAX_REQUEST_SIZE: usize = 1024 * 1024; // 1MB
    
    // 检查Content-Length
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length) = content_length.to_str() {
            if let Ok(size) = length.parse::<usize>() {
                if size > MAX_REQUEST_SIZE {
                    return Ok(Response::builder()
                        .status(axum::http::StatusCode::PAYLOAD_TOO_LARGE)
                        .body(Body::from("Request too large"))
                        .unwrap());
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}
```

### 5. 安全监控和审计

#### 5.1 安全事件日志
**目标**: 记录和监控安全相关事件

**实现策略**:
```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub event_type: SecurityEventType,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: String,
    pub details: serde_json::Value,
    pub severity: SecuritySeverity,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SecurityEventType {
    AuthenticationSuccess,
    AuthenticationFailure,
    AuthorizationFailure,
    RateLimitExceeded,
    MaliciousInputDetected,
    SchemaValidationFailed,
    SuspiciousActivity,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct SecurityLogger {
    events: Vec<SecurityEvent>,
    max_events: usize,
}

impl SecurityLogger {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: Vec::new(),
            max_events,
        }
    }
    
    pub fn log_event(&mut self, event: SecurityEvent) {
        self.events.push(event);
        
        // 保持事件数量限制
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
        
        // 发送到监控系统
        self.send_to_monitoring(&event);
    }
    
    pub fn get_events_by_type(&self, event_type: SecurityEventType) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.event_type == event_type)
            .collect()
    }
    
    pub fn get_events_by_severity(&self, severity: SecuritySeverity) -> Vec<&SecurityEvent> {
        self.events
            .iter()
            .filter(|event| event.severity == severity)
            .collect()
    }
    
    fn send_to_monitoring(&self, event: &SecurityEvent) {
        // 发送到监控系统
        tracing::warn!("Security event: {:?}", event);
    }
}
```

#### 5.2 异常检测
**目标**: 检测异常行为模式

**实现策略**:
```rust
pub struct AnomalyDetector {
    request_history: VecDeque<RequestRecord>,
    baseline_metrics: BaselineMetrics,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct RequestRecord {
    timestamp: DateTime<Utc>,
    user_id: Option<String>,
    ip_address: String,
    endpoint: String,
    response_time: Duration,
    status_code: u16,
}

impl AnomalyDetector {
    pub fn new() -> Self {
        Self {
            request_history: VecDeque::new(),
            baseline_metrics: BaselineMetrics::new(),
            alert_thresholds: AlertThresholds::default(),
        }
    }
    
    pub fn record_request(&mut self, record: RequestRecord) -> Option<SecurityAlert> {
        self.request_history.push_back(record.clone());
        
        // 保持历史记录大小
        if self.request_history.len() > 10000 {
            self.request_history.pop_front();
        }
        
        // 检测异常
        self.detect_anomalies(&record)
    }
    
    fn detect_anomalies(&self, record: &RequestRecord) -> Option<SecurityAlert> {
        // 检测异常请求频率
        if self.is_abnormal_request_rate(record) {
            return Some(SecurityAlert::new(
                SecurityEventType::SuspiciousActivity,
                SecuritySeverity::High,
                "Abnormal request rate detected",
            ));
        }
        
        // 检测异常响应时间
        if self.is_abnormal_response_time(record) {
            return Some(SecurityAlert::new(
                SecurityEventType::SuspiciousActivity,
                SecuritySeverity::Medium,
                "Abnormal response time detected",
            ));
        }
        
        // 检测异常错误率
        if self.is_abnormal_error_rate(record) {
            return Some(SecurityAlert::new(
                SecurityEventType::SuspiciousActivity,
                SecuritySeverity::Medium,
                "Abnormal error rate detected",
            ));
        }
        
        None
    }
    
    fn is_abnormal_request_rate(&self, record: &RequestRecord) -> bool {
        let recent_requests: Vec<_> = self.request_history
            .iter()
            .filter(|r| r.timestamp > record.timestamp - chrono::Duration::minutes(5))
            .collect();
        
        let rate = recent_requests.len() as f64 / 5.0;
        rate > self.alert_thresholds.max_requests_per_minute
    }
    
    fn is_abnormal_response_time(&self, record: &RequestRecord) -> bool {
        record.response_time > self.alert_thresholds.max_response_time
    }
    
    fn is_abnormal_error_rate(&self, record: &RequestRecord) -> bool {
        if !record.status_code.is_server_error() {
            return false;
        }
        
        let recent_errors: Vec<_> = self.request_history
            .iter()
            .filter(|r| {
                r.timestamp > record.timestamp - chrono::Duration::minutes(5)
                    && r.status_code.is_server_error()
            })
            .collect();
        
        let error_rate = recent_errors.len() as f64 / 5.0;
        error_rate > self.alert_thresholds.max_error_rate_per_minute
    }
}
```

## 性能监控

### 1. 关键性能指标

#### 1.1 响应时间指标
```rust
pub struct ResponseTimeMetrics {
    p50: f64,
    p95: f64,
    p99: f64,
    p999: f64,
    average: f64,
    min: f64,
    max: f64,
}

impl ResponseTimeMetrics {
    pub fn calculate(response_times: &[Duration]) -> Self {
        let mut times: Vec<f64> = response_times
            .iter()
            .map(|d| d.as_secs_f64())
            .collect();
        
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        Self {
            p50: Self::percentile(&times, 50.0),
            p95: Self::percentile(&times, 95.0),
            p99: Self::percentile(&times, 99.0),
            p999: Self::percentile(&times, 99.9),
            average: times.iter().sum::<f64>() / times.len() as f64,
            min: times.first().copied().unwrap_or(0.0),
            max: times.last().copied().unwrap_or(0.0),
        }
    }
    
    fn percentile(sorted_times: &[f64], p: f64) -> f64 {
        if sorted_times.is_empty() {
            return 0.0;
        }
        
        let index = ((p / 100.0) * (sorted_times.len() - 1) as f64) as usize;
        sorted_times[index]
    }
}
```

#### 1.2 吞吐量指标
```rust
pub struct ThroughputMetrics {
    requests_per_second: f64,
    validations_per_second: f64,
    cache_hit_rate: f64,
    error_rate: f64,
}

impl ThroughputMetrics {
    pub fn calculate(
        request_count: usize,
        validation_count: usize,
        cache_hits: usize,
        error_count: usize,
        duration: Duration,
    ) -> Self {
        let seconds = duration.as_secs_f64();
        
        Self {
            requests_per_second: request_count as f64 / seconds,
            validations_per_second: validation_count as f64 / seconds,
            cache_hit_rate: if validation_count > 0 {
                cache_hits as f64 / validation_count as f64
            } else {
                0.0
            },
            error_rate: if request_count > 0 {
                error_count as f64 / request_count as f64
            } else {
                0.0
            },
        }
    }
}
```

### 2. 性能优化建议

#### 2.1 缓存优化建议
- **Schema缓存**: 确保热点Schema常驻内存
- **验证结果缓存**: 优化缓存键生成策略
- **分布式缓存**: 考虑使用Redis集群提高缓存可用性

#### 2.2 并发优化建议
- **工作线程数**: 根据CPU核心数调整工作线程数
- **连接池大小**: 根据负载调整数据库和Redis连接池大小
- **异步I/O**: 确保所有I/O操作都是异步的

#### 2.3 内存优化建议
- **内存池**: 使用对象池减少内存分配
- **字符串去重**: 对重复字符串进行去重处理
- **零拷贝**: 尽可能使用零拷贝操作

## 安全最佳实践

### 1. 定期安全审计
- **代码审计**: 定期进行代码安全审计
- **依赖检查**: 定期检查依赖库的安全漏洞
- **渗透测试**: 定期进行渗透测试

### 2. 安全配置
- **最小权限原则**: 应用最小权限原则
- **安全配置**: 使用安全的服务器配置
- **密钥管理**: 使用安全的密钥管理方案

### 3. 应急响应
- **安全事件响应**: 制定安全事件响应计划
- **备份和恢复**: 定期备份关键数据
- **灾难恢复**: 制定灾难恢复计划

这个综合的性能优化和安全策略确保了JSON验证MCP服务器在高负载下的稳定性和安全性，同时提供了企业级的安全防护机制。