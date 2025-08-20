//! 服务器配置管理

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::tls::TlsVersion;
use crate::performance::PerformanceConfig as OptimizedPerformanceConfig;

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务器配置
    pub server: ServerSettings,
    /// 缓存配置
    pub cache: CacheConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 指标配置
    pub metrics: MetricsConfig,
    /// 监控配置
    pub monitoring: MonitoringConfig,
    /// JSON验证配置
    pub validation: ValidationConfig,
    /// 性能配置
    pub performance: PerformanceConfig,
    /// 部署配置
    pub deployment: DeploymentConfig,
    /// 数据库配置
    pub database: DatabaseConfig,
    /// Redis配置
    pub redis: RedisConfig,
    /// 通知配置
    pub notifications: NotificationConfig,
    /// 备份配置
    pub backup: BackupConfig,
    /// 审计配置
    pub audit: AuditConfig,
}

/// 服务器基础设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerSettings {
    /// 监听地址
    pub host: String,
    /// 监听端口
    pub port: u16,
    /// 工作线程数
    pub workers: usize,
    /// 最大连接数
    pub max_connections: usize,
    /// 请求超时时间（秒）
    pub timeout: u64,
    /// 请求体大小限制（字节）
    pub max_request_size: usize,
    /// 是否启用压缩
    pub compression: bool,
    /// 是否启用HTTPS
    pub https_enabled: bool,
    /// HTTPS证书路径
    pub cert_path: String,
    /// HTTPS私钥路径
    pub key_path: String,
    /// 是否强制客户端证书验证
    pub client_auth_required: bool,
    /// 客户端CA证书路径
    pub client_ca_path: String,
}

impl Default for ServerSettings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            workers: 4,
            max_connections: 1000,
            timeout: 30,
            max_request_size: 10 * 1024 * 1024, // 10MB
            compression: true,
            https_enabled: false,
            cert_path: "certs/server.crt".to_string(),
            key_path: "certs/server.key".to_string(),
            client_auth_required: false,
            client_ca_path: "certs/client-ca.crt".to_string(),
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存类型 (redis, lru)
    pub cache_type: String,
    /// Redis连接URL
    pub redis_url: Option<String>,
    /// 缓存TTL（秒）
    pub ttl: u64,
    /// 最大缓存条目数
    pub max_size: usize,
    /// 缓存键前缀
    pub key_prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_type: "lru".to_string(),
            redis_url: None,
            ttl: 3600,
            max_size: 1000,
            key_prefix: "json_validator:".to_string(),
        }
    }
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// 是否启用安全功能
    pub enabled: bool,
    /// JWT密钥
    pub jwt_secret: String,
    /// JWT过期时间（秒）
    pub jwt_expiry: u64,
    /// 速率限制（每分钟请求数）
    pub rate_limit: u32,
    /// 最大请求体大小（字节）
    pub max_body_size: usize,
    /// 是否启用API密钥认证
    pub api_key_enabled: bool,
    /// API密钥前缀
    pub api_key_prefix: String,
    /// 是否启用严格安全模式
    pub strict_mode: bool,
    /// CORS配置
    pub cors: CorsConfig,
    /// 速率限制配置
    pub rate_limiting: RateLimitingConfig,
    /// API密钥配置
    pub api_keys: HashMap<String, ApiKeyConfig>,
    /// TLS配置
    pub tls: TlsConfig,
    /// IP白名单配置
    pub allowed_ips: IpWhitelistConfig,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut api_keys = HashMap::new();
        api_keys.insert("admin_key".to_string(), ApiKeyConfig {
            name: "Admin Key".to_string(),
            permissions: vec!["read".to_string(), "write".to_string(), "admin".to_string()],
            rate_limit: Some(1000),
        });
        api_keys.insert("user_key".to_string(), ApiKeyConfig {
            name: "User Key".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            rate_limit: Some(100),
        });
        api_keys.insert("readonly_key".to_string(), ApiKeyConfig {
            name: "Readonly Key".to_string(),
            permissions: vec!["read".to_string()],
            rate_limit: Some(50),
        });

        Self {
            enabled: true,
            jwt_secret: "your-secret-key-here-change-in-production".to_string(),
            jwt_expiry: 86400,
            rate_limit: 100,
            max_body_size: 10 * 1024 * 1024,
            api_key_enabled: true,
            api_key_prefix: "json-val".to_string(),
            strict_mode: true,
            cors: CorsConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
            api_keys,
            tls: TlsConfig::default(),
            allowed_ips: IpWhitelistConfig::default(),
        }
    }
}

/// API密钥配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// 密钥名称
    pub name: String,
    /// 权限列表
    pub permissions: Vec<String>,
    /// 速率限制
    pub rate_limit: Option<u32>,
}

/// CORS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// 是否启用CORS
    pub enabled: bool,
    /// 允许的源
    pub allow_origins: Vec<String>,
    /// 允许的方法
    pub allow_methods: Vec<String>,
    /// 允许的头部
    pub allow_headers: Vec<String>,
    /// 是否允许凭证
    pub allow_credentials: bool,
    /// 预检请求缓存时间（秒）
    pub max_age: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string(), "OPTIONS".to_string()],
            allow_headers: vec!["*".to_string()],
            allow_credentials: false,
            max_age: 86400,
        }
    }
}

/// 速率限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// 限流算法
    pub algorithm: String,
    /// 全局限流配置
    pub global_limit: u32,
    /// IP白名单
    pub whitelist: Vec<String>,
    /// 限流错误消息
    pub error_message: String,
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            algorithm: "token_bucket".to_string(),
            global_limit: 1000,
            whitelist: vec!["127.0.0.1".to_string(), "::1".to_string()],
            error_message: "Rate limit exceeded. Please try again later.".to_string(),
        }
    }
}

/// TLS配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// 最小TLS版本
    pub min_version: String,
    /// 密码套件
    pub cipher_suites: Vec<String>,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            min_version: "tls1_2".to_string(),
            cipher_suites: vec![
                "TLS13_AES_256_GCM_SHA384".to_string(),
                "TLS13_AES_128_GCM_SHA256".to_string(),
                "TLS_CHACHA20_POLY1305_SHA256".to_string(),
                "TLS_AES_256_GCM_SHA384".to_string(),
                "TLS_AES_128_GCM_SHA256".to_string(),
            ],
        }
    }
}

/// IP白名单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpWhitelistConfig {
    /// IP地址列表
    pub ips: Vec<String>,
    /// 是否启用IP白名单
    pub enabled: bool,
}

impl Default for IpWhitelistConfig {
    fn default() -> Self {
        Self {
            ips: vec!["127.0.0.1".to_string(), "::1".to_string()],
            enabled: false,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 日志格式 (json, text)
    pub format: String,
    /// 日志文件路径
    pub file_path: Option<PathBuf>,
    /// 是否输出到标准输出
    pub stdout: bool,
    /// 是否输出到标准错误
    pub stderr: bool,
    /// 日志轮转配置
    pub rotation: LogRotationConfig,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "json".to_string(),
            file_path: None,
            stdout: true,
            stderr: false,
            rotation: LogRotationConfig::default(),
        }
    }
}

/// 日志轮转配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// 是否启用日志轮转
    pub enabled: bool,
    /// 最大文件大小（字节）
    pub max_size: u64,
    /// 最大保留文件数
    pub max_files: usize,
    /// 轮转周期
    pub rotation: String,
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            rotation: "daily".to_string(),
        }
    }
}

/// 指标配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// 是否启用指标收集
    pub enabled: bool,
    /// 指标端口
    pub port: u16,
    /// 指标路径
    pub path: String,
    /// 指标收集间隔（秒）
    pub interval: u64,
    /// 是否启用Prometheus指标
    pub prometheus_enabled: bool,
    /// 是否启用详细指标
    pub detailed_metrics: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            port: 9090,
            path: "/metrics".to_string(),
            interval: 15,
            prometheus_enabled: true,
            detailed_metrics: true,
        }
    }
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// 健康检查间隔（秒）
    pub health_check_interval: u64,
    /// 是否启用性能监控
    pub performance_monitoring: bool,
    /// 是否启用错误追踪
    pub error_tracking: bool,
    /// 是否启用请求追踪
    pub request_tracing: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            health_check_interval: 30,
            performance_monitoring: true,
            error_tracking: true,
            request_tracing: true,
        }
    }
}

/// JSON验证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// 最大JSON大小（字节）
    pub max_json_size: usize,
    /// 最大Schema大小（字节）
    pub max_schema_size: usize,
    /// 是否启用严格模式
    pub strict_mode: bool,
    /// 是否启用自定义格式验证
    pub enable_custom_formats: bool,
    /// 验证超时时间（毫秒）
    pub timeout: u64,
    /// 并发验证限制
    pub max_concurrent: usize,
    /// 是否启用验证缓存
    pub cache_validation: bool,
    /// 是否启用详细错误信息
    pub detailed_errors: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            max_json_size: 10 * 1024 * 1024, // 10MB
            max_schema_size: 1 * 1024 * 1024, // 1MB
            strict_mode: false,
            enable_custom_formats: false,
            timeout: 5000, // 5秒
            max_concurrent: 100,
            cache_validation: true,
            detailed_errors: true,
        }
    }
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 基础性能配置
    pub basic: BasicPerformanceConfig,
    /// 优化性能配置
    pub optimized: OptimizedPerformanceConfig,
    /// 内存管理配置
    pub memory: MemoryConfig,
    /// 并发控制配置
    pub concurrency: ConcurrencyConfig,
    /// 缓存优化配置
    pub cache_optimization: CacheOptimizationConfig,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            basic: BasicPerformanceConfig::default(),
            optimized: OptimizedPerformanceConfig::default(),
            memory: MemoryConfig::default(),
            concurrency: ConcurrencyConfig::default(),
            cache_optimization: CacheOptimizationConfig::default(),
        }
    }
}

/// 基础性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicPerformanceConfig {
    /// 连接池大小
    pub connection_pool_size: usize,
    /// 请求队列大小
    pub request_queue_size: usize,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 压缩级别
    pub compression_level: u32,
    /// 是否启用缓存
    pub enable_caching: bool,
}

impl Default for BasicPerformanceConfig {
    fn default() -> Self {
        Self {
            connection_pool_size: 10,
            request_queue_size: 100,
            enable_compression: true,
            compression_level: 6,
            enable_caching: true,
        }
    }
}

/// 内存管理配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 内存限制 (MB)
    pub memory_limit_mb: usize,
    /// 是否启用内存池
    pub enable_memory_pool: bool,
    /// 内存池大小
    pub memory_pool_size: usize,
    /// 是否启用垃圾回收优化
    pub enable_gc_optimization: bool,
    /// 内存分配策略
    pub allocation_strategy: String,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_limit_mb: 512,
            enable_memory_pool: true,
            memory_pool_size: 1000,
            enable_gc_optimization: true,
            allocation_strategy: "adaptive".to_string(),
        }
    }
}

/// 并发控制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcurrencyConfig {
    /// 最大并发请求数
    pub max_concurrent_requests: usize,
    /// 工作线程数
    pub worker_threads: usize,
    /// 请求超时时间
    pub request_timeout_secs: u64,
    /// 是否启用连接复用
    pub enable_connection_reuse: bool,
    /// 是否启用异步处理
    pub enable_async_processing: bool,
}

impl Default for ConcurrencyConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 1000,
            worker_threads: 4,
            request_timeout_secs: 30,
            enable_connection_reuse: true,
            enable_async_processing: true,
        }
    }
}

/// 缓存优化配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOptimizationConfig {
    /// 缓存预热
    pub cache_warmup: bool,
    /// 缓存淘汰策略
    pub eviction_policy: String,
    /// 缓存压缩
    pub enable_cache_compression: bool,
    /// 缓存分片
    pub cache_sharding: bool,
    /// 缓存分片数量
    pub cache_shards: usize,
}

impl Default for CacheOptimizationConfig {
    fn default() -> Self {
        Self {
            cache_warmup: true,
            eviction_policy: "lru".to_string(),
            enable_cache_compression: false,
            cache_sharding: false,
            cache_shards: 4,
        }
    }
}

/// 部署配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    /// 环境类型
    pub environment: String,
    /// 是否启用调试模式
    pub debug_mode: bool,
    /// 是否启用热重载
    pub hot_reload: bool,
    /// 是否启用优雅关闭
    pub graceful_shutdown: bool,
    /// 关闭超时时间（秒）
    pub shutdown_timeout: u64,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            debug_mode: false,
            hot_reload: false,
            graceful_shutdown: true,
            shutdown_timeout: 30,
        }
    }
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 是否启用数据库
    pub enabled: bool,
    /// 数据库URL
    pub url: String,
    /// 连接池大小
    pub pool_size: u32,
    /// 最大连接数
    pub max_connections: u32,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "postgresql://username:password@localhost/json_validator".to_string(),
            pool_size: 5,
            max_connections: 10,
            connection_timeout: 30,
        }
    }
}

/// Redis配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// 是否启用Redis
    pub enabled: bool,
    /// Redis URL
    pub url: String,
    /// 数据库编号
    pub db: i64,
    /// 连接池大小
    pub pool_size: u32,
    /// 连接超时时间（秒）
    pub connection_timeout: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: "redis://localhost:6379".to_string(),
            db: 0,
            pool_size: 5,
            connection_timeout: 5,
        }
    }
}

/// 通知配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// 是否启用通知
    pub enabled: bool,
    /// 通知类型
    pub types: Vec<String>,
    /// 告警阈值
    pub alert_thresholds: AlertThresholds,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            types: vec!["email".to_string(), "webhook".to_string(), "slack".to_string()],
            alert_thresholds: AlertThresholds::default(),
        }
    }
}

/// 告警阈值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    /// 错误率阈值
    pub error_rate: f64,
    /// 响应时间阈值（毫秒）
    pub response_time: u64,
    /// 内存使用率阈值
    pub memory_usage: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            error_rate: 0.1,
            response_time: 1000,
            memory_usage: 0.8,
        }
    }
}

/// 备份配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// 是否启用备份
    pub enabled: bool,
    /// 备份间隔（小时）
    pub interval: u64,
    /// 备份保留天数
    pub retention_days: u64,
    /// 备份存储路径
    pub storage_path: String,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: 24,
            retention_days: 30,
            storage_path: "/var/backups/json-validator".to_string(),
        }
    }
}

/// 审计配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// 是否启用审计日志
    pub enabled: bool,
    /// 审计日志文件路径
    pub file_path: String,
    /// 审计事件类型
    pub event_types: Vec<String>,
    /// 是否启用详细审计
    pub detailed: bool,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            file_path: "/var/log/json-validator-audit.log".to_string(),
            event_types: vec![
                "authentication".to_string(),
                "authorization".to_string(),
                "validation".to_string(),
                "error".to_string(),
            ],
            detailed: true,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            server: ServerSettings::default(),
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            metrics: MetricsConfig::default(),
            monitoring: MonitoringConfig::default(),
            validation: ValidationConfig::default(),
            performance: PerformanceConfig::default(),
            deployment: DeploymentConfig::default(),
            database: DatabaseConfig::default(),
            redis: RedisConfig::default(),
            notifications: NotificationConfig::default(),
            backup: BackupConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

impl ServerConfig {
    /// 获取服务器监听地址
    pub fn listen_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// 获取指标监听地址
    pub fn metrics_address(&self) -> String {
        format!("{}:{}", self.server.host, self.metrics.port)
    }

    /// 获取TLS版本
    pub fn get_tls_version(&self) -> Result<TlsVersion, anyhow::Error> {
        match self.security.tls.min_version.as_str() {
            "tls1_2" => Ok(TlsVersion::Tls1_2),
            "tls1_3" => Ok(TlsVersion::Tls1_3),
            _ => Err(anyhow::anyhow!("Unsupported TLS version: {}", self.security.tls.min_version)),
        }
    }

    /// 获取性能配置
    pub fn get_performance_config(&self) -> OptimizedPerformanceConfig {
        OptimizedPerformanceConfig {
            max_concurrent_requests: self.performance.concurrency.max_concurrent_requests,
            connection_pool_size: self.performance.basic.connection_pool_size,
            cache_size: self.cache.max_size,
            memory_limit_mb: self.performance.memory.memory_limit_mb,
            request_timeout: std::time::Duration::from_secs(self.performance.concurrency.request_timeout_secs),
            enable_memory_pool: self.performance.memory.enable_memory_pool,
            enable_connection_reuse: self.performance.concurrency.enable_connection_reuse,
            enable_compression: self.performance.basic.enable_compression,
        }
    }

    /// 验证配置有效性
    pub fn validate(&self) -> anyhow::Result<()> {
        // 服务器配置验证
        if self.server.workers == 0 {
            return Err(anyhow::anyhow!("Workers must be greater than 0"));
        }

        if self.server.max_connections == 0 {
            return Err(anyhow::anyhow!("Max connections must be greater than 0"));
        }

        if self.server.timeout == 0 {
            return Err(anyhow::anyhow!("Timeout must be greater than 0"));
        }

        // 安全配置验证
        if self.security.enabled {
            if self.security.jwt_secret == "your-secret-key-here-change-in-production" && 
               self.deployment.environment == "production" {
                return Err(anyhow::anyhow!("JWT secret must be changed in production"));
            }

            if self.security.api_key_enabled && self.security.api_keys.is_empty() {
                return Err(anyhow::anyhow!("API keys must be configured when API key authentication is enabled"));
            }
        }

        // 验证配置验证
        if self.validation.max_concurrent == 0 {
            return Err(anyhow::anyhow!("Max concurrent validations must be greater than 0"));
        }

        // 性能配置验证
        if self.performance.basic.connection_pool_size == 0 {
            return Err(anyhow::anyhow!("Connection pool size must be greater than 0"));
        }

        if self.performance.concurrency.max_concurrent_requests == 0 {
            return Err(anyhow::anyhow!("Max concurrent requests must be greater than 0"));
        }

        // 部署配置验证
        if !["development", "staging", "production"].contains(&self.deployment.environment.as_str()) {
            return Err(anyhow::anyhow!("Invalid environment: {}", self.deployment.environment));
        }

        // 数据库配置验证
        if self.database.enabled && self.database.url.is_empty() {
            return Err(anyhow::anyhow!("Database URL must be provided when database is enabled"));
        }

        // Redis配置验证
        if self.redis.enabled && self.redis.url.is_empty() {
            return Err(anyhow::anyhow!("Redis URL must be provided when Redis is enabled"));
        }

        Ok(())
    }

    /// 检查是否为生产环境
    pub fn is_production(&self) -> bool {
        self.deployment.environment == "production"
    }

    /// 检查是否为开发环境
    pub fn is_development(&self) -> bool {
        self.deployment.environment == "development"
    }

    /// 获取环境特定的配置
    pub fn get_environment_config(&self) -> EnvironmentConfig {
        EnvironmentConfig {
            is_production: self.is_production(),
            is_development: self.is_development(),
            debug_mode: self.deployment.debug_mode,
            strict_mode: self.security.strict_mode,
        }
    }
}

/// 环境特定配置
#[derive(Debug, Clone)]
pub struct EnvironmentConfig {
    /// 是否为生产环境
    pub is_production: bool,
    /// 是否为开发环境
    pub is_development: bool,
    /// 是否启用调试模式
    pub debug_mode: bool,
    /// 是否启用严格模式
    pub strict_mode: bool,
}

impl EnvironmentConfig {
    /// 获取日志级别
    pub fn get_log_level(&self) -> &str {
        if self.debug_mode {
            "debug"
        } else if self.is_production {
            "info"
        } else {
            "debug"
        }
    }

    /// 获取是否启用详细日志
    pub fn enable_detailed_logging(&self) -> bool {
        self.debug_mode || !self.is_production
    }

    /// 获取是否启用性能监控
    pub fn enable_performance_monitoring(&self) -> bool {
        !self.is_development || self.debug_mode
    }
}