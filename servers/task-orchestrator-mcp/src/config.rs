use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(PathBuf),
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("Environment variable not found: {0}")]
    #[allow(dead_code)]
    EnvVarNotFound(String), // Reserved for future use
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub task: TaskConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u32,
    pub max_connections: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            timeout_seconds: 30,
            max_connections: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub enable_wal_mode: bool,
    pub busy_timeout_seconds: u32,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:///data/tasks.db".to_string(),
            max_connections: 20,
            min_connections: 5,
            enable_wal_mode: true,
            busy_timeout_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub file: Option<PathBuf>,
    pub max_size_mb: u64,
    pub max_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            file: None,
            max_size_mb: 100,
            max_files: 5,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    pub max_concurrent_tasks: usize,
    pub default_timeout_seconds: u32,
    pub max_retries: u32,
    pub cleanup_interval_seconds: u32,
    pub heartbeat_interval_seconds: u32,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            default_timeout_seconds: 3600,
            max_retries: 3,
            cleanup_interval_seconds: 3600,
            heartbeat_interval_seconds: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_interval_seconds: u32,
    pub health_check_enabled: bool,
    pub health_check_interval_seconds: u32,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            metrics_interval_seconds: 60,
            health_check_enabled: true,
            health_check_interval_seconds: 30,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
            task: TaskConfig::default(),
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Config::default();

        // Server configuration
        if let Ok(host) = std::env::var("SERVER_HOST") {
            config.server.host = host;
        }
        if let Ok(port_str) = std::env::var("SERVER_PORT") {
            config.server.port = port_str.parse().map_err(|e| {
                ConfigError::Invalid(format!("Invalid SERVER_PORT: {e}"))
            })?;
        }
        if let Ok(timeout_str) = std::env::var("SERVER_TIMEOUT") {
            config.server.timeout_seconds = timeout_str.parse().map_err(|e| {
                ConfigError::Invalid(format!("Invalid SERVER_TIMEOUT: {e}"))
            })?;
        }

        // Database configuration
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.database.url = database_url;
        }
        if let Ok(max_connections_str) = std::env::var("DATABASE_MAX_CONNECTIONS") {
            config.database.max_connections = max_connections_str.parse().map_err(|e| {
                ConfigError::Invalid(format!("Invalid DATABASE_MAX_CONNECTIONS: {e}"))
            })?;
        }

        // Logging configuration
        if let Ok(log_level) = std::env::var("RUST_LOG") {
            config.logging.level = log_level;
        }
        if let Ok(log_format) = std::env::var("LOG_FORMAT") {
            config.logging.format = match log_format.as_str() {
                "json" => LogFormat::Json,
                "pretty" => LogFormat::Pretty,
                _ => return Err(ConfigError::Invalid("Invalid LOG_FORMAT. Use 'json' or 'pretty'".to_string())),
            };
        }

        // Task configuration
        if let Ok(max_concurrent_str) = std::env::var("TASK_MAX_CONCURRENT") {
            config.task.max_concurrent_tasks = max_concurrent_str.parse().map_err(|e| {
                ConfigError::Invalid(format!("Invalid TASK_MAX_CONCURRENT: {e}"))
            })?;
        }
        if let Ok(max_retries_str) = std::env::var("TASK_MAX_RETRIES") {
            config.task.max_retries = max_retries_str.parse().map_err(|e| {
                ConfigError::Invalid(format!("Invalid TASK_MAX_RETRIES: {e}"))
            })?;
        }

        Ok(config)
    }

    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.clone()));
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            ConfigError::Invalid(format!("Failed to read config file: {e}"))
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            ConfigError::Invalid(format!("Failed to parse config file: {e}"))
        })?;

        Ok(config)
    }

    pub fn from_file_or_env(path: &PathBuf) -> Result<Self, ConfigError> {
        match Self::from_file(path) {
            Ok(config) => Ok(config),
            Err(ConfigError::FileNotFound(_)) => Self::from_env(),
            Err(e) => Err(e),
        }
    }

    #[allow(dead_code)]
  pub fn ensure_data_dir(&self) -> Result<(), ConfigError> {
        // Reserved for future use when database storage is implemented
        if let Some(db_path) = self.database.url.strip_prefix("sqlite://") {
            let db_path = PathBuf::from(db_path);
            if let Some(parent) = db_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ConfigError::Invalid(format!("Failed to create data directory: {e}"))
                })?;
            }
        }
        Ok(())
    }
}