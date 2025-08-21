use serde::{Deserialize, Serialize};
use config::{Config, ConfigError, File, Environment as ConfigEnv};
use std::path::PathBuf;
use crate::errors::AppError;
use std::env;

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
    pub enable_wal_mode: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: u64,
    pub cache_size: i64,
    pub mmap_size: i64,
    pub page_size: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://tasks.db".to_string(),
            max_connections: 100,
            min_connections: 10,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
            enable_wal_mode: true,
            enable_foreign_keys: true,
            busy_timeout: 30,
            cache_size: -64000, // 64MB
            mmap_size: 268435456, // 256MB
            page_size: 4096,
        }
    }
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: usize,
    pub timeout: u64,
    pub max_request_size: u64,
    pub enable_cors: bool,
    pub enable_compression: bool,
    pub enable_request_id: bool,
    pub enable_tracing: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            workers: 4,
            timeout: 30,
            max_request_size: 10 * 1024 * 1024, // 10MB
            enable_cors: true,
            enable_compression: true,
            enable_request_id: true,
            enable_tracing: true,
        }
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<PathBuf>,
    pub max_file_size: u64,
    pub max_backup_count: usize,
    pub enable_json: bool,
    pub enable_pretty: bool,
    pub targets: Vec<LogTarget>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Json,
            file: None,
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_backup_count: 5,
            enable_json: true,
            enable_pretty: false,
            targets: vec![LogTarget::Stdout],
        }
    }
}

/// 日志格式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
    Compact,
}

/// 日志目标
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogTarget {
    Stdout,
    Stderr,
    File,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_auth: bool,
    pub api_key_required: bool,
    pub api_keys: Vec<String>,
    pub enable_cors: bool,
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub rate_limit_enabled: bool,
    pub rate_limit_requests_per_minute: u32,
    pub rate_limit_burst_size: u32,
    pub enable_https: bool,
    pub tls_cert_path: Option<PathBuf>,
    pub tls_key_path: Option<PathBuf>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_auth: true,
            api_key_required: true,
            api_keys: vec![],
            enable_cors: true,
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
            allowed_headers: vec!["*".to_string()],
            rate_limit_enabled: true,
            rate_limit_requests_per_minute: 1000,
            rate_limit_burst_size: 100,
            enable_https: false,
            tls_cert_path: None,
            tls_key_path: None,
        }
    }
}

/// 任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub max_concurrent_tasks: u32,
    pub default_task_timeout: u64,
    pub max_task_retries: u32,
    pub task_cleanup_interval: u64,
    pub task_result_ttl: u64,
    pub enable_priority_scheduling: bool,
    pub enable_fair_scheduling: bool,
    pub worker_timeout: u64,
    pub heartbeat_interval: u64,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            default_task_timeout: 3600,
            max_task_retries: 3,
            task_cleanup_interval: 3600,
            task_result_ttl: 2592000, // 30 days
            enable_priority_scheduling: true,
            enable_fair_scheduling: true,
            worker_timeout: 300,
            heartbeat_interval: 30,
        }
    }
}

/// 监控配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_endpoint: String,
    pub enable_health_check: bool,
    pub health_check_endpoint: String,
    pub enable_prometheus: bool,
    pub prometheus_endpoint: String,
    pub enable_tracing: bool,
    pub tracing_endpoint: Option<String>,
    pub metrics_collection_interval: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_metrics: true,
            metrics_endpoint: "/metrics".to_string(),
            enable_health_check: true,
            health_check_endpoint: "/health".to_string(),
            enable_prometheus: true,
            prometheus_endpoint: "/prometheus".to_string(),
            enable_tracing: false,
            tracing_endpoint: None,
            metrics_collection_interval: 60,
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enable_cache: bool,
    pub cache_type: CacheType,
    pub cache_ttl: u64,
    pub cache_size: u64,
    pub redis_url: Option<String>,
    pub memory_cache_size: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_cache: true,
            cache_type: CacheType::Memory,
            cache_ttl: 300,
            cache_size: 1000,
            redis_url: None,
            memory_cache_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// 缓存类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Memory,
    Redis,
}

/// 外部服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalServiceConfig {
    pub enable_external_services: bool,
    pub services: std::collections::HashMap<String, ExternalService>,
}

impl Default for ExternalServiceConfig {
    fn default() -> Self {
        Self {
            enable_external_services: false,
            services: std::collections::HashMap::new(),
        }
    }
}

/// 外部服务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalService {
    pub url: String,
    pub timeout: u64,
    pub retries: u32,
    pub api_key: Option<String>,
    pub enable_circuit_breaker: bool,
    pub circuit_breaker_threshold: u32,
    pub circuit_breaker_timeout: u64,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub task: TaskConfig,
    pub monitoring: MonitoringConfig,
    pub cache: CacheConfig,
    pub external_services: ExternalServiceConfig,
    pub environment: Environment,
    pub debug: bool,
    pub version: String,
}

impl AppConfig {
    /// 从环境变量加载配置
    pub fn from_env() -> Result<Self, ConfigError> {
        // 确定环境
        let environment = match std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()).as_str() {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            "development" => Environment::Development,
            "test" => Environment::Test,
            _ => Environment::Development,
        };

        let config = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(File::with_name("config/production").required(false))
            .add_source(ConfigEnv::with_prefix("APP").separator("_"))
            .set_default("environment", environment.to_string())?
            .set_default("debug", matches!(environment, Environment::Development))?
            .set_default("version", env!("CARGO_PKG_VERSION"))?
            .build()?;

        config.try_deserialize()
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), AppError> {
        // 验证数据库配置
        if self.database.url.is_empty() {
            return Err(AppError::Configuration(
                ConfigError::Message("Database URL cannot be empty".to_string())
            ));
        }

        if self.database.max_connections < self.database.min_connections {
            return Err(AppError::Configuration(
                ConfigError::Message("Max connections must be greater than or equal to min connections".to_string())
            ));
        }

        // 验证服务器配置
        if self.server.port == 0 {
            return Err(AppError::Configuration(
                ConfigError::Message("Server port cannot be zero".to_string())
            ));
        }

        if self.server.workers == 0 {
            return Err(AppError::Configuration(
                ConfigError::Message("Server workers cannot be zero".to_string())
            ));
        }

        // 验证安全配置
        if self.security.enable_auth && self.security.api_keys.is_empty() {
            return Err(AppError::Configuration(
                ConfigError::Message("API keys are required when authentication is enabled".to_string())
            ));
        }

        // 验证任务配置
        if self.task.max_concurrent_tasks == 0 {
            return Err(AppError::Configuration(
                ConfigError::Message("Max concurrent tasks cannot be zero".to_string())
            ));
        }

        if self.task.default_task_timeout == 0 {
            return Err(AppError::Configuration(
                ConfigError::Message("Default task timeout cannot be zero".to_string())
            ));
        }

        // 验证缓存配置
        if self.cache.enable_cache && self.cache.cache_type == CacheType::Redis && self.cache.redis_url.is_none() {
            return Err(AppError::Configuration(
                ConfigError::Message("Redis URL is required when Redis cache is enabled".to_string())
            ));
        }

        Ok(())
    }

    /// 获取数据库连接字符串
    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    /// 获取服务器地址
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// 是否为开发环境
    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    /// 是否为生产环境
    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    /// 是否为测试环境
    pub fn is_test(&self) -> bool {
        matches!(self.environment, Environment::Test)
    }
}

/// 环境类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Test,
    Staging,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Test => write!(f, "test"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Development
    }
}

/// 配置管理器
pub struct ConfigManager {
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器
    pub fn new() -> Result<Self, AppError> {
        let config = AppConfig::from_env()?;
        config.validate()?;
        Ok(Self { config })
    }

    /// 从配置创建
    pub fn from_config(config: AppConfig) -> Result<Self, AppError> {
        config.validate()?;
        Ok(Self { config })
    }

    /// 获取配置
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// 获取数据库配置
    pub fn database(&self) -> &DatabaseConfig {
        &self.config.database
    }

    /// 获取服务器配置
    pub fn server(&self) -> &ServerConfig {
        &self.config.server
    }

    /// 获取日志配置
    pub fn logging(&self) -> &LoggingConfig {
        &self.config.logging
    }

    /// 获取安全配置
    pub fn security(&self) -> &SecurityConfig {
        &self.config.security
    }

    /// 获取任务配置
    pub fn task(&self) -> &TaskConfig {
        &self.config.task
    }

    /// 获取监控配置
    pub fn monitoring(&self) -> &MonitoringConfig {
        &self.config.monitoring
    }

    /// 获取缓存配置
    pub fn cache(&self) -> &CacheConfig {
        &self.config.cache
    }

    /// 重新加载配置
    pub fn reload(&mut self) -> Result<(), AppError> {
        self.config = AppConfig::from_env()?;
        self.config.validate()?;
        Ok(())
    }
}

impl std::ops::Deref for ConfigManager {
    type Target = AppConfig;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::from_env().unwrap();
        assert!(!config.database.url.is_empty());
        assert!(config.server.port > 0);
        assert!(config.server.workers > 0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig::from_env().unwrap();
        
        // 测试无效配置
        config.database.url = "".to_string();
        assert!(config.validate().is_err());
        
        config.database.url = "sqlite:///test.db".to_string();
        config.database.max_connections = 5;
        config.database.min_connections = 10;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_detection() {
        std::env::set_var("APP_ENV", "production");
        let config = AppConfig::from_env().unwrap();
        assert!(config.is_production());
        
        std::env::set_var("APP_ENV", "development");
        let config = AppConfig::from_env().unwrap();
        assert!(config.is_development());
        
        std::env::set_var("APP_ENV", "test");
        let config = AppConfig::from_env().unwrap();
        assert!(config.is_test());
    }
}