use serde::{Deserialize, Serialize};
use std::env;

/// 应用配置
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

/// 日志格式
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
pub struct ConfigManager {
    config: AppConfig,
}

impl ConfigManager {
    pub fn new() -> Result<Self, String> {
        let config = AppConfig::from_env()?;
        Ok(Self { config })
    }
    
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    
    pub fn server_config(&self) -> &ServerConfig {
        &self.config.server
    }
    
    pub fn task_config(&self) -> &TaskConfig {
        &self.config.task
    }
    
    pub fn database_config(&self) -> &DatabaseConfig {
        &self.config.database
    }
    
    pub fn logging_config(&self) -> &LoggingConfig {
        &self.config.logging
    }
    
    pub fn security_config(&self) -> &SecurityConfig {
        &self.config.security
    }
    
    pub fn monitoring_config(&self) -> &MonitoringConfig {
        &self.config.monitoring
    }
}