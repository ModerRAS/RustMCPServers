//! # 配置管理模块
//! 
//! 该模块提供了 Simple Task Orchestrator 的完整配置管理功能。
//! 
//! ## 主要功能
//! 
//! - **应用配置**: 集中管理所有配置项
//! - **环境变量支持**: 支持通过环境变量覆盖配置
//! - **默认值**: 提供合理的默认配置
//! - **类型安全**: 使用 Rust 类型系统确保配置的正确性
//! - **序列化支持**: 支持 TOML、JSON 等格式的配置文件
//! 
//! ## 配置结构
//! 
//! 配置采用分层结构设计：
//! 
//! - `AppConfig`: 主配置结构，包含所有子配置
//! - `ServerConfig`: 服务器相关配置
//! - `TaskConfig`: 任务执行相关配置
//! - `DatabaseConfig`: 数据库连接配置
//! - `LoggingConfig`: 日志记录配置
//! - `SecurityConfig`: 安全相关配置
//! - `MonitoringConfig`: 监控相关配置
//! 
//! ## 使用示例
//! 
//! ```rust
//! use simple_task_orchestrator::config::{AppConfig, ConfigManager};
//! 
//! // 从环境变量创建配置
//! let config = AppConfig::from_env()?;
//! 
//! // 使用配置管理器
//! let manager = ConfigManager::new()?;
//! let server_config = manager.server_config();
//! let task_config = manager.task_config();
//! ```
//! 
//! ## 环境变量
//! 
//! 支持通过环境变量覆盖配置：
//! 
//! - `APP_SERVER_HOST`: 服务器主机地址
//! - `APP_SERVER_PORT`: 服务器端口
//! - `APP_SERVER_TIMEOUT`: 服务器超时时间
//! - `APP_TASK_MAX_RETRIES`: 任务最大重试次数
//! - `APP_TASK_TIMEOUT`: 任务超时时间
//! - `RUST_LOG`: 日志级别
//! - `APP_SECURITY_RATE_LIMIT`: 安全限流设置
//! - `APP_MONITORING_METRICS_INTERVAL`: 监控指标收集间隔

use serde::{Deserialize, Serialize};
use std::env;

/// 应用主配置结构
/// 
/// 包含服务器的所有配置项，支持从环境变量加载和覆盖默认值。
/// 
/// # 字段说明
/// 
/// - `server`: 服务器相关配置（主机、端口、超时等）
/// - `task`: 任务执行相关配置（重试次数、超时等）
/// - `database`: 数据库连接配置
/// - `logging`: 日志记录配置
/// - `security`: 安全相关配置（API密钥、限流等）
/// - `monitoring`: 监控相关配置
/// 
/// # 示例
/// 
/// ```rust
/// let config = AppConfig::from_env()?;
/// println!("服务器运行在 {}:{}", config.server.host, config.server.port);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub task: TaskConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            task: TaskConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl AppConfig {
    /// 从环境变量创建配置实例
    /// 
    /// 该方法会读取相关的环境变量并覆盖默认配置值。
    /// 如果环境变量不存在或格式不正确，则使用默认值。
    /// 
    /// # 返回值
    /// 
    /// 返回 `Result<AppConfig, String>`，其中：
    /// - `Ok(AppConfig)`: 成功创建的配置实例
    /// - `Err(String)`: 环境变量解析失败时的错误信息
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// std::env::set_var("APP_SERVER_PORT", "8081");
    /// let config = AppConfig::from_env()?;
    /// assert_eq!(config.server.port, 8081);
    /// ```
    pub fn from_env() -> Result<Self, String> {
        let mut config = Self::default();
        
        // 服务器配置
        if let Ok(host) = env::var("APP_SERVER_HOST") {
            config.server.host = host;
        }
        
        if let Ok(port) = env::var("APP_SERVER_PORT") {
            config.server.port = port.parse().map_err(|e| format!("Invalid port: {}", e))?;
        }
        
        if let Ok(timeout) = env::var("APP_SERVER_TIMEOUT") {
            config.server.timeout = timeout.parse().map_err(|e| format!("Invalid timeout: {}", e))?;
        }
        
        // 任务配置
        if let Ok(max_retries) = env::var("APP_TASK_MAX_RETRIES") {
            config.task.max_retries = max_retries.parse().map_err(|e| format!("Invalid max retries: {}", e))?;
        }
        
        if let Ok(timeout) = env::var("APP_TASK_TIMEOUT") {
            config.task.timeout = timeout.parse().map_err(|e| format!("Invalid task timeout: {}", e))?;
        }
        
        if let Ok(cleanup_interval) = env::var("APP_TASK_CLEANUP_INTERVAL") {
            config.task.cleanup_interval = cleanup_interval.parse().map_err(|e| format!("Invalid cleanup interval: {}", e))?;
        }
        
        // 日志配置
        if let Ok(level) = env::var("RUST_LOG") {
            config.logging.level = level;
        }
        
        // 安全配置
        if let Ok(rate_limit) = env::var("APP_SECURITY_RATE_LIMIT") {
            config.security.rate_limit = rate_limit.parse().map_err(|e| format!("Invalid rate limit: {}", e))?;
        }
        
        // 监控配置
        if let Ok(metrics_interval) = env::var("APP_MONITORING_METRICS_INTERVAL") {
            config.monitoring.metrics_interval = metrics_interval.parse().map_err(|e| format!("Invalid metrics interval: {}", e))?;
        }
        
        Ok(config)
    }
}

/// 服务器配置
/// 
/// 定义服务器的网络和运行时配置。
/// 
/// # 字段说明
/// 
/// - `host`: 服务器监听的主机地址
/// - `port`: 服务器监听的端口号
/// - `timeout`: 请求超时时间（秒）
/// - `workers`: 工作线程数量
/// 
/// # 默认值
/// 
/// - `host`: "0.0.0.0" (监听所有接口)
/// - `port`: 8080
/// - `timeout`: 30秒
/// - `workers`: 4个线程
/// 
/// # 示例
/// 
/// ```rust
/// let config = ServerConfig {
///     host: "127.0.0.1".to_string(),
///     port: 3000,
///     timeout: 60,
///     workers: 8,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout: u64,
    pub workers: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            timeout: 30,
            workers: 4,
        }
    }
}

/// 任务配置
/// 
/// 定义任务执行和管理的相关配置。
/// 
/// # 字段说明
/// 
/// - `max_retries`: 任务最大重试次数
/// - `timeout`: 任务执行超时时间（秒）
/// - `cleanup_interval`: 清理已完成任务的间隔（秒）
/// - `heartbeat_interval`: 任务心跳间隔（秒）
/// - `worker_timeout`: 工作线程超时时间（秒）
/// 
/// # 默认值
/// 
/// - `max_retries`: 3次
/// - `timeout`: 3600秒（1小时）
/// - `cleanup_interval`: 3600秒（1小时）
/// - `heartbeat_interval`: 30秒
/// - `worker_timeout`: 1800秒（30分钟）
/// 
/// # 示例
/// 
/// ```rust
/// let config = TaskConfig {
///     max_retries: 5,
///     timeout: 7200,  // 2小时
///     cleanup_interval: 1800,  // 30分钟
///     heartbeat_interval: 15,
///     worker_timeout: 3600,  // 1小时
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub max_retries: u32,
    pub timeout: u64,
    pub cleanup_interval: u64,
    pub heartbeat_interval: u64,
    pub worker_timeout: u64,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            timeout: 3600,
            cleanup_interval: 3600,
            heartbeat_interval: 30,
            worker_timeout: 1800,
        }
    }
}

/// 数据库配置
/// 
/// 定义数据库连接池和连接管理的相关配置。
/// 
/// # 字段说明
/// 
/// - `url`: 数据库连接URL
/// - `max_connections`: 最大连接数
/// - `min_connections`: 最小连接数
/// - `connection_timeout`: 连接超时时间（秒）
/// - `idle_timeout`: 空闲连接超时时间（秒）
/// - `max_lifetime`: 连接最大生命周期（秒）
/// 
/// # 默认值
/// 
/// - `url`: "sqlite:///data/tasks.db"
/// - `max_connections`: 10
/// - `min_connections`: 1
/// - `connection_timeout`: 30秒
/// - `idle_timeout`: 600秒（10分钟）
/// - `max_lifetime`: 3600秒（1小时）
/// 
/// # 示例
/// 
/// ```rust
/// let config = DatabaseConfig {
///     url: "postgres://user:pass@localhost/mydb".to_string(),
///     max_connections: 20,
///     min_connections: 2,
///     connection_timeout: 60,
///     idle_timeout: 1800,  // 30分钟
///     max_lifetime: 7200,  // 2小时
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:///data/tasks.db".to_string(),
            max_connections: 10,
            min_connections: 1,
            connection_timeout: 30,
            idle_timeout: 600,
            max_lifetime: 3600,
        }
    }
}

/// 日志配置
/// 
/// 定义日志记录的相关配置，支持不同级别和格式的日志输出。
/// 
/// # 字段说明
/// 
/// - `level`: 日志级别（trace, debug, info, warn, error）
/// - `format`: 日志格式（JSON或Pretty）
/// - `file`: 日志文件路径，None表示输出到标准输出
/// 
/// # 默认值
/// 
/// - `level`: "info"
/// - `format`: `LogFormat::Json`
/// - `file`: None（输出到标准输出）
/// 
/// # 示例
/// 
/// ```rust
/// let config = LoggingConfig {
///     level: "debug".to_string(),
///     format: LogFormat::Pretty,
///     file: Some("/var/log/app.log".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Json,
            file: None,
        }
    }
}

/// 日志格式枚举
/// 
/// 定义支持的日志输出格式。
/// 
/// # 变体说明
/// 
/// - `Json`: JSON格式的日志，适合机器处理和日志聚合系统
/// - `Pretty`: 美化格式的日志，适合开发调试和人工阅读
/// 
/// # 示例
/// 
/// ```rust
/// let format = LogFormat::Json;  // 机器可读的JSON格式
/// let format = LogFormat::Pretty;  // 人类可读的美化格式
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Json
    }
}

/// 安全配置
/// 
/// 定义服务器的安全相关配置，包括API密钥、限流和CORS设置。
/// 
/// # 字段说明
/// 
/// - `api_key`: API密钥，None表示不启用API密钥验证
/// - `rate_limit`: 请求限流阈值（每秒请求数）
/// - `enable_cors`: 是否启用CORS支持
/// 
/// # 默认值
/// 
/// - `api_key`: None（不启用API密钥验证）
/// - `rate_limit`: 1000（每秒1000个请求）
/// - `enable_cors`: true（启用CORS）
/// 
/// # 示例
/// 
/// ```rust
/// let config = SecurityConfig {
///     api_key: Some("your-api-key".to_string()),
///     rate_limit: 500,  // 每秒500个请求
///     enable_cors: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub api_key: Option<String>,
    pub rate_limit: u32,
    pub enable_cors: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            rate_limit: 1000,
            enable_cors: true,
        }
    }
}

/// 监控配置
/// 
/// 定义服务器监控和健康检查的相关配置。
/// 
/// # 字段说明
/// 
/// - `metrics_enabled`: 是否启用指标收集
/// - `metrics_port`: 指标服务端口
/// - `metrics_interval`: 指标收集间隔（秒）
/// - `health_check_interval`: 健康检查间隔（秒）
/// 
/// # 默认值
/// 
/// - `metrics_enabled`: true（启用指标收集）
/// - `metrics_port`: 9090
/// - `metrics_interval`: 60秒
/// - `health_check_interval`: 30秒
/// 
/// # 示例
/// 
/// ```rust
/// let config = MonitoringConfig {
///     metrics_enabled: true,
///     metrics_port: 8081,
///     metrics_interval: 30,  // 30秒收集一次指标
///     health_check_interval: 10,  // 10秒检查一次健康状态
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub metrics_interval: u64,
    pub health_check_interval: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_port: 9090,
            metrics_interval: 60,
            health_check_interval: 30,
        }
    }
}

/// 配置管理器
/// 
/// 提供配置的统一访问和管理接口。
/// 
/// 该管理器封装了配置的加载和访问逻辑，提供类型安全的配置访问方法。
/// 
/// # 示例
/// 
/// ```rust
/// let manager = ConfigManager::new()?;
/// let server_config = manager.server_config();
/// println!("服务器端口: {}", server_config.port);
/// 
/// let task_config = manager.task_config();
/// println!("最大重试次数: {}", task_config.max_retries);
/// ```
pub struct ConfigManager {
    config: AppConfig,
}

impl ConfigManager {
    /// 创建新的配置管理器实例
    /// 
    /// 该方法会从环境变量加载配置并创建管理器实例。
    /// 
    /// # 返回值
    /// 
    /// 返回 `Result<ConfigManager, String>`，其中：
    /// - `Ok(ConfigManager)`: 成功创建的配置管理器
    /// - `Err(String)`: 配置加载失败时的错误信息
    /// 
    /// # 示例
    /// 
    /// ```rust
    /// let manager = ConfigManager::new()?;
    /// ```
    pub fn new() -> Result<Self, String> {
        let config = AppConfig::from_env()?;
        Ok(Self { config })
    }
    
    /// 获取完整的应用配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&AppConfig`，包含所有配置项
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    
    /// 获取服务器配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&ServerConfig`，包含服务器相关配置
    pub fn server_config(&self) -> &ServerConfig {
        &self.config.server
    }
    
    /// 获取任务配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&TaskConfig`，包含任务执行相关配置
    pub fn task_config(&self) -> &TaskConfig {
        &self.config.task
    }
    
    /// 获取数据库配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&DatabaseConfig`，包含数据库连接配置
    pub fn database_config(&self) -> &DatabaseConfig {
        &self.config.database
    }
    
    /// 获取日志配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&LoggingConfig`，包含日志记录配置
    pub fn logging_config(&self) -> &LoggingConfig {
        &self.config.logging
    }
    
    /// 获取安全配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&SecurityConfig`，包含安全相关配置
    pub fn security_config(&self) -> &SecurityConfig {
        &self.config.security
    }
    
    /// 获取监控配置
    /// 
    /// # 返回值
    /// 
    /// 返回 `&MonitoringConfig`，包含监控相关配置
    pub fn monitoring_config(&self) -> &MonitoringConfig {
        &self.config.monitoring
    }
}