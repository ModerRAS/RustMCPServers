#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.server.timeout, 30);
        assert_eq!(config.server.workers, 4);
        
        assert_eq!(config.task.max_retries, 3);
        assert_eq!(config.task.timeout, 3600);
        assert_eq!(config.task.cleanup_interval, 3600);
        
        assert_eq!(config.database.url, "sqlite:///data/tasks.db");
        assert_eq!(config.database.max_connections, 10);
        
        assert_eq!(config.logging.level, "info");
        assert_eq!(config.logging.format, LogFormat::Json);
        
        assert_eq!(config.security.rate_limit, 1000);
        assert!(config.security.enable_cors);
        
        assert!(config.monitoring.metrics_enabled);
        assert_eq!(config.monitoring.metrics_port, 9090);
    }

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.timeout, 30);
        assert_eq!(config.workers, 4);
    }

    #[test]
    fn test_task_config_default() {
        let config = TaskConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.timeout, 3600);
        assert_eq!(config.cleanup_interval, 3600);
        assert_eq!(config.heartbeat_interval, 30);
        assert_eq!(config.worker_timeout, 1800);
    }

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.url, "sqlite:///data/tasks.db");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.connection_timeout, 30);
        assert_eq!(config.idle_timeout, 600);
        assert_eq!(config.max_lifetime, 3600);
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Json);
        assert!(config.file.is_none());
    }

    #[test]
    fn test_log_format_default() {
        let format = LogFormat::default();
        assert_eq!(format, LogFormat::Json);
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.api_key.is_none());
        assert_eq!(config.rate_limit, 1000);
        assert!(config.enable_cors);
    }

    #[test]
    fn test_monitoring_config_default() {
        let config = MonitoringConfig::default();
        assert!(config.metrics_enabled);
        assert_eq!(config.metrics_port, 9090);
        assert_eq!(config.metrics_interval, 60);
        assert_eq!(config.health_check_interval, 30);
    }

    #[test]
    fn test_app_config_from_env() {
        // 设置环境变量
        env::set_var("APP_SERVER_HOST", "127.0.0.1");
        env::set_var("APP_SERVER_PORT", "9090");
        env::set_var("APP_TASK_MAX_RETRIES", "5");
        env::set_var("RUST_LOG", "debug");
        env::set_var("APP_SECURITY_RATE_LIMIT", "500");
        env::set_var("APP_MONITORING_METRICS_INTERVAL", "120");

        let config = AppConfig::from_env().unwrap();

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.task.max_retries, 5);
        assert_eq!(config.logging.level, "debug");
        assert_eq!(config.security.rate_limit, 500);
        assert_eq!(config.monitoring.metrics_interval, 120);

        // 清理环境变量
        env::remove_var("APP_SERVER_HOST");
        env::remove_var("APP_SERVER_PORT");
        env::remove_var("APP_TASK_MAX_RETRIES");
        env::remove_var("RUST_LOG");
        env::remove_var("APP_SECURITY_RATE_LIMIT");
        env::remove_var("APP_MONITORING_METRICS_INTERVAL");
    }

    #[test]
    fn test_app_config_from_env_invalid_port() {
        env::set_var("APP_SERVER_PORT", "invalid");

        let result = AppConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid port"));

        env::remove_var("APP_SERVER_PORT");
    }

    #[test]
    fn test_app_config_from_env_invalid_max_retries() {
        env::set_var("APP_TASK_MAX_RETRIES", "invalid");

        let result = AppConfig::from_env();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid max retries"));

        env::remove_var("APP_TASK_MAX_RETRIES");
    }

    #[test]
    fn test_config_manager_new() {
        let manager = ConfigManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_config_manager_config() {
        let manager = ConfigManager::new().unwrap();
        let config = manager.config();
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_config_manager_server_config() {
        let manager = ConfigManager::new().unwrap();
        let server_config = manager.server_config();
        assert_eq!(server_config.host, "0.0.0.0");
    }

    #[test]
    fn test_config_manager_task_config() {
        let manager = ConfigManager::new().unwrap();
        let task_config = manager.task_config();
        assert_eq!(task_config.max_retries, 3);
    }

    #[test]
    fn test_config_manager_database_config() {
        let manager = ConfigManager::new().unwrap();
        let db_config = manager.database_config();
        assert_eq!(db_config.url, "sqlite:///data/tasks.db");
    }

    #[test]
    fn test_config_manager_logging_config() {
        let manager = ConfigManager::new().unwrap();
        let logging_config = manager.logging_config();
        assert_eq!(logging_config.level, "info");
    }

    #[test]
    fn test_config_manager_security_config() {
        let manager = ConfigManager::new().unwrap();
        let security_config = manager.security_config();
        assert_eq!(security_config.rate_limit, 1000);
    }

    #[test]
    fn test_config_manager_monitoring_config() {
        let manager = ConfigManager::new().unwrap();
        let monitoring_config = manager.monitoring_config();
        assert!(monitoring_config.metrics_enabled);
    }

    #[test]
    fn test_log_format_deserialization() {
        // 测试JSON格式
        let json_value = serde_json::json!("json");
        let json_format: LogFormat = serde_json::from_value(json_value).unwrap();
        assert_eq!(json_format, LogFormat::Json);

        // 测试Pretty格式
        let pretty_value = serde_json::json!("pretty");
        let pretty_format: LogFormat = serde_json::from_value(pretty_value).unwrap();
        assert_eq!(pretty_format, LogFormat::Pretty);
    }

    #[test]
    fn test_log_format_serialization() {
        let json_format = LogFormat::Json;
        let serialized = serde_json::to_string(&json_format).unwrap();
        assert_eq!(serialized, "\"json\"");

        let pretty_format = LogFormat::Pretty;
        let serialized = serde_json::to_string(&pretty_format).unwrap();
        assert_eq!(serialized, "\"pretty\"");
    }

    #[test]
    fn test_app_config_serialization() {
        let config = AppConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.len() > 0);
        
        // 验证可以反序列化
        let deserialized: AppConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.server.host, config.server.host);
        assert_eq!(deserialized.task.max_retries, config.task.max_retries);
    }

    #[test]
    fn test_app_config_partial_from_env() {
        // 只设置部分环境变量
        env::set_var("APP_SERVER_HOST", "192.168.1.1");
        env::set_var("APP_TASK_MAX_RETRIES", "7");

        let config = AppConfig::from_env().unwrap();

        // 验证设置的环境变量
        assert_eq!(config.server.host, "192.168.1.1");
        assert_eq!(config.task.max_retries, 7);

        // 验证默认值保持不变
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.task.timeout, 3600);

        // 清理环境变量
        env::remove_var("APP_SERVER_HOST");
        env::remove_var("APP_TASK_MAX_RETRIES");
    }

    #[test]
    fn test_config_manager_with_env_overrides() {
        env::set_var("APP_MONITORING_METRICS_ENABLED", "false");

        let manager = ConfigManager::new().unwrap();
        let monitoring_config = manager.monitoring_config();
        assert!(!monitoring_config.metrics_enabled);

        env::remove_var("APP_MONITORING_METRICS_ENABLED");
    }

    #[test]
    fn test_config_edge_cases() {
        // 测试边界值
        env::set_var("APP_SERVER_TIMEOUT", "0");
        env::set_var("APP_TASK_TIMEOUT", "1");
        env::set_var("APP_SECURITY_RATE_LIMIT", "0");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.timeout, 0);
        assert_eq!(config.task.timeout, 1);
        assert_eq!(config.security.rate_limit, 0);

        env::remove_var("APP_SERVER_TIMEOUT");
        env::remove_var("APP_TASK_TIMEOUT");
        env::remove_var("APP_SECURITY_RATE_LIMIT");
    }
}